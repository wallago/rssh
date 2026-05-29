use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use dioxus::signals::{Signal, WritableExt};
use futures::future::join_all;
use miniflux_api::{ApiError, MinifluxApi};
use reqwest::Client;
use rusqlite::{Connection, params};

pub mod prelude {
    pub use crate::utils::article::*;
    pub use crate::utils::tree::*;
    pub use crate::utils::*;
}

mod article;
mod tree;

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

pub async fn refresh(api: Arc<MinifluxApi>, db: Arc<Mutex<Connection>>) -> Option<()> {
    let (c, f, e) = fetch_all(&api, &Client::new()).await?;
    let mut conn = db.lock().ok()?;
    write_all(&mut *conn, &c, &f, &e).ok()
}

pub async fn sync_and_load(
    api: Arc<MinifluxApi>,
    db: Arc<Mutex<Connection>>,
    mut notice: Signal<Option<Notice>>,
    mut tree: Signal<Option<Vec<CategoryNode>>>,
) -> anyhow::Result<()> {
    use anyhow::Context;
    let empty = {
        let mut conn = db.lock().ok().context("access to DB failed")?;
        is_empty(&mut *conn).context("checking cache")?
    };

    if empty {
        notice.set(Some(Notice::sync("initial sync…")));
        refresh(api, db.clone())
            .await
            .ok_or_else(|| anyhow::anyhow!("initial sync failed"))?;
    }

    notice.set(Some(Notice::sync("loading…")));
    let (cats, feeds, articles) = {
        let mut conn = db.lock().ok().context("access to DB failed")?;
        (
            load_categories(&conn).context("load categories")?,
            load_feeds(&conn).context("load feeds")?,
            load_articles(&conn).context("load articles")?,
        )
    };

    tree.set(Some(build_tree(cats, feeds, articles)));
    notice.set(None);
    Ok(())
}

pub fn sibling_article(
    db: Arc<Mutex<Connection>>,
    article: Article,
    filter: Filter,
    direction: i64,
) -> Option<i64> {
    let articles = load_articles(&mut *db.lock().ok()?).ok()?;
    let mut i = article.id + direction;
    while i >= 0 && (i as usize) < articles.len() {
        let a = &articles[i as usize];
        if a.matches(filter) {
            return Some(a.id.clone());
        }
        i += direction;
    }
    None
}
