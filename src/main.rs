use crate::{
    components::{bar::NavTabs, toast::Toast},
    pages::{home::Home, random::Random, reader::Reader, sorted::Sorted},
};
use dioxus::prelude::*;

use rssh::prelude::*;

mod components;
mod pages;

const STYLES: &[Asset] = &[
    asset!("/assets/tokens.css"),
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
    #[route("/random")]
    Random {},
    #[route("/sorted")]
    Sorted {},
    #[route("/article/:id")]
    Reader { id: i64 },
}

#[component]
fn AppLayout() -> Element {
    rsx! {
        div { class: "app",
            main { class: "app-main", Outlet::<Route> {} }
            NavTabs {}
        }
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_effect(|| {
        if let Err(e) = theme_system_bars() {
            // {e:?} on an anyhow::Error includes the full cause chain,
            // i.e. the Java exception message from the failing JNI call.
            eprintln!("THEME_BARS_ERROR: {e:?}");
        }
    });
    use_future(|| async { SERVER().sync_app(false).await });
    rsx! {
        for style in STYLES {
            document::Stylesheet { href: *style }
        }
        Toast {}
        Router::<Route> {}
    }
}
