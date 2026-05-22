use std::{env, rc::Rc, str::FromStr, sync::Arc};

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use reqwest::Url;
use rusqlite::{Connection, Result, params};

mod components;
mod mock;
mod pages;
mod prelude;
mod utils;

use crate::prelude::*;

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
        let conn = Connection::open(dirs::cache_dir().unwrap()).unwrap();
        Rc::new(conn)
    });

    use_context_provider(|| {
        let url = Url::from_str(env!("MINIFLUX_URL")).unwrap();
        let usename = env!("MINIFLUX_USERNAME");
        let passwd = env!("MINIFLUX_PASSWORD");
        Arc::new(MinifluxApi::new(
            &url,
            usename.to_string(),
            passwd.to_string(),
        ))
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
