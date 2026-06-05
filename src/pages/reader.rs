use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use rssh::prelude::*;
use rusqlite::Connection;

use crate::Route;

#[component]
pub fn Reader(id: ReadSignal<i64>) -> Element {
    let tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();
    let notice = use_context::<Signal<Option<Notice>>>();
    let filter = use_context::<Signal<Filter>>();

    let mut nav = use_signal(Vec::<i64>::new);
    let mut pos = use_signal(|| 0usize);

    // pointer / animation state
    let mut start = use_signal(|| (0.0_f64, 0.0_f64));
    let mut dx = use_signal(|| 0.0_f64);
    let mut horizontal = use_signal(|| false);
    let mut slide = use_signal(|| 0i32); // -1 prev, +1 next, 0 idle (animating when != 0)
    let mut snapping = use_signal(|| false); // animating back to center after a short drag
    let threshold = 80.0;
    let offset = dx();

    let article = use_memo(move || iter_articles(tree()?, None, None, None).find(|a| a.id == id()));

    use_effect(move || {
        if !nav.read().is_empty() {
            return;
        }
        let Some(cats) = tree() else { return };
        let Some(cur) = article_by_id(&cats, id()) else {
            return;
        };
        let ids = feed_nav_ids(&cats, cur.feed.category.id, cur.feed.id, filter(), cur.id);
        if let Some(p) = ids.iter().position(|x| *x == cur.id) {
            pos.set(p);
            nav.set(ids);
        } else {
            navigator().replace(Route::Home {});
        }
    });

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

    use_effect(move || {
        let ids = nav.read().clone();
        let Some(&cur_id) = ids.get(pos()) else {
            return;
        };
        let Some(cats) = tree() else { return };
        let Some(a) = article_by_id(&cats, cur_id) else {
            return;
        };
        if !a.is_read {
            let (api, db) = (api.clone(), db.clone());
            spawn(async move {
                toggle_read(api, db, tree, notice, a).await;
            });
        }
    });

    let ids = nav.read().clone();
    if ids.is_empty() {
        return rsx! { div { class: "reader-page reader-loading", "Loading…" } };
    }
    let cats = tree().unwrap_or_default();
    let cur = pos();
    let get = |i: usize| ids.get(i).copied().and_then(|x| article_by_id(&cats, x));
    let prev = if cur > 0 { get(cur - 1) } else { None };
    let current = get(cur);
    let next = get(cur + 1);
    let (has_prev, has_next) = (prev.is_some(), next.is_some());

    let transform = match slide() {
        1 => "translateX(-200vw)".to_string(), // sliding to next
        -1 => "translateX(0vw)".to_string(),   // sliding to prev
        _ => format!("translateX(calc(-100vw + {offset}px))"),
    };
    let settling = slide() != 0 || snapping();
    let track_class = if settling {
        "reader-track settling"
    } else {
        "reader-track"
    };

    rsx! {
        div { class: "reader-viewport",
        div {
            class: "{track_class}",
            style: "transform: {transform}",
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
                    // block dragging past either end
                    let clamped = if (mx > 0.0 && !has_prev) || (mx < 0.0 && !has_next) {
                        0.0
                    } else {
                        mx
                    };
                    dx.set(clamped);
                }
            },
            onpointerup: move |_| {
                if settling { return; }
                let v = dx();
                horizontal.set(false);
                if v <= -threshold && has_next {
                    slide.set(1);
                } else if v >= threshold && has_prev {
                    slide.set(-1);
                } else if v != 0.0 {
                    snapping.set(true);
                    dx.set(0.0);
                }
            },
            ontransitionend: move |_| {
                let s = slide();
                if s != 0 {
                    let p = pos() as i32 + s;
                    if p >= 0 {
                        pos.set(p as usize);
                    }
                    slide.set(0);
                    dx.set(0.0);
                } else if snapping() {
                    snapping.set(false);
                }
            },
            div { class: "reader-pane", if let Some(a) = prev { {article_view(a)} } }
            div { class: "reader-pane", if let Some(a) = current { {article_view(a)} } }
            div { class: "reader-pane", if let Some(a) = next { {article_view(a)} } }
        }
        }
    }
}

fn article_view(article: Article) -> Element {
    rsx! {
        div { class: "reader-page",
            div { class: "reader-header",
                span { class: "reader-source", style: "color: {article.feed.category.color}", "{article.feed.label}" }
                a { class: "reader-open", href: "{article.url}", target: "_blank", rel: "noopener noreferrer", "Open ↗" }
            }
            div { class: "reader-body",
                h1 { class: "reader-title", "{article.title}" }
                div { class: "reader-meta",
                    span { "{article.timestamp}" }
                    if article.is_starred { span { class: "star", " · ★" } }
                }
                div { class: "reader-content", dangerous_inner_html: "{article.content}" }
            }
        }
    }
}
