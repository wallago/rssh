use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use reqwest::Client;

use rssh::prelude::*;
use rusqlite::Connection;

use crate::{
    Route,
    components::{
        row::{ArticleRow, CategoryRow, FeedRow},
        skeleton::SkeletonRow,
    },
    pages::reader::ReaderPage,
};

#[component]
pub fn InboxPage() -> Element {
    let api = use_context::<Arc<MinifluxApi>>();
    let tree = use_context::<Signal<Load<Vec<CategoryNode>>>>();

    rsx! {
        div { class: "page inbox-page",
            { match tree() {
                Load::Idle | Load::Loading => rsx! {
                   div { class: "tree",
                       SkeletonRow { width: "long" }
                       SkeletonRow { width: "medium" }
                       SkeletonRow { width: "short" }
                       SkeletonRow { width: "medium" }
                       SkeletonRow { width: "long" }
                   }
                },
                Load::Failed(err)          => rsx! { div { class: "error", "{err}" } },
                Load::Ready(categories)    => rsx! {
                    div { class: "tree",
                        for cat in categories {
                            CategoryNodeView { key: "{cat.category.id}", node: cat }
                        }
                    }
                },
            } }
        }
    }
}

#[component]
fn CategoryNodeView(node: CategoryNode) -> Element {
    let tree = use_context::<Signal<Load<Vec<CategoryNode>>>>();

    rsx! {
        CategoryRow {
            category: node.category.clone(),
            unread_count: match node.feeds.clone() {
                Load::Ready(feeds) => {
                    let articles = feeds.into_iter().map(|feed| match feed.articles {
                        Load::Ready(articles) => {
                            articles.into_iter().filter(|a| !a.is_read).count()
                        },
                        _ => 0,
                    }).sum::<usize>() as i64;
                    Some(articles)
                },
                _ => None,
            },
            expanded: node.expanded,
            on_click: {
                let id = node.category.id.clone();
                move |_| toggle_category(tree, &id)
            }
        }
        if node.expanded {
            div { class: "feed-list",
                { match &node.feeds {
                    Load::Idle | Load::Loading => rsx! {
                       div { class: "tree",
                           SkeletonRow { width: "medium" }
                           SkeletonRow { width: "short" }
                       }
                    },
                    Load::Failed(e)            => rsx! { div { class: "error", "{e}" } },
                    Load::Ready(feeds)         => rsx! {
                        for feed in feeds.iter().cloned() {
                            FeedNodeView { key: "{feed.feed.id}", node: feed, id: node.category.id.clone() }
                        }
                    },
                } }
            }
        }
    }
}

#[component]
fn FeedNodeView(node: FeedNode, id: String) -> Element {
    let mut tree = use_context::<Signal<Load<Vec<CategoryNode>>>>();
    let feed_id = node.feed.id.clone();

    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();

    rsx! {
        FeedRow {
            feed: node.feed.clone(),
            unread_count: match node.articles.clone() {
                Load::Ready(articles) => {
                    Some(articles.into_iter().filter(|a| !a.is_read).count() as i64)
                },
                _ => None,
            },
            expanded: node.expanded,
            on_click: move |_| toggle_feed(tree, &id, &feed_id),
        }
        if node.expanded {
            div { class: "article-list",
                { match &node.articles {
                    Load::Idle | Load::Loading => rsx! {
                        div { class: "tree",
                            SkeletonRow { width: "long" }
                            SkeletonRow { width: "long" }
                            SkeletonRow { width: "medium" }
                        }
                    },
                    Load::Failed(e)            => rsx! { div { class: "error", "{e}" } },
                    Load::Ready(articles)      => rsx! {
                        {articles.iter().cloned().enumerate().map(|(i, article)| {
                            let id  = article.id.parse::<i64>().unwrap_or_default();
                            let api = api.clone();
                            let db  = db.clone();
                            rsx! {
                                ArticleRow {
                                    key: "{article.id}",
                                    article: article.clone(),
                                    is_selected: false,
                                    on_click: {
                                        let aid = article.id.clone();
                                    move |_| { navigator().push(Route::ReaderPage { id: aid.clone() }); }
                                    },
                                    on_swipe: move |_| {
                                        let api = api.clone();
                                        let db  = db.clone();
                                        let aid  = article.id.clone();
                                        spawn(async move {
                                            mark_read(api, db, id, true).await;
                                            set_article_read(tree, &aid, true);
                                        });
                                    },
                                },
                            }
                        })}
                    },
                } }
            }
        }
    }
}
