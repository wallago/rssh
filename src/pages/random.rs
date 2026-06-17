use crate::{Route, components::article::ArticleRow};
use dioxus::prelude::*;
use rssh::prelude::*;

#[component]
pub fn Random() -> Element {
    // Shuffle order captured once when the page opens (frozen).
    let mut order = use_signal(Vec::<i64>::new);

    use_effect(move || {
        if !order.read().is_empty() {
            return;
        }
        if TREE.read().is_none() {
            return;
        }
        let mut ids: Vec<i64> = TREE
            .articles(None, None, Some(Filter::Unread))
            .iter()
            .map(|a| a.id)
            .collect();
        fastrand::shuffle(&mut ids);
        order.set(ids);
    });

    // Resolve to live articles in the frozen order, dropping any now read.
    let articles = use_memo(move || {
        order
            .read()
            .iter()
            .filter_map(|id| TREE.article_by_id(*id))
            .filter(|a| !a.is_read)
            .collect::<Vec<_>>()
    });
    rsx! {
        div { class: "page",
            div { class: "tree",
                for article in articles() {
                    ArticleRow {
                        key: "{article.id}",
                        article: article.clone(),
                        is_selected: false,
                        on_click: {
                            let id = article.id;
                            let ids = order.read().clone();
                            move |_| {
                                *READER_IDS.write() = Some(ids.clone());
                                navigator().push(Route::Reader { id });
                            }
                        },
                        on_swipe_left: {
                            let a = article.clone();
                            move |_| { let mut a = a.clone(); spawn(async move { SERVER().toggle_read_status(&mut a).await; }); }
                        },
                        on_swipe_right: {
                            let a = article.clone();
                            move |_| { let mut a = a.clone(); spawn(async move { SERVER().toggle_star_status(&mut a).await; }); }
                        },
                    }
                }
            }
        }
    }
}
