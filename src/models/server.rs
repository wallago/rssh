use anyhow::Context;
use reqwest::Client;
use std::sync::{Arc, Mutex};

use miniflux_api::MinifluxApi;
use rusqlite::{Connection, params};

use crate::{
    db::ConnectionExt,
    models::{
        notice::Retry,
        state::{NOTICE, TREE},
    },
    prelude::{Article, MinifluxApiExt, Notice, Tree},
};

/// RSS server and Local DB
#[derive(Clone)]
pub struct Server {
    /// Miniflux API
    pub api: Arc<MinifluxApi>,
    /// SQLITE Database connection
    pub db: Arc<Mutex<Connection>>,
}

impl Server {
    /// Open the local DB and the Miniflux client (used by the `SERVER` global)
    pub fn init() -> Self {
        let db = Connection::get_db_conn().expect("failed to get DB connection");
        let api = MinifluxApi::get_api_conn().expect("failed to get Miniflux connection");
        Self { api, db }
    }

    /// Refresh the DB with synchronized values
    async fn refresh_db(&self) -> anyhow::Result<()> {
        let (c, f, e) = (*self.api).fetch_all(&Client::new()).await?;
        let mut conn = self
            .db
            .lock()
            .map_err(|_| anyhow::anyhow!("DB lock poisoned"))?;
        conn.write_all(&c, &f, &e).context("failed to update DB")
    }

    /// Sync with Miniflux when needed (or forced), then rebuild `TREE` from the DB
    pub async fn sync_and_load(&self, force: bool) -> anyhow::Result<()> {
        let empty = {
            let conn = self.db.lock().ok().context("access to DB failed")?;
            conn.is_empty().context("checking cache")?
        };

        if empty || force {
            *NOTICE.write() = Some(Notice::sync(if force {
                "refreshing…"
            } else {
                "initial sync…"
            }));
            self.refresh_db().await?;
        }

        *NOTICE.write() = Some(Notice::sync("loading…"));
        let (cats, feeds, articles) = {
            let conn = self.db.lock().ok().context("access to DB failed")?;
            (
                (*conn).load_categories().context("load categories")?,
                (*conn).load_feeds().context("load feeds")?,
                (*conn).load_articles().context("load articles")?,
            )
        };

        TREE.build(cats, feeds, articles);
        *NOTICE.write() = None;
        Ok(())
    }

    /// `sync_and_load` + the standard error toast with retry — call this from the UI
    pub async fn sync_app(&self, force: bool) {
        if let Err(e) = self.sync_and_load(force).await {
            tracing::error!("sync failed: {e:?}");
            *NOTICE.write() = Some(Notice::error(
                format!("Couldn't load data: {e}"),
                Some(Retry::SyncApp),
            ));
        }
    }

    /// Mark a batch of entries read at once: one SQL UPDATE, one Miniflux
    /// call, one tree write, one toast
    pub async fn mark_read(&self, ids: Vec<i64>) -> anyhow::Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        {
            let conn = self
                .db
                .lock()
                .map_err(|e| anyhow::anyhow!("failed to lock db connection: {e}"))?;
            let placeholders = vec!["?"; ids.len()].join(",");
            conn.execute(
                &format!("UPDATE entries SET status='read' WHERE id IN ({placeholders})"),
                rusqlite::params_from_iter(ids.iter()),
            )?;
        }
        self.api
            .update_entries_status(
                ids.clone(),
                miniflux_api::models::EntryStatus::Read,
                &Client::new(),
            )
            .await?;
        TREE.mark_read(&ids);
        *NOTICE.write() = Some(Notice::info(format!("{} marked as read", ids.len())));
        Ok(())
    }

    /// Flip read/unread in the local DB and on Miniflux, then mirror it on `article`
    pub async fn toggle_read_status(&self, article: &mut Article) -> anyhow::Result<()> {
        let status = if !article.is_read {
            miniflux_api::models::EntryStatus::Read
        } else {
            miniflux_api::models::EntryStatus::Unread
        };
        let s: &str = status.into();
        self.db
            .lock()
            .map_err(|e| anyhow::anyhow!("failed to lock db connection: {}", e))?
            .execute(
                "UPDATE entries SET status=?1 WHERE id=?2",
                params![s, article.id],
            )?;
        self.api
            .update_entries_status(vec![article.id], status, &Client::new())
            .await?;
        article.is_read = !article.is_read;
        TREE.update_article(article.id, |a| a.is_read = article.is_read);
        *NOTICE.write() = Some(Notice::info(if article.is_read {
            "Marked as read"
        } else {
            "Marked as unread"
        }));

        Ok(())
    }

    /// Flip starred in the local DB and on Miniflux, then mirror it on `article`
    pub async fn toggle_star_status(&self, article: &mut Article) -> anyhow::Result<()> {
        self.db
            .lock()
            .map_err(|e| anyhow::anyhow!("failed to lock db connection: {}", e))?
            .execute(
                "UPDATE entries SET starred = NOT starred WHERE id=?1",
                params![article.id],
            )?;
        self.api.toggle_bookmark(article.id, &Client::new()).await?;
        article.is_starred = !article.is_starred;
        TREE.update_article(article.id, |a| a.is_starred = article.is_starred);
        *NOTICE.write() = Some(Notice::info(if article.is_starred {
            "Marked as star"
        } else {
            "Marked as unstar"
        }));
        Ok(())
    }
}
