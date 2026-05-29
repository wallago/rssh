use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use dioxus::{
    html::article,
    signals::{Signal, WritableExt},
};
use futures::future::join_all;
use miniflux_api::{ApiError, MinifluxApi};
use reqwest::Client;
use rusqlite::{Connection, params};

use crate::prelude::*;

pub async fn toggle_read(
    api: Arc<MinifluxApi>,
    db: Arc<Mutex<Connection>>,
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
    Some(())
}

pub async fn toggle_star(
    api: Arc<MinifluxApi>,
    db: Arc<Mutex<Connection>>,
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
    Some(())
}
