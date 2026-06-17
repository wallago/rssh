use dioxus::prelude::*;

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
