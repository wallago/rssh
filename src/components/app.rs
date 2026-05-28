use dioxus::prelude::*;
use strum::IntoEnumIterator;

use rssh::prelude::Filter;

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
pub fn FilterChips() -> Element {
    let mut filter = use_context::<Signal<Filter>>();
    let current = filter();

    rsx! {
        div { class: "filter-chips",
            for f in Filter::iter() {
                {
                    let cls = if f == current { "filter-chip active" } else { "filter-chip" };
                    rsx! {
                        span {
                            key: "{f}",
                            class: "{cls}",
                            onclick: move |_| filter.set(f),
                            "{f}"
                        }
                    }
                }
            }
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
