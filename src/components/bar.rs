use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use rssh::prelude::*;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::Route;

#[component]
pub fn BottomBar() -> Element {
    let route = use_route::<Route>();
    let tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();
    rsx! {
        nav { class: "bottom-bar",
            { match route {
                Route::Home {}    => rsx! { InboxFilters {} },
                Route::Reader { id }  => {
                    if let Some(article) = iter_articles(tree().unwrap_or(Vec::new()), None, None, None).find(|a| a.id == id) {
                        rsx! { ArticleActions { article } }
                    } else {
                        rsx! {}
                    }
                },
            } }
        }
    }
}

#[component]
fn InboxFilters() -> Element {
    let mut filter = use_context::<Signal<Filter>>();
    let current = filter();

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
                onclick: move |_| filter.set(f),
                span { class: "bb-icon", "{icon.to_string()}" }
                span { class: "bb-label", "{label.to_string()}" }
            }
        }
    }
}

#[component]
fn ArticleActions(article: ReadSignal<Article>) -> Element {
    let tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();
    let notice = use_context::<Signal<Option<Notice>>>();

    let is_starred = article().is_starred;
    let url = article().url;

    rsx! {
        button {
            class: if is_starred { "bb-button active" } else { "bb-button" },
            onclick: {
                move |_| {
                    let api = api.clone();
                    let db  = db.clone();
                    spawn(async move {
                        toggle_star(api, db, tree, notice ,article()).await;
                    });
                }
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
