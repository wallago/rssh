use std::{env, str::FromStr, sync::Arc};

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use reqwest::Url;

mod components;
mod mock;
mod pages;

use components::prelude::*;
use pages::prelude::*;

const STYLE: Asset = asset!("/assets/style.css");

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tab {
    Inbox,
    Feeds,
    Saved,
    Settings,
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut tab = use_signal(|| Tab::Inbox);
    use_context_provider(|| {
        let url = Url::from_str(&env::var("URL").unwrap()).unwrap();
        let usename = env::var("USERNAME").unwrap();
        let passwd = env::var("PASSWORD").unwrap();
        Arc::new(MinifluxApi::new(&url, usename, passwd))
    });

    rsx! {
        document::Stylesheet { href: STYLE }
        div { class: "app",
            main { class: "app-main",
                match tab() {
                    Tab::Inbox => rsx! { InboxPage {} },
                    Tab::Feeds => rsx! { FeedsPage {} },
                    Tab::Saved => rsx! { SavedPage {} },
                    Tab::Settings => rsx! { SettingsPage {} },
                }
            }
            BottomTabBar {
                current: tab(),
                on_change: move |t| tab.set(t),
            }
        }
    }
}
