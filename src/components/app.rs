use dioxus::prelude::*;
use miniflux_api::models::EntryStatus;
use strum::IntoEnumIterator;

use crate::components::article::Filter;

#[component]
pub fn AppHeader(unread_count: usize, synced_ago: String) -> Element {
    rsx! {
        header { class: "app-header",
            div { class: "header-row",
                span { class: "app-title", "rss" }
                span { class: "sync-dot" }
            }
            div { class: "header-meta",
                "{unread_count} unread · synced {synced_ago}"
            }
        }
    }
}

#[component]
pub fn SearchBar(value: String, on_input: EventHandler<String>) -> Element {
    rsx! {
        div { class: "search-bar",
            span { class: "search-prompt", "▸" }
            input {
                class: "search-input",
                placeholder: "search feeds & articles",
                value: "{value}",
                oninput: move |evt| on_input.call(evt.value()),
            }
        }
    }
}

#[component]
pub fn FilterChips(current: Filter, on_change: EventHandler<Filter>) -> Element {
    rsx! {
        div { class: "filter-chips",
            for filter in Filter::iter() {
                FilterChip {
                    key: "{filter.to_string()}",
                    label: filter.to_string(),
                    is_active: current == filter,
                    on_click: move |_| on_change.call(filter),
                }
            }
            span { class: "filter-add", "＋" }
        }
    }
}

#[component]
fn FilterChip(label: String, is_active: bool, on_click: EventHandler<()>) -> Element {
    let class = if is_active {
        "filter-chip active"
    } else {
        "filter-chip"
    };
    rsx! {
        span {
            class: "{class}",
            onclick: move |_| on_click.call(()),
            "{label}"
        }
    }
}

#[component]
pub fn StatusLine(mode: String, position: String, context: String, last_sync: String) -> Element {
    rsx! {
        div { class: "status-line",
            span { class: "status-mode", "{mode}" }
            span { class: "status-position", "{position}" }
            span { class: "status-context", "{context}" }
            span { class: "status-sync", "↻ {last_sync}" }
        }
    }
}
