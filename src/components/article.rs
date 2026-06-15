use dioxus::prelude::*;

use rssh::prelude::*;

use crate::components::gesture::SwipeRow;

#[component]
pub fn ArticleRow(
    article: Article,
    is_selected: bool,
    on_click: EventHandler<()>,
    on_swipe_left: EventHandler<()>,
    on_swipe_right: EventHandler<()>,
) -> Element {
    let class = match (is_selected, article.is_read) {
        (true, true) => "article-row selected read",
        (true, false) => "article-row selected",
        (false, true) => "article-row read",
        (false, false) => "article-row",
    };

    rsx! {
        SwipeRow {
            row_class: "{class}",
            on_click,
            on_swipe_left,
            on_swipe_right: move |_| on_swipe_right.call(()),
            div {
                class: "source-bar",
                style: "background-color: {article.feed.category.color}",
            }
            div { class: "article-content",
                div { class: "article-meta",
                    span {
                        class: "source-name",
                        style: "color: {article.feed.category.color}",
                        "{article.feed.label}"
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
