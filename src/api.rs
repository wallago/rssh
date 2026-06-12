use std::{str::FromStr, sync::Arc};

use miniflux_api::MinifluxApi;
use reqwest::{Client, Url};

/// MinifluxApi extra functions
pub trait MinifluxApiExt {
    /// Build a Miniflux client from the compile-time
    fn get_api_conn() -> Option<Arc<MinifluxApi>>;
    ///  Fetch the full server state — every category, feed, and entry
    fn fetch_all(
        &self,
        client: &Client,
    ) -> impl std::future::Future<
        Output = anyhow::Result<(
            Vec<miniflux_api::models::Category>,
            Vec<miniflux_api::models::Feed>,
            Vec<miniflux_api::models::Entry>,
        )>,
    > + Send;
}

impl MinifluxApiExt for MinifluxApi {
    fn get_api_conn() -> Option<Arc<Self>> {
        let url = Url::from_str(env!("MINIFLUX_URL")).ok()?;
        let usename = env!("MINIFLUX_USERNAME");
        let passwd = env!("MINIFLUX_PASSWORD");
        Some(Arc::new(Self::new(
            &url,
            usename.to_string(),
            passwd.to_string(),
        )))
    }

    async fn fetch_all(
        &self,
        client: &Client,
    ) -> anyhow::Result<(
        Vec<miniflux_api::models::Category>,
        Vec<miniflux_api::models::Feed>,
        Vec<miniflux_api::models::Entry>,
    )> {
        let categories = self.get_categories(client).await?;
        let feeds = self.get_feeds(client).await?;
        let mut entries = Vec::new();
        let (mut offset, limit) = (0i64, 250i64);
        loop {
            let page = self
                .get_entries(
                    None,
                    Some(offset),
                    Some(limit),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    client,
                )
                .await?;
            let got = page.len() as i64;
            entries.extend(page);
            if got < limit {
                break;
            }
            offset += limit;
        }
        Ok((categories, feeds, entries))
    }
}
