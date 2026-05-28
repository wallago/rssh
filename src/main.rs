use std::{
    env,
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex},
};

use dioxus::prelude::*;
use dioxus_router::RouterConfig;
use dioxus_web::WebHistory;
use miniflux_api::MinifluxApi;
use reqwest::Url;
use rusqlite::{Connection, Result, params};

use crate::{
    components::{bar::BottomBar, toast::Toast},
    pages::{inbox::InboxPage, reader::ReaderPage},
};

use rssh::prelude::*;

mod components;
mod pages;

const STYLE: Asset = asset!("/assets/style.css");

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Home {},
    #[route("/article/:id")]
    Reader { id: String },
}

#[component]
fn AppLayout() -> Element {
    rsx! {
        div { class: "app",
            main { class: "app-main", Outlet::<Route> {} }
            BottomBar {}
        }
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Load local DB Connection
    use_context_provider(|| get_db_conn());

    // Load Miniflux Connection
    use_context_provider(|| get_api_conn());

    let mut tree: Signal<Option<Vec<CategoryNode>>> = use_signal(|| None);
    use_context_provider(|| tree);

    let syncing: Signal<(bool, Option<String>)> = use_signal(|| (false, None));
    use_context_provider(|| syncing);

    let filter: Signal<Filter> = use_signal(|| Filter::Unread);
    use_context_provider(|| filter);

    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();

    use_future(move || {
        async move {
            syncing.set((true, None));
            // TODO
            // handle error by a toast
            let Ok(conn) = db.get_mut() else {
                return;
            };
            // TODO
            // handle error by a toast
            let Ok(empty) = is_empty(conn) else {
                return;
            };
            if empty {
                syncing.set((true, Some("initial sync".to_string())));
                // TODO
                // handle error by a toast
                initial_sync(api, db).await;
            }

            syncing.set((true, Some("load categories".to_string())));
            // TODO
            // handle error by a toast
            let Ok(cats) = load_categories(&conn) else {
                return;
            };

            syncing.set((true, Some("load feeds".to_string())));
            // TODO
            // handle error by a toast
            let Ok(feeds) = load_feeds(&conn) else {
                return;
            };

            syncing.set((true, Some("load articles".to_string())));
            // TODO
            // handle error by a toast
            let Ok(articles) = load_articles(&conn) else {
                return;
            };
            drop(conn);

            syncing.set((true, Some("build architecture".to_string())));
            let builded_tree = build_tree(cats, feeds, articles);
            tree.set(Some(builded_tree));

            syncing.set((false, None));
        }
    });

    rsx! {
        document::Stylesheet { href: STYLE }
        Toast {}
        Router::<Route> {}
    }
}
