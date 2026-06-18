use crate::{Route, components::article::ArticleRow};
use dioxus::prelude::*;
use rssh::prelude::*;

#[component]
pub fn Sorted() -> Element {
    let articles = use_memo(|| {
        let mut a = TREE.articles(None, None, Some(FILTER()));
        a.sort_by(|x, y| y.timestamp.cmp(&x.timestamp)); // newest first
        a
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
                            let ids: Vec<i64> = articles().iter().map(|a| a.id).collect();
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
