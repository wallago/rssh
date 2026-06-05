use std::{str::FromStr, sync::Arc};

use miniflux_api::MinifluxApi;
use reqwest::{Client, Url};

pub fn get_api_conn() -> Option<Arc<MinifluxApi>> {
    let url = Url::from_str(env!("MINIFLUX_URL")).ok()?;
    let usename = env!("MINIFLUX_USERNAME");
    let passwd = env!("MINIFLUX_PASSWORD");
    Some(Arc::new(MinifluxApi::new(
        &url,
        usename.to_string(),
        passwd.to_string(),
    )))
}

pub async fn fetch_all(
    api: &MinifluxApi,
    client: &Client,
) -> anyhow::Result<(
    Vec<miniflux_api::models::Category>,
    Vec<miniflux_api::models::Feed>,
    Vec<miniflux_api::models::Entry>,
)> {
    let categories = api.get_categories(client).await?;
    let feeds = api.get_feeds(client).await?;
    let mut entries = Vec::new();
    let (mut offset, limit) = (0i64, 250i64);
    loop {
        let page = api
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
