use std::sync::{Arc, Mutex};

use dioxus::{html::article, prelude::*};
use miniflux_api::MinifluxApi;
use reqwest::Client;

use rssh::prelude::*;
use rusqlite::Connection;

use crate::{
    Route,
    components::{
        app::{AppHeader, FilterChips},
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
            AppHeader {
                unread_count: match tree() {
                    Load::Ready(cats) => cats.iter().map(|c| match &c.feeds {
                        Load::Ready(feeds) => feeds.iter().map(|f| match &f.articles {
                            Load::Ready(articles) => articles.iter().filter(|a| !a.is_read).count(),
                            _ => 0,
                        }).sum::<usize>(),
                        _ => 0,
                    }).sum::<usize>(),
                    _ => 0,
                },
                synced_ago: String::new()
            }
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
    let filter = use_context::<Signal<Filter>>();
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();

    rsx! {
        CategoryRow {
            category: node.category.clone(),
            count: match node.feeds.clone() {
                Load::Ready(feeds) => {
                    let articles = feeds.into_iter().map(|feed| match feed.articles {
                        Load::Ready(articles) => {
                            articles.iter()
                                .cloned()
                                .filter(|a| a.matches(filter()))
                                .count()
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
                move |_| {
                    toggle_category(tree, &id)
                }
            },
            on_swipe_left: {
                let node = node.clone();
                let api    = api.clone();
                let db     = db.clone();
                move |_| {
                    let api    = api.clone();
                    let db     = db.clone();
                    let node = node.clone();
                    spawn(async move {
                        mark_category_read(api, db, node.clone()).await;
                        set_category_articles_read(tree, node);
                    });
                }
            },
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
    let filter = use_context::<Signal<Filter>>();
    let feed_id = node.feed.id.clone();

    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();

    rsx! {
        FeedRow {
            feed: node.feed.clone(),
            count: match node.articles.clone() {
                Load::Ready(articles) => {
                    Some(articles.iter()
                        .cloned()
                        .filter(|a| a.matches(filter()))
                        .count() as i64)
                },
                _ => None,
            },
            expanded: node.expanded,
            on_click: move |_| toggle_feed(tree, &id, &feed_id),
            on_swipe_left: {
                let node = node.clone();
                let api    = api.clone();
                let db     = db.clone();
                move |_| {
                    let api    = api.clone();
                    let db     = db.clone();
                    let node = node.clone();
                    spawn(async move {
                        mark_feed_read(api, db, node.clone()).await;
                        set_feed_articles_read(tree, node);
                    });
                }
            },
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
                    Load::Ready(articles)      => {
                        let visible: Vec<_> = articles.iter()
                            .cloned()
                            .filter(|a| a.matches(filter()))
                            .collect();
                        rsx! {
                            {visible.iter().cloned().enumerate().map(|(i, article)| {
                                let api = api.clone();
                                let db  = db.clone();
                                rsx! {
                                    ArticleRow {
                                        key: "{article.id}",
                                        article: article.clone(),
                                        is_selected: false,
                                        on_click: {
                                            let id = article.id.clone();
                                            move |_| {
                                                navigator().push(Route::ReaderPage { id: id.clone() });
                                            }
                                        },
                                        on_swipe_left: {
                                            let id = article.id.clone();
                                            let api = api.clone();
                                            let db  = db.clone();
                                            move |_| {
                                                let api = api.clone();
                                                let db  = db.clone();
                                                let id = id.clone();
                                                spawn(async move {
                                                    mark_read(api, db, id.clone(), true).await;
                                                    set_article_read(tree, id, true);
                                                });
                                            }
                                        },
                                        on_swipe_right: {
                                            let id = article.id.clone();
                                            let next = !article.is_starred;
                                            let api = api.clone();
                                            let db  = db.clone();
                                            move |_| {
                                                let id = id.clone();
                                                let api = api.clone();
                                                let db  = db.clone();
                                                spawn(async move {
                                                    toggle_star(api, db, id.clone()).await;
                                                    set_article_starred(tree, id, next);
                                                });
                                            }
                                        },
                                    },
                                }
                            })}
                        }
                    },
                } }
            }
        }
    }
}
