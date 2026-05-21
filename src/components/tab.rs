use dioxus::prelude::*;

use crate::Tab;

#[component]
pub fn BottomTabBar(current: Tab, on_change: EventHandler<Tab>) -> Element {
    let tabs = [
        ("▤", "inbox", Tab::Inbox),
        ("≡", "feeds", Tab::Feeds),
        ("★", "saved", Tab::Saved),
        ("⚙", "cfg", Tab::Settings),
    ];

    rsx! {
        nav { class: "tab-bar",
            for (icon, label, tab) in tabs {
                TabButton {
                    key: "{label}",
                    icon: icon.to_string(),
                    label: label.to_string(),
                    is_active: current == tab,
                    on_click: move |_| on_change.call(tab),
                }
            }
        }
    }
}

#[component]
fn TabButton(icon: String, label: String, is_active: bool, on_click: EventHandler<()>) -> Element {
    let class = if is_active {
        "tab-button active"
    } else {
        "tab-button"
    };
    rsx! {
        button {
            class: "{class}",
            onclick: move |_| on_click.call(()),
            span { class: "tab-icon", "{icon}" }
            span { class: "tab-label", "{label}" }
        }
    }
}
