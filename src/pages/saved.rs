use dioxus::prelude::*;

use crate::components::prelude::*;

#[component]
pub fn SavedPage() -> Element {
    rsx! {
        div { class: "page placeholder",
            div { class: "placeholder-title", "saved/" }
            div { class: "placeholder-text", "your starred articles" }
        }
    }
}
