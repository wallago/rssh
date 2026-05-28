use std::sync::Arc;

use miniflux_api::MinifluxApi;
use reqwest::Client;

pub fn get_api_conn() -> Option<Arc<MinifluxApi>> {
    let url = Url::from_str(env!("MINIFLUX_URL"))?;
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
) -> Some<(
    Vec<miniflux_api::models::Category>,
    Vec<miniflux_api::models::Feed>,
    Vec<miniflux_api::models::Entry>,
)> {
    let categories = api.get_categories(client).await.ok()?;
    let feeds = api.get_feeds(client).await.ok()?;
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
            .await
            .ok()?;
        let got = page.len() as i64;
        entries.extend(page);
        if got < limit {
            break;
        }
        offset += limit;
    }
    Some((categories, feeds, entries))
}
