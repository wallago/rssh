use std::sync::{Arc, Mutex};

use dioxus::signals::{Signal, WritableExt};
use miniflux_api::MinifluxApi;
use reqwest::Client;
use rusqlite::{Connection, params};

use crate::prelude::*;

pub async fn toggle_read(
    api: Arc<MinifluxApi>,
    db: Arc<Mutex<Connection>>,
    tree: Signal<Option<Vec<CategoryNode>>>,
    mut notice: Signal<Option<Notice>>,
    mut article: Article,
) -> Option<()> {
    let status = if !article.is_read {
        miniflux_api::models::EntryStatus::Read
    } else {
        miniflux_api::models::EntryStatus::Unread
    };
    let s: &str = status.into();
    db.lock()
        .ok()?
        .execute(
            "UPDATE entries SET status=?1 WHERE id=?2",
            params![s, article.id],
        )
        .ok()?;
    let _ = api
        .update_entries_status(vec![article.id], status, &Client::new())
        .await
        .ok()?;
    article.is_read = !article.is_read;
    update_article(tree, article.id, move |a| a.is_read = article.is_read);
    notice.set(Some(Notice::info(if article.is_read {
        "Marked as read"
    } else {
        "Marked as unread"
    })));
    Some(())
}

pub async fn toggle_star(
    api: Arc<MinifluxApi>,
    db: Arc<Mutex<Connection>>,
    tree: Signal<Option<Vec<CategoryNode>>>,
    mut notice: Signal<Option<Notice>>,
    mut article: Article,
) -> Option<()> {
    db.lock()
        .ok()?
        .execute(
            "UPDATE entries SET starred = NOT starred WHERE id=?1",
            params![article.id],
        )
        .ok()?;
    let _ = api.toggle_bookmark(article.id, &Client::new()).await.ok()?;
    article.is_starred = !article.is_starred;
    update_article(tree, article.id, move |a| a.is_starred = article.is_starred);
    notice.set(Some(Notice::info(if article.is_starred {
        "Starred"
    } else {
        "Unstarred"
    })));
    Some(())
}
