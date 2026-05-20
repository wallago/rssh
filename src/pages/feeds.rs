use std::sync::Arc;

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use miniflux_api::models::{Entry, EntryStatus};
use reqwest::Client;

use crate::components::prelude::*;

#[component]
pub fn InboxPage() -> Element {
    let api = use_context::<Arc<MinifluxApi>>();

    let mut search = use_signal(String::new);
    let mut filter = use_signal(|| Some(EntryStatus::Unread));
    let articles = use_resource(move || {
        let api = api.clone();
        async move {
            api.get_entries(
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                &Client::new(),
            )
            .await
        }
    });

    let filtered: Vec<Entry> = match &*articles.read() {
        Some(Ok(articles)) => articles.iter().cloned().collect(),
        _ => Vec::new(),
    };
    // .filter(|a| match filter() {
    //     Filter::All => true,
    //     Filter::Unread => !a.is_read,
    //     Filter::Starred => a.is_starred,
    //     Filter::Archived => false,
    // })
    // .filter(|a| {
    //     let q = search().to_lowercase();
    //     q.is_empty()
    //         || a.title.to_lowercase().contains(&q)
    //         || a.source.to_lowercase().contains(&q)
    // })

    let unread_count: usize = match &*articles.read() {
        Some(Ok(articles)) => articles
            .iter()
            .filter(|a| {
                let unread_status: &str = EntryStatus::Unread.into();
                a.status == unread_status
            })
            .count(),
        _ => 0,
    };

    let article_count: usize = match &*articles.read() {
        Some(Ok(articles)) => articles.len(),
        _ => 0,
    };

    let position = format!("{}/{}", filtered.len(), article_count);
    let mut selected_id = use_signal(|| match &*articles.read() {
        Some(Ok(articles)) => Some(articles.first().unwrap().id),
        _ => None,
    });

    rsx! {
        div { class: "page inbox-page",
            AppHeader {
                unread_count,
                synced_ago: "2m ago".to_string(),
            }
            SearchBar {
                value: search(),
                on_input: move |v| search.set(v),
            }
            // FilterChips {
            //     current: filter(),
            //     on_change: move |f| filter.set(f),
            // }
            div { class: "divider" }

            div { class: "article-list",
                {filtered.iter().cloned().map(|article| {
                    let id = article.id.clone();
                    let id_for_click = id.clone();
                    let is_selected = selected_id() == Some(id.clone());
                    rsx! {
                        ArticleRow {
                            key: "{id}",
                            article,
                            is_selected,
                            on_click: move |_| selected_id.set(Some(id_for_click.clone())),
                        }
                    }
                })}
            }

            StatusLine {
                mode: "NORMAL".to_string(),
                position,
                context: "main · unread".to_string(),
                last_sync: "2m".to_string(),
            }
        }
    }
}
