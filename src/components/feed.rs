use dioxus::prelude::*;

use rssh::prelude::*;

use crate::{
    Route,
    components::{article::ArticleRow, gesture::SwipeRow, skeleton::SkeletonRow},
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

    rsx! {
        SwipeRow {
            row_class: "{class}",
            on_click,
            on_swipe_left,
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

#[component]
pub fn FeedNodeView(node: ReadSignal<FeedNode>) -> Element {
    rsx! {
        FeedRow {
            feed: node().feed,
            count: TREE.count_articles(Some(node().feed.category.id), Some(node().feed.id), Some(FILTER())),
            expanded: node().expanded,
            on_click: move |_| TREE.toggle_feed(node()),
            on_swipe_left: move |_| {
                spawn(async move {
                    let ids = TREE.articles(Some(node().feed.category.id), Some(node().feed.id), Some(Filter::Unread)).iter().map(|a| a.id).collect();
                    SERVER().mark_read(ids).await;
                });
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
                    Some(articles) => {
                        let visible: Vec<Article> = articles
                            .iter()
                            .filter(|a| a.matches(FILTER()))
                            .cloned()
                            .collect();
                        rsx! {
                            for article in visible {
                                ArticleRow {
                                    key: "{article.id}",
                                    article: article.clone(),
                                    is_selected: false,
                                    on_click: {
                                        let id = article.id;
                                        move |_| {
                                            *READER_IDS.write() = None;
                                            navigator().push(Route::Reader { id });
                                        }
                                    },
                                    on_swipe_left: {
                                        let article = article.clone();
                                        move |_| {
                                            let mut a = article.clone();
                                            spawn(async move {
                                                SERVER().toggle_read_status(&mut a).await;
                                            });
                                        }
                                    },
                                    on_swipe_right: {
                                        let article = article.clone();
                                        move |_| {
                                            let mut a = article.clone();
                                            spawn(async move {
                                                SERVER().toggle_star_status(&mut a).await;
                                            });
                                        }
                                    },
                                }
                            }
                        }
                    },
                } }
            }
        }
    }
}
