use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use rusqlite::Connection;

use rssh::prelude::*;

use crate::{
    Route,
    components::{article::ArticleRow, skeleton::SkeletonRow},
};

#[component]
pub fn FeedRow(
    feed: Feed,
    count: usize,
    expanded: bool,
    on_click: EventHandler<()>,
    on_swipe_left: EventHandler<()>,
) -> Element {
    let class = if expanded {
        "feed-row expanded"
    } else {
        "feed-row"
    };
    let chevron = if expanded { "▼" } else { "▶" };

    let mut start = use_signal(|| (0.0_f64, 0.0_f64));
    let mut dx = use_signal(|| 0.0_f64);
    let mut horizontal = use_signal(|| false);
    let mut moved = use_signal(|| false);
    let threshold = 100.0;
    let offset = dx();

    rsx! {
        div { class: "swipe-row",
            div { class: "swipe-action swipe-action-right", "Read" }
            div {
                class: "{class}",
                style: "transform: translateX({offset}px)",
                onpointerdown: move |e| {
                    let p = e.client_coordinates();
                    start.set((p.x, p.y));
                    horizontal.set(false);
                    moved.set(false);
                },
                onpointermove: move |e| {
                    let (sx, sy) = start();
                    let p = e.client_coordinates();
                    let (mx, my) = (p.x - sx, p.y - sy);
                    if !horizontal() && mx.abs() > 8.0 && mx.abs() > my.abs() {
                        horizontal.set(true);
                    }
                    if horizontal() {
                        moved.set(true);
                        dx.set(mx.min(0.0));
                    }
                },
                onpointerup: move |_| {
                    if dx() <= -threshold { on_swipe_left.call(()); }
                    dx.set(0.0);
                    horizontal.set(false);
                },
                onclick: move |_| {
                    if moved() { return; }
                    on_click.call(())
                },

                div { class: "row-content",
                    span { class: "chevron", "{chevron}" }
                    span { class: "row-title", "{feed.label}" }
                    div { class: "row-meta",
                        if feed.error_count > 0 {
                            span { class: "feed-error", title: "Feed has fetch errors", "⚠" }
                        }
                        span { class: "unread-count", "{count}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn FeedNodeView(node: ReadSignal<FeedNode>) -> Element {
    let tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();
    let filter = use_context::<Signal<Filter>>();

    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();
    let notice = use_context::<Signal<Option<Notice>>>();

    rsx! {
        FeedRow {
            feed: node().feed,
            count:  iter_articles(tree().unwrap_or_default(), Some(node().feed.category.id), Some(node().feed.id), Some(filter())).count(),
            expanded: node().expanded,
            on_click: move |_| toggle_feed(tree, node()),
            on_swipe_left: {
                let api = api.clone();
                let db = db.clone();
                move |_| {
                    let api = api.clone();
                    let db = db.clone();
                    spawn(async move {
                        if let Some(cats) = tree() {
                            let articles = iter_articles(cats, Some(node().feed.category.id), Some(node().feed.id), Some(Filter::Unread));
                            for a in articles {
                                toggle_read(api.clone(), db.clone(), tree, notice, a).await;
                            }
                        }
                    });
                }
            },
        }
        if node().expanded {
            div { class: "article-list",
                { match &node().articles {
                    None => rsx! {
                        div { class: "tree",
                            SkeletonRow { width: "long" }
                            SkeletonRow { width: "long" }
                            SkeletonRow { width: "medium" }
                        }
                    },
                    Some(articles)      => {
                        let visible: Vec<_> = articles.iter()
                            .filter(|a| a.matches(filter()))
                            .collect();
                        rsx! {
                            {visible.iter().cloned().map(|article| {
                                let api = api.clone();
                                let db  = db.clone();
                                let article = article.clone();
                                rsx! {
                                    ArticleRow {
                                        key: "{article.id}",
                                        article: article.clone(),
                                        is_selected: false,
                                        on_click: {
                                            let id = article.id;
                                            move |_| {
                                                navigator().push(Route::Reader { id });
                                            }
                                        },
                                        on_swipe_left: {
                                            let api = api.clone();
                                            let db = db.clone();
                                            let article = article.clone();
                                            move |_| {
                                                let api = api.clone();
                                                let db = db.clone();
                                                let article = article.clone();
                                                spawn(async move {
                                                    toggle_read(api, db, tree, notice, article).await;
                                                });
                                            }
                                        },
                                        on_swipe_right: {
                                            let api = api.clone();
                                            let db = db.clone();
                                            let article = article.clone();
                                            move |_| {
                                                let api = api.clone();
                                                let db = db.clone();
                                                let article = article.clone();
                                                spawn(async move {
                                                    toggle_star(api, db, tree, notice, article).await;
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
