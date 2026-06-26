use crate::{
    components::{
        bar::{FilterBar, NavTabs},
        toast::Toast,
    },
    pages::{home::Home, random::Random, reader::Reader, sorted::Sorted},
};
use dioxus::prelude::*;

use rssh::prelude::*;

mod components;
mod pages;

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
    let route = use_route::<Route>();
    let show_filter = !matches!(route, Route::Reader { .. });
    rsx! {
        div { class: "app",
            if show_filter {
                FilterBar {}
            }
            main { class: "app-main", Outlet::<Route> {} }
            NavTabs {}
        }
    }
}

fn main() {
    #[cfg(target_os = "android")]
    {
        let css = [
            include_str!("../assets/tokens.css"),
            include_str!("../assets/style.css"),
            include_str!("../assets/article.css"),
            include_str!("../assets/bar.css"),
            include_str!("../assets/category_feed.css"),
            include_str!("../assets/header.css"),
            include_str!("../assets/misc.css"),
            include_str!("../assets/placeholder.css"),
            include_str!("../assets/reader.css"),
            include_str!("../assets/search.css"),
            include_str!("../assets/skeleton.css"),
            include_str!("../assets/swipe.css"),
            include_str!("../assets/toast.css"),
            include_str!("../assets/pull-indicator.css"),
        ]
        .concat();
        dioxus::LaunchBuilder::mobile()
            .with_cfg(
                dioxus::mobile::Config::new()
                    .with_background_color((13, 13, 13, 255))
                    .with_custom_head(format!("<style>{css}</style>")),
            )
            .launch(App);
    }

    #[cfg(not(target_os = "android"))]
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
        document::Style { {include_str!("../assets/tokens.css")} }
        document::Style { {include_str!("../assets/style.css")} }
        document::Style { {include_str!("../assets/article.css")} }
        document::Style { {include_str!("../assets/bar.css")} }
        document::Style { {include_str!("../assets/category_feed.css")} }
        document::Style { {include_str!("../assets/header.css")} }
        document::Style { {include_str!("../assets/misc.css")} }
        document::Style { {include_str!("../assets/placeholder.css")} }
        document::Style { {include_str!("../assets/reader.css")} }
        document::Style { {include_str!("../assets/search.css")} }
        document::Style { {include_str!("../assets/skeleton.css")} }
        document::Style { {include_str!("../assets/swipe.css")} }
        document::Style { {include_str!("../assets/toast.css")} }
        document::Style { {include_str!("../assets/pull-indicator.css")} }
        Toast {}
        Router::<Route> {}
    }
}
