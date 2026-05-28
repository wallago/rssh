use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use rssh::prelude::*;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::Route;

#[component]
pub fn BottomBar() -> Element {
    let route = use_route::<Route>();
    rsx! {
        nav { class: "bottom-bar",
            { match route {
                Route::InboxPage {}    => rsx! { InboxFilters {} },
                Route::ReaderPage { id }  => rsx! { ArticleActions { id } },
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
                span { class: "bb-icon", "{icon}" }
                span { class: "bb-label", "{label}" }
            }
        }
    }
}

#[component]
fn ArticleActions(id: String) -> Element {
    let tree = use_context::<Signal<Load<Vec<CategoryNode>>>>();
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();

    let Some(article) = find_article(&tree(), &id) else {
        return rsx! {};
    };
    let entry_id = article.id.parse::<i64>().unwrap_or_default();
    let is_starred = article.is_starred;
    let is_read = article.is_read;
    let url = article.url.clone();

    rsx! {
        // // Star
        // button {
        //     class: if is_starred { "bb-button active" } else { "bb-button" },
        //     onclick: {
        //         let api = api.clone();
        //         let db  = db.clone();
        //         let id = id.clone();
        //         move |_| {
        //             let api = api.clone();
        //             let db  = db.clone();
        //             let id = id.clone();
        //             spawn(async move {
        //                 toggle_star(api, db, id.clone()).await;
        //                 set_article_starred(tree, id, is_starred);
        //             });
        //         }
        //     },
        //     span { class: "bb-icon", "★" }
        //     span { class: "bb-label", "Star" }
        // }
        // // Read
        // button {
        //     class: if is_read { "bb-button active" } else { "bb-button" },
        //     onclick: {
        //         let api = api.clone();
        //         let db  = db.clone();
        //         let id = id.clone();
        //         move |_| {
        //             let next = !is_read;
        //             let api = api.clone();
        //             let db  = db.clone();
        //             let id = id.clone();
        //             spawn(async move {
        //                 mark_read(api, db, id.clone(), next).await;
        //                 set_article_read(tree, id, next);
        //             });
        //         }
        //     },
        //     span { class: "bb-icon", "◐" }
        //     span { class: "bb-label", "Read" }
        // }
        a {
            class: "bb-button",
            href: "{url}",
            target: "_blank",
            rel: "noopener noreferrer",
            span { class: "bb-icon", "↗" }
            span { class: "bb-label", "Open" }
        }
    }
}
