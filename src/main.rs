use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use rusqlite::Connection;

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
const PULL_INDICATOR: Asset = asset!("/assets/pull-indicator.css");

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
    use_future(move || async move {
        if let Err(e) = SERVER().sync_and_load(false).await {
            tracing::error!("startup load failed: {e:?}");
            NOTICE().set(Some(Notice::error(
                format!("Couldn't load data: {e}"),
                Some(Retry::SyncApp),
            )));
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
        document::Stylesheet { href: PULL_INDICATOR }
        Toast {}
        Router::<Route> {}
    }
}
