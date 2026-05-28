use std::sync::Arc;

use miniflux_api::MinifluxApi;
use reqwest::Client;

pub async fn fetch_all(
    api: &MinifluxApi,
    client: &Client,
) -> Result<
    (
        Vec<miniflux_api::models::Category>,
        Vec<miniflux_api::models::Feed>,
        Vec<miniflux_api::models::Entry>,
    ),
    miniflux_api::ApiError,
> {
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
