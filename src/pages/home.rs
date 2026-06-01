use std::sync::{Arc, Mutex};

use dioxus::{html::article, prelude::*};
use miniflux_api::MinifluxApi;
use reqwest::Client;

use rssh::prelude::*;
use rusqlite::Connection;

use crate::{
    Route,
    components::{
        app::{AppHeader, FilterChips},
        article::ArticleRow,
        category::CategoryNodeView,
        skeleton::SkeletonRow,
    },
    pages::reader::Reader,
};

#[component]
pub fn Home() -> Element {
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();
    let tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();
    let mut notice = use_context::<Signal<Option<Notice>>>();

    let unread_count = use_memo({
        let conn = db.clone();
        move || {
            let conn = conn.lock().ok()?;
            load_articles(&conn).ok()
        }
    });

    // let mut start_y = use_signal(|| 0.0_f64);
    // let mut at_top = use_signal(|| true);
    // let mut pull = use_signal(|| 0.0_f64);
    // let mut refreshing = use_signal(|| false);
    // let threshold = 80.0;

    // use_effect(move || {
    //     tracing::debug!("pull value: {}", pull());
    // });

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
                indicator.style.height = fire ? '40px' : '0px';
                if (fire) { indicator.textConte.send('refresh'); }
                pulling = false; dist = 0;
            });
        "#,
            );

            while let Ok(msg) = channel.recv::<String>().await {
                if msg == "refresh" {
                    let _ = sync_and_load(api.clone(), db.clone(), notice, tree, true).await;
                    let _ = document::eval("documenator').style.height='0px';").await;
                }
            }
        });
    });

    rsx! {
        div { class: "page inbox-page",
            // style: "overscroll-behavior-y: contain;",
            // onpointerdown: move |e| {
            //     start_y.set(e.client_coordinates().y);
            //     spawn(async move {
            //         if let Ok(v) = document::eval("return document.querySelector('.app-main').scrollTop")
            //             .await
            //         {
            //             at_top.set(v.as_f64().unwrap_or(1.0) <= 0.0);
            //         }
            //     });
            // },
            // onpointermove: move |e| {
            //     if at_top() && !refreshing() {
            //         let dy = e.client_coordinates().y - start_y();
            //         if dy > 0.0 { pull.set((dy * 0.5).min(120.0)); } // resistance + cap
            //     }
            // },
            // onpointerup: {
            //     let conn = db.clone();
            //     move |_| {
            //     if pull() >= threshold && !refreshing() {
            //         refreshing.set(true);
            //         let (api, db) = (api.clone(), conn.clone());
            //         spawn(async move {
            //             let _ = sync_and_load(api, db, notice, tree, true).await;
            //             refreshing.set(false);
            //             pull.set(0.0);
            //         });
            //     } else {
            //         pull.set(0.0);
            //     }
            //     }
            // },
            // div {
            //     class: "pull-indicator",
            //     style: "transform: translateY({pull()}px); height: {pull()}px;",
            //     if refreshing() { span { class: "spin", "⟳" } " refreshing…" }
            //     else if pull() >= threshold { "release to refresh" }
            //     else if pull() > 0.0 { "pull to refresh" }
            // }
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
