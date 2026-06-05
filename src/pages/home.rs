use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;

use rssh::prelude::*;
use rusqlite::Connection;

use crate::components::{app::AppHeader, category::CategoryNodeView, skeleton::SkeletonRow};

#[component]
pub fn Home() -> Element {
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();
    let tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();
    let notice = use_context::<Signal<Option<Notice>>>();

    use_effect(move || {
        let (api, db) = (api.clone(), db.clone());
        spawn(async move {
            let mut channel = document::eval(
                r#"
            const scroller  = document.querySelector('.app-main');
            const indicator = document.querySelector('.pull-indicator');
            const threshold = 60;
            let startY = 0, dist = 0, pulling = false;

            scroller.addEventListener('touchstart', (e) => {
                if (scroller.scrollTop <= 0) { startY = e.touches[0].clientY; pulling = true; dist = 0; }
            }, { passive: false });

            scroller.addEventListener('touchmove', (e) => {
                if (!pulling) return;
                dist = e.touches[0].clientY - startY;
                if (dist > 0) {
                    e.preventDefault();                       // stops native scroll, keeps events flowing
                    const p = Math.min(dist * 0.6, 120);
                    indicator.style.height = p + 'px';
                    indicator.textContent = p >= threshold ? 'release to refresh' : 'pull to refresh';
                } else { pulling = false; indicator.style.height = '0px'; }
            }, { passive: false });

            scroller.addEventListener('touchend', () => {
                const fire = pulling && dist * 0.6 >= threshold;
                indicator.style.height = '0px';
                indicator.textContent = '';
                if (fire) { dioxus.send('refresh'); }
                pulling = false; dist = 0;
            });
        "#,
            );

            while let Ok(msg) = channel.recv::<String>().await {
                if msg == "refresh" {
                    let _ = sync_and_load(api.clone(), db.clone(), notice, tree, true).await;
                }
            }
        });
    });

    rsx! {
        div { class: "page inbox-page",
            div { class: "pull-indicator" }
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
