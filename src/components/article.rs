use dioxus::prelude::*;
use miniflux_api::models::{Entry, EntryStatus};
use strum::EnumIter;

use crate::utils::string_to_color;

#[derive(Clone, Copy, PartialEq, Debug, EnumIter, strum::Display)]
pub enum Filter {
    All,
    Unread,
    Starred,
    Archived,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Article {
    pub id: String,
    pub source: String,
    pub source_color: String,
    pub title: String,
    pub snippet: String,
    pub timestamp: String,
    pub is_read: bool,
    pub is_starred: bool,
}

impl From<&Entry> for Article {
    fn from(entry: &Entry) -> Self {
        let read_status: &str = EntryStatus::Read.into();
        Article {
            id: entry.id.to_string(),
            source: entry.feed.title.clone(),
            source_color: string_to_color(&entry.feed.title),
            title: entry.title.clone(),
            snippet: entry.content.clone(),
            timestamp: entry.published_at.clone(),
            is_read: entry.status == read_status,
            is_starred: entry.starred,
        }
    }
}

#[component]
pub fn ArticleRow(article: Article, is_selected: bool, on_click: EventHandler<()>) -> Element {
    let class = match (is_selected, article.is_read) {
        (true, true) => "article-row selected read",
        (true, false) => "article-row selected",
        (false, true) => "article-row read",
        (false, false) => "article-row",
    };

    rsx! {
        div {
            class: "{class}",
            onclick: move |_| on_click.call(()),

            div {
                class: "source-bar",
                style: "background-color: {article.source_color}",
            }

            div { class: "article-content",
                div { class: "article-meta",
                    span {
                        class: "source-name",
                        style: "color: {article.source_color}",
                        "{article.source}"
                    }
                    div { class: "meta-right",
                        if article.is_starred {
                            span { class: "star", "★" }
                        }
                        span { class: "timestamp", "{article.timestamp}" }
                    }
                }
                div { class: "article-title", "{article.title}" }
                div { class: "article-snippet", "{article.snippet}" }
            }
        }
    }
}
