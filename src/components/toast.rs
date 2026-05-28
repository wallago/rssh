use dioxus::prelude::*;

#[component]
pub fn Toast() -> Element {
    let syncing = use_context::<Signal<(bool, Option<String>)>>();
    if !syncing().0 {
        return rsx! {};
    }

    rsx! {
        div { class: "toast",
            div { class: "toast-dot pulse" }
            {if let Some(desc) = syncing().1 {
                "Syncing: {desc}"
            } else {
                "Syncing…"
            }}
        }
    }
}
