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
    pages::{home::Home, reader::Reader},
};

use rssh::prelude::*;

mod components;
mod pages;

const STYLE: Asset = asset!("/assets/style.css");
const ARTICLE: Asset = asset!("/assets/article.css");
const BAR: Asset = asset!("/assets/bar.css");
const CATEGORY_FEED: Asset = asset!("/assets/category_feed.css");
const HEADER: Asset = asset!("/assets/header.css");
const MISC: Asset = asset!("/assets/misc.css");
const PLACEHOLDER: Asset = asset!("/assets/placeholder.css");
const READER: Asset = asset!("/assets/reader.css");
const SEARCH: Asset = asset!("/assets/search.css");
const SKELETON: Asset = asset!("/assets/skeleton.css");
const SWIPE: Asset = asset!("/assets/swipe.css");
const TOAST: Asset = asset!("/assets/toast.css");

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Home {},
    #[route("/article/:id")]
    Reader { id: i64 },
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
    use_context_provider(|| {
        let db = get_db_conn().expect("failed to get DB connection");
        db
    });

    // Load Miniflux Connection
    use_context_provider(|| {
        let api = get_api_conn().expect("failed to get Miniflux connection");
        api
    });

    let mut tree: Signal<Option<Vec<CategoryNode>>> = use_signal(|| None);
    use_context_provider(|| tree);

    let mut notice: Signal<Option<Notice>> = use_signal(|| None);
    use_context_provider(|| notice);

    let filter: Signal<Filter> = use_signal(|| Filter::Unread);
    use_context_provider(|| filter);

    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();

    use_future(move || {
        let db = db.clone();
        let api = api.clone();
        async move {
            if let Err(e) = sync_and_load(api, db, notice, tree).await {
                tracing::error!("startup load failed: {e:?}");
                notice.set(Some(Notice::error(
                    format!("Couldn't load data: {e}"),
                    Some(Retry::Refresh),
                )));
            }
        }
    });

    rsx! {
        document::Stylesheet { href: STYLE }
        document::Stylesheet { href: ARTICLE }
        document::Stylesheet { href: BAR }
        document::Stylesheet { href: CATEGORY_FEED }
        document::Stylesheet { href: HEADER }
        document::Stylesheet { href: MISC }
        document::Stylesheet { href: PLACEHOLDER }
        document::Stylesheet { href: READER }
        document::Stylesheet { href: SEARCH }
        document::Stylesheet { href: SKELETON }
        document::Stylesheet { href: SWIPE }
        document::Stylesheet { href: TOAST }
        Toast {}
        Router::<Route> {}
    }
}
