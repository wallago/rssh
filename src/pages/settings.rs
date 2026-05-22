use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    rsx! {
        div { class: "page placeholder",
            div { class: "placeholder-title", "config/" }
            div { class: "placeholder-text", "theme · keybindings · sync" }
        }
    }
}
