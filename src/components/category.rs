use dioxus::prelude::*;
use rssh::prelude::*;

use crate::components::{feed::FeedNodeView, gesture::SwipeRow, skeleton::SkeletonRow};

#[component]
pub fn CategoryRow(
    category: Category,
    count: usize,
    expanded: bool,
    on_click: EventHandler<()>,
    on_swipe_left: EventHandler<()>,
) -> Element {
    let class = if expanded {
        "category-row expanded"
    } else {
        "category-row"
    };
    let chevron = if expanded { "▼" } else { "▶" };

    rsx! {
        SwipeRow {
            row_class: "{class}",
            on_click,
            on_swipe_left,
            div { class: "row-content",
                span { class: "chevron", "{chevron}" }
                span { class: "row-title", "{category.label}" }
                div { class: "row-meta",
                    span { class: "unread-count", "{count}" }
                }
            }
        }
    }
}

#[component]
pub fn CategoryNodeView(node: ReadSignal<CategoryNode>) -> Element {
    rsx! {
        CategoryRow {
            category: node().category,
            count: TREE.count_articles(Some(node().category.id), None, Some(FILTER())),
            expanded: node().expanded,
            on_click: move |_| TREE.toggle_category(node()),
            on_swipe_left: move |_| {
                spawn(async move {
                    let ids = TREE.articles(Some(node().category.id), None, Some(Filter::Unread)).iter().map(|a| a.id).collect();
                    SERVER().mark_read(ids).await;
                });
            },
        }
        if node().expanded {
            div { class: "feed-list",
                { match &node().feeds {
                    None => rsx! {
                       div { class: "tree",
                           SkeletonRow { width: "medium" }
                           SkeletonRow { width: "short" }
                       }
                    },
                    Some(feeds) => rsx! {
                        for feed in feeds.iter().cloned() {
                            FeedNodeView { key: "{feed.feed.id}", node: feed }
                        }
                    },
                } }
            }
        }
    }
}
