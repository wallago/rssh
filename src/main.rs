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
    InboxPage {},
    #[route("/article/:id")]
    ReaderPage { id: String },
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
    use_context_provider(|| {
        let app_name = env!("CARGO_PKG_NAME");
        let dir = dirs::data_dir().unwrap_or_else(std::env::temp_dir);
        std::fs::create_dir_all(&dir).ok();
        let conn =
            Connection::open(dir.join(format!("{app_name}.db"))).expect("failed to open rssh.db");
        init_schema(&conn).expect("init schema");
        Arc::new(Mutex::new(conn))
    });

    use_context_provider(|| {
        let url =
            Url::from_str(env!("MINIFLUX_URL")).expect("failed to parse MINIFLUX_URL into URL");
        let usename = env!("MINIFLUX_USERNAME");
        let passwd = env!("MINIFLUX_PASSWORD");
        Arc::new(MinifluxApi::new(
            &url,
            usename.to_string(),
            passwd.to_string(),
        ))
    });

    let mut tree: Signal<Load<Vec<CategoryNode>>> = use_signal(|| Load::Idle);
    let syncing: Signal<bool> = use_signal(|| false);
    let filter: Signal<Filter> = use_signal(|| Filter::Unread);
    use_context_provider(|| filter);
    use_context_provider(|| syncing);
    use_context_provider(|| tree);

    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();

    use_future(move || {
        let api = api.clone();
        let db = db.clone();
        let mut syncing = syncing;
        async move {
            syncing.set(true);
            // scope the lock so the guard is dropped BEFORE the await
            let empty = { is_empty(&db.lock().unwrap()).unwrap_or(true) };
            if empty {
                let _ = initial_sync(api, db.clone()).await;
            }
            // now build the tree from SQLite (sync reads), not the network
            let conn = db.lock().unwrap();
            let cats = load_categories(&conn).expect("fail to laod categories from DB");
            let feeds = load_feeds(&conn).expect("fail to laod feeds from DB");
            let articles = load_articles(&conn).expect("fail to laod articles from DB");
            drop(conn);
            tree.set(Load::Ready(build_tree(cats, feeds, articles)));
            syncing.set(false);
        }
    });

    rsx! {
        document::Stylesheet { href: STYLE }
        Toast {}
        Router::<Route> {}
    }
}
