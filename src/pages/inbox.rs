use std::sync::Arc;

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use reqwest::Client;

use crate::components::prelude::*;

#[component]
pub fn InboxPage() -> Element {
    let api = use_context::<Arc<MinifluxApi>>();

    let mut search = use_signal(String::new);
    let mut filter = use_signal(|| Filter::Unread);
    let articles = use_resource(move || {
        tracing::debug!("load entries for inbox");
        let api = api.clone();
        async move {
            match api
                .get_entries(
                    None,
                    None,
                    Some(200),
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
            {
                Ok(entries) => Some(entries.iter().map(Article::from).collect::<Vec<Article>>()),
                Err(_) => None,
            }
        }
    });

    let filtered = use_memo(move || match articles() {
        Some(Some(articles)) => articles
            .into_iter()
            .filter(|a| match filter() {
                Filter::All => true,
                Filter::Unread => !a.is_read,
                Filter::Starred => a.is_starred,
                Filter::Archived => false,
            })
            .filter(|a| {
                let q = search().to_lowercase();
                q.is_empty()
                    || a.title.to_lowercase().contains(&q)
                    || a.source.to_lowercase().contains(&q)
            })
            .collect(),
        _ => Vec::new(),
    });

    let unread_count = use_memo(move || match articles() {
        Some(Some(articles)) => articles.iter().filter(|a| !a.is_read).count(),
        _ => 0,
    });

    let article_count = use_memo(move || match articles() {
        Some(Some(articles)) => articles.len(),
        _ => 0,
    });

    let position = format!("{}/{}", filtered.len(), article_count);

    let mut selected_id = use_signal(|| None::<String>);
    let current_id = selected_id().or_else(|| filtered.first().map(|a| a.id.clone()));

    rsx! {
        div { class: "page inbox-page",
            AppHeader {
                unread_count: unread_count(),
                synced_ago: "2m ago".to_string(),
            }
            SearchBar {
                value: search(),
                on_input: move |v| search.set(v),
            }
            FilterChips {
                current: filter(),
                on_change: move |f| filter.set(f),
            }
            div { class: "divider" }

            div { class: "article-list",
                {filtered().into_iter().map(|article| {
                    let id = article.id.clone();
                    let id_for_click = id.clone();
                    let is_selected = current_id.as_ref() == Some(&id);
                    rsx! {
                        ArticleRow {
                            key: "{id}",
                            article: article.clone(),
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
