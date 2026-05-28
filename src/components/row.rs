use dioxus::prelude::*;
use miniflux_api::models::{Entry, EntryStatus};
use strum::EnumIter;

use rssh::prelude::*;

#[component]
pub fn CategoryRow(
    category: Category,
    count: Option<i64>,
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

    let mut start = use_signal(|| (0.0_f64, 0.0_f64));
    let mut dx = use_signal(|| 0.0_f64);
    let mut horizontal = use_signal(|| false);
    let mut moved = use_signal(|| false);
    let threshold = 100.0;
    let offset = dx();

    rsx! {
        div {
            class: "{class}",
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
                    // clamp to left-only swipe (no right action on category rows)
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
                span { class: "row-title", "{category.label}" }
                div { class: "row-meta",
                    if let Some(count) = count {
                        span { class: "unread-count", "{count}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn FeedRow(
    feed: Feed,
    count: Option<i64>,
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
                    // clamp to left-only swipe (no right action on category rows)
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
                    if let Some(count) = count {
                        span { class: "unread-count", "{count}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ArticleRow(
    article: Article,
    is_selected: bool,
    on_click: EventHandler<()>,
    on_swipe_left: EventHandler<()>,
    on_swipe_right: EventHandler<()>,
) -> Element {
    let mut start = use_signal(|| (0.0_f64, 0.0_f64));
    let mut dx = use_signal(|| 0.0_f64); // current horizontal offset
    let mut horizontal = use_signal(|| false); // gesture locked to horizontal?
    let mut moved = use_signal(|| false); // a real drag happened → suppress the click

    let threshold = 100.0;
    let offset = dx();

    let class = match (is_selected, article.is_read) {
        (true, true) => "article-row selected read",
        (true, false) => "article-row selected",
        (false, true) => "article-row read",
        (false, false) => "article-row",
    };

    let row_class = if horizontal() {
        format!("{class} dragging")
    } else {
        class.to_string()
    };

    rsx! {
        div { class: "swipe-row",
            div { class: "swipe-action swipe-action-left", "★" }
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
                    // lock the axis once, after a small dead-zone
                    if !horizontal() && mx.abs() > 8.0 && mx.abs() > my.abs() {
                        horizontal.set(true);
                    }
                    if horizontal() {
                        moved.set(true);
                        dx.set(mx);
                    }
                },
                onpointerup: move |_| {
                    let v = dx();
                    if v <= -threshold { on_swipe_left.call(()); }
                    else if v >= threshold { on_swipe_right.call(()); }
                    dx.set(0.0);
                    horizontal.set(false);
                },
                onclick: move |_| {
                    if moved() { return; }
                    on_click.call(())
                },
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
}
