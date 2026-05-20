use dioxus::prelude::*;
use miniflux_api::models::{Entry, EntryStatus};

#[derive(Clone, Copy, PartialEq, Debug)]
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

#[component]
pub fn ArticleRow(article: Entry, is_selected: bool, on_click: EventHandler<()>) -> Element {
    let read_status: &str = EntryStatus::Read.into();
    let class = match (is_selected, article.status == read_status) {
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
            //     style: "background-color: {article.source_color}",
            }

            div { class: "article-content",
                div { class: "article-meta",
                    span {
                        class: "source-name",
                        // style: "color: {article.source_color}",
                        "{article.url}"
                    }
                    div { class: "meta-right",
                        if article.starred {
                            span { class: "star", "★" }
                        }
                        span { class: "timestamp", "{article.published_at}" }
                    }
                }
                div { class: "article-title", "{article.title}" }
                div { class: "article-snippet", "{article.content}" }
            }
        }
    }
}
