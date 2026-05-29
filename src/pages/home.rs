use std::sync::{Arc, Mutex};

use dioxus::{html::article, prelude::*};
use miniflux_api::MinifluxApi;
use reqwest::Client;

use rssh::prelude::*;
use rusqlite::Connection;

use crate::{
    Route,
    components::{
        app::{AppHeader, FilterChips},
        article::ArticleRow,
        category::CategoryNodeView,
        skeleton::SkeletonRow,
    },
    pages::reader::Reader,
};

#[component]
pub fn Home() -> Element {
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();
    let tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();

    let unread_count = use_memo(move || {
        let conn = db.lock().ok()?;
        load_articles(&conn).ok()
    });

    rsx! {
        div { class: "page inbox-page",
            AppHeader {
                unread_count:  iter_articles(tree().unwrap_or_default(), None, None, Some(Filter::Unread)).count(),
                synced_ago: String::new()
            }
            { match tree() {
                None => rsx! {
                   div { class: "tree",
                       SkeletonRow { width: "long" }
                       SkeletonRow { width: "medium" }
                       SkeletonRow { width: "short" }
                       SkeletonRow { width: "medium" }
                       SkeletonRow { width: "long" }
                   }
                },
                Some(categories)    => rsx! {
                    div { class: "tree",
                        for cat in categories {
                            CategoryNodeView { key: "{cat.category.id}", node: cat }
                        }
                    }
                },
            } }
        }
    }
}
