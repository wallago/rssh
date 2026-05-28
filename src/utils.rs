use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use dioxus::{
    html::article,
    signals::{Signal, WritableExt},
};
use miniflux_api::{ApiError, MinifluxApi};
use reqwest::Client;
use rusqlite::{Connection, params};

use crate::prelude::*;

pub fn string_to_color(s: &str) -> String {
    // generate hash form string (not randomly)
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish(); // hash from srting

    // get HLS code
    let hue = (hash % 360) as f64; // Hue (0–360°): position on the color wheel (red → yellow → green → cyan → blue → magenta → red)
    let saturation = 0.65; // Saturation (0–1): grayness vs. vividness
    let lightness = 0.60; //Lightness (0–1): black → color → white
    let chroma = (1.0 - (2.0 * lightness - 1.0)) * saturation; // Chroma is the magnitude of colorfulness
    let sector = hue / 60.0; // Divid Hue in sector (in degree)
    let x = chroma * (1.0 - ((sector % 2.0) - 1.0).abs()); // Second color channel's value within the sector
    let m = lightness - chroma / 2.0; // Lightness offset added to all three channels equally
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let to = |v: f64| ((v + m) * 255.0).round() as u8; // Scales to 0–255

    format!("#{:02x}{:02x}{:02x}", to(r), to(g), to(b))
}

pub fn toggle_category(mut tree: Signal<Load<Vec<CategoryNode>>>, id: &str) {
    let mut guard = tree.write();
    let Load::Ready(cats) = &mut *guard else {
        return;
    };
    if let Some(node) = cats.iter_mut().find(|c| c.category.id == id) {
        node.expanded = !node.expanded;
    }
}

pub fn toggle_feed(mut tree: Signal<Load<Vec<CategoryNode>>>, cat_id: &str, id: &str) {
    let mut guard = tree.write();
    let Load::Ready(cats) = &mut *guard else {
        return;
    };
    if let Some(cat) = cats.iter_mut().find(|c| c.category.id == cat_id) {
        if let Some(node) = match &mut cat.feeds {
            Load::Ready(feeds) => feeds.iter_mut().find(|f| f.feed.id == id),
            _ => return,
        } {
            node.expanded = !node.expanded;
        }
    }
}

pub fn build_tree(
    cats: Vec<Category>,
    feeds: Vec<Feed>,
    articles: Vec<Article>,
) -> Vec<CategoryNode> {
    cats.into_iter()
        .map(|category| {
            let cat_feeds = feeds
                .clone()
                .into_iter()
                .filter(|f| f.category.id == category.id)
                .map(|feed| {
                    let feed_articles = articles
                        .clone()
                        .into_iter()
                        .filter(|a| a.feed.id == feed.id)
                        .collect();
                    FeedNode {
                        feed,
                        expanded: false,
                        articles: Load::Ready(feed_articles),
                    }
                })
                .collect::<Vec<FeedNode>>();
            CategoryNode {
                category,
                expanded: false,
                feeds: Load::Ready(cat_feeds),
            }
        })
        .collect()
}

pub async fn initial_sync(
    api: Arc<MinifluxApi>,
    db: Arc<Mutex<Connection>>,
) -> Result<(), ApiError> {
    let empty = { is_empty(&db.lock().unwrap()).unwrap_or(true) };
    if empty {
        let (c, f, e) = fetch_all(&api, &Client::new()).await?;
        write_all(&mut db.lock().unwrap(), &c, &f, &e).ok();
    }
    Ok(())
}

pub async fn refresh(api: Arc<MinifluxApi>, db: Arc<Mutex<Connection>>) -> Result<(), ApiError> {
    let (c, f, e) = fetch_all(&api, &Client::new()).await?;
    write_all(&mut db.lock().unwrap(), &c, &f, &e).ok();
    Ok(())
}

pub async fn mark_read(
    api: Arc<MinifluxApi>,
    db: Arc<Mutex<Connection>>,
    entry_id: i64,
    read: bool,
) {
    let status = if read {
        miniflux_api::models::EntryStatus::Read
    } else {
        miniflux_api::models::EntryStatus::Unread
    };
    {
        let s: &str = status.into();
        db.lock()
            .unwrap()
            .execute(
                "UPDATE entries SET status=?1 WHERE id=?2",
                params![s, entry_id],
            )
            .ok();
    }
    let _ = api
        .update_entries_status(vec![entry_id], status, &Client::new())
        .await; // push
}

pub async fn toggle_star(api: Arc<MinifluxApi>, db: Arc<Mutex<Connection>>, entry_id: i64) {
    {
        db.lock()
            .unwrap()
            .execute(
                "UPDATE entries SET starred = NOT starred WHERE id=?1",
                params![entry_id],
            )
            .ok();
    }
    let _ = api.toggle_bookmark(entry_id, &Client::new()).await; // server-side toggle
}

pub fn set_article_read(mut tree: Signal<Load<Vec<CategoryNode>>>, article_id: &str, read: bool) {
    let mut guard = tree.write();
    let Load::Ready(cats) = &mut *guard else {
        return;
    };

    for cat in cats.iter_mut() {
        let Load::Ready(feeds) = &mut cat.feeds else {
            continue;
        };
        for feed in feeds.iter_mut() {
            let Load::Ready(articles) = &mut feed.articles else {
                continue;
            };
            if let Some(a) = articles.iter_mut().find(|a| a.id == article_id) {
                a.is_read = read;
                return;
            }
        }
    }
}

pub fn find_article(tree: &Load<Vec<CategoryNode>>, article_id: &str) -> Option<Article> {
    let Load::Ready(cats) = tree else { return None };
    for cat in cats {
        let Load::Ready(feeds) = &cat.feeds else {
            continue;
        };
        for feed in feeds {
            let Load::Ready(articles) = &feed.articles else {
                continue;
            };
            if let Some(a) = articles.iter().find(|a| a.id == article_id) {
                return Some(a.clone());
            }
        }
    }
    None
}
