use std::sync::{Arc, Mutex};

use dioxus::{html::article, prelude::*};
use miniflux_api::MinifluxApi;
use rssh::prelude::*;
use rusqlite::Connection;

use crate::Route;

#[component]
pub fn Reader(id: ReadSignal<i64>) -> Element {
    let tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();

    let mut start = use_signal(|| (0.0_f64, 0.0_f64));
    let mut dx = use_signal(|| 0.0_f64);
    let mut horizontal = use_signal(|| false);

    let filter = use_context::<Signal<Filter>>();
    let threshold = 80.0;
    let offset = dx();
    let id_for_swipe = id.clone();

    let article = use_memo(move || iter_articles(tree()?, None, None, None).find(|a| a.id == id()));

    use_effect(move || {
        if matches!(tree(), Some(_)) && article().is_none() {
            navigator().replace(Route::Home {});
        }
    });

    use_effect(move || {
        let mut eval = document::eval(
            r#"
                history.pushState({ rssh_reader: true }, '');
                window.addEventListener('popstate',
                    () => { dioxus.send('back'); },
                    { once: true }
                );
            "#,
        );
        let nav = navigator();
        spawn(async move {
            if eval.recv::<String>().await.is_ok() {
                nav.go_back();
            }
        });
    });

    let Some(article) = article() else {
        return rsx! {
            div { class: "reader-page reader-loading", "Loading…" }
        };
    };

    use_effect({
        let db = db.clone();
        let article = article.clone();
        move || {
            let api = api.clone();
            let db = db.clone();
            let article = article.clone();
            spawn(async move {
                toggle_read(api, db, article).await;
            });
        }
    });

    rsx! {
        div { class: "reader-page",
            style: "transform: translateX({offset}px)",
            onpointerdown: move |e| {
                let p = e.client_coordinates();
                start.set((p.x, p.y));
                horizontal.set(false);
            },
            onpointermove: move |e| {
                let (sx, sy) = start();
                let p = e.client_coordinates();
                let (mx, my) = (p.x - sx, p.y - sy);
                if !horizontal() && mx.abs() > 8.0 && mx.abs() > my.abs() {
                    horizontal.set(true);
                }
                if horizontal() {
                    dx.set(mx);
                }
            },
            onpointerup: {
                let id = id_for_swipe.clone();
                let article = article.clone();
                move |_| {
                    let v = dx();
                    dx.set(0.0);
                    horizontal.set(false);

                    if v.abs() < threshold { return; }

                    let dir = if v < 0.0 { 1 } else { -1 };
                    let article = article.clone();
                    if let Some(next) = sibling_article(db.clone(), article, filter(), dir) {
                        navigator().replace(Route::Reader { id: next });
                    }
                }
            },
            div { class: "reader-header",
                span {
                    class: "reader-source",
                    style: "color: {article.feed.category.color}",
                    "{article.feed.label}"
                }
                a {
                    class: "reader-open",
                    href: "{article.url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "Open ↗"
                }
            }

            div { class: "reader-body",
                h1 { class: "reader-title", "{article.title}" }
                div { class: "reader-meta",
                    span { "{article.timestamp}" }
                    if article.is_starred {
                        span { class: "star", " · ★" }
                    }
                }
                div {
                    class: "reader-content",
                    dangerous_inner_html: "{article.content}",
                }
            }
        }
    }
}
