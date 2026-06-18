use dioxus::prelude::*;

use rssh::prelude::*;

use crate::Route;

#[component]
pub fn NavTabs() -> Element {
    let route = use_route::<Route>();
    let tabs = [
        (Route::Home {}, "≡", "Home"),
        (Route::Random {}, "⤮", "Random"),
        (Route::Sorted {}, "↧", "Sorted"),
    ];
    rsx! {
        nav { class: "bottom-bar",
            for (r, icon, label) in tabs {
                button {
                    key: "{label}",
                    class: if r == route { "bb-button active" } else { "bb-button" },
                    onclick: move |_| { navigator().push(r.clone()); },
                    span { class: "bb-icon", "{icon}" }
                    span { class: "bb-label", "{label}" }
                }
            }
        }
    }
}

#[component]
pub fn FilterBar() -> Element {
    let current = FILTER();

    let chips = [
        (Filter::Unread, "●", "Unread"),
        (Filter::Starred, "★", "Marked"),
        (Filter::All, "≡", "All"),
    ];

    rsx! {
        nav { class: "top-bar",
            for (f, icon, label) in chips {
                button {
                    key: "{label}",
                    class: if f == current { "bb-button active" } else { "bb-button" },
                    onclick: move |_| *FILTER.write() = f,
                    span { class: "bb-icon", "{icon}" }
                    span { class: "bb-label", "{label}" }
                }
            }
        }
    }
}
