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
fn InboxFilters() -> Element {
    let current = FILTER();

    let chips = [
        (Filter::All, "≡", "All"),
        (Filter::Unread, "●", "Unread"),
        (Filter::Starred, "★", "Starred"),
    ];

    rsx! {
        for (f, icon, label) in chips {
            button {
                key: "{label}",
                class: if f == current { "bb-button active" } else { "bb-button" },
                onclick: move |_| *FILTER.write() = f,
                span { class: "bb-icon", "{icon.to_string()}" }
                span { class: "bb-label", "{label.to_string()}" }
            }
        }
    }
}

#[component]
fn ArticleActions(article: ReadSignal<Article>) -> Element {
    let is_starred = article().is_starred;
    let url = article().url;

    rsx! {
        button {
            class: if is_starred { "bb-button active" } else { "bb-button" },
            onclick: move |_| {
                spawn(async move {
                    SERVER().toggle_star_status(&mut article()).await;
                });
            },
            span { class: "bb-icon", "★" }
            span { class: "bb-label", "Star" }
        }
        a {
            class: "bb-button",
            href: "{url.to_string()}",
            target: "_blank",
            rel: "noopener noreferrer",
            span { class: "bb-icon", "↗" }
            span { class: "bb-label", "Open" }
        }
    }
}
