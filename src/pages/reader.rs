use dioxus::prelude::*;
use rssh::prelude::*;

use crate::Route;

#[component]
pub fn Reader(id: ReadSignal<i64>) -> Element {
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

    let article = use_memo(move || TREE.article_by_id(id()));

    use_effect(move || {
        if !nav.read().is_empty() {
            return;
        }
        if TREE.read().is_none() {
            return;
        }
        let Some(cur) = TREE.article_by_id(id()) else {
            return;
        };
        let ids = TREE.feed_nav_ids(cur.feed.category.id, cur.feed.id, FILTER(), cur.id);
        if let Some(p) = ids.iter().position(|x| *x == cur.id) {
            pos.set(p);
            nav.set(ids);
        } else {
            navigator().replace(Route::Home {});
        }
    });

    use_effect(move || {
        if TREE.read().is_some() && article().is_none() {
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

    if article().is_none() {
        return rsx! {
            div { class: "reader-page reader-loading", "Loading…" }
        };
    }

    use_effect(move || {
        let ids = nav.read().clone();
        let Some(&cur_id) = ids.get(pos()) else {
            return;
        };
        let Some(a) = TREE.article_by_id(cur_id) else {
            return;
        };
        if !a.is_read {
            spawn(async move {
                SERVER().toggle_read_status(&mut a).await;
            });
        }
    });

    let ids = nav.read().clone();
    if ids.is_empty() {
        return rsx! { div { class: "reader-page reader-loading", "Loading…" } };
    }
    let cur = pos();
    let get = |i: usize| ids.get(i).copied().and_then(|x| TREE.article_by_id(x));
    let prev = if cur > 0 { get(cur - 1) } else { None };
    let current = get(cur);
    let next = get(cur + 1);
    let (has_prev, has_next) = (prev.is_some(), next.is_some());

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
