use dioxus::prelude::*;

#[component]
pub fn FeedsPage() -> Element {
    rsx! {
        div { class: "page placeholder",
            div { class: "placeholder-title", "feeds/" }
            div { class: "placeholder-text", "manage your subscriptions" }
            div { class: "placeholder-hint", "press : to add a feed" }
        }
    }
}
