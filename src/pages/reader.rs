use dioxus::prelude::*;
use rssh::prelude::*;

use crate::Route;

#[component]
pub fn ReaderPage(id: String) -> Element {
    let tree = use_context::<Signal<Load<Vec<CategoryNode>>>>();

    let article = use_memo({
        let id = id.clone();
        move || find_article(&tree(), &id)
    });

    use_effect(move || {
        if matches!(tree(), Load::Ready(_)) && article().is_none() {
            navigator().replace(Route::InboxPage {});
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

    rsx! {
        div { class: "reader-page",
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
