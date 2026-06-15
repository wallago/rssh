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

const STYLES: &[Asset] = &[
    asset!("/assets/style.css"),
    asset!("/assets/article.css"),
    asset!("/assets/bar.css"),
    asset!("/assets/category_feed.css"),
    asset!("/assets/header.css"),
    asset!("/assets/misc.css"),
    asset!("/assets/placeholder.css"),
    asset!("/assets/reader.css"),
    asset!("/assets/search.css"),
    asset!("/assets/skeleton.css"),
    asset!("/assets/swipe.css"),
    asset!("/assets/toast.css"),
    asset!("/assets/pull-indicator.css"),
];

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
    use_future(|| async { SERVER().sync_app(false).await });
    rsx! {
        for style in STYLES {
            document::Stylesheet { href: *style }
        }
        Toast {}
        Router::<Route> {}
    }
}
