use dioxus::prelude::*;

#[component]
pub fn Toast() -> Element {
    let syncing = use_context::<Signal<bool>>();
    if !syncing() {
        return rsx! {};
    }

    rsx! {
        div { class: "toast",
            div { class: "toast-dot pulse" }
            "Syncing…"
        }
    }
}
