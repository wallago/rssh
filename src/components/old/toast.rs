use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use miniflux_api::MinifluxApi;
use rssh::{
    prelude::{CategoryNode, Notice, NoticeKind, Retry},
    utils::sync_and_load,
};
use rusqlite::Connection;

#[component]
pub fn Toast() -> Element {
    let api = use_context::<Arc<MinifluxApi>>();
    let db = use_context::<Arc<Mutex<Connection>>>();
    let mut notice = use_context::<Signal<Option<Notice>>>();
    let mut tree = use_context::<Signal<Option<Vec<CategoryNode>>>>();

    // auto-dismiss
    use_effect({
        let db = db.clone();
        let api = api.clone();
        move || {
            let Some(current) = notice.read().clone() else {
                return;
            };
            let delay_ms = match current.kind {
                NoticeKind::Sync(_) => return, // sync stays until code clears it
                NoticeKind::Info(_) => 3000,
                NoticeKind::Error { .. } => 6000,
            };
            let r#gen = current.r#gen;
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                // only clear if the notice on screen is still ours
                if notice.peek().as_ref().map(|n| n.r#gen) == Some(r#gen) {
                    notice.set(None);
                }
            });
        }
    });

    let Some(current) = notice.read().clone() else {
        return rsx! {};
    };

    let (label, is_error, show_spinner, retry) = match &current.kind {
        NoticeKind::Sync(s) => (s.clone(), false, true, None),
        NoticeKind::Info(s) => (s.clone(), false, false, None),
        NoticeKind::Error { msg, retry } => (msg.clone(), true, false, retry.clone()),
    };
    let toast_class = if is_error { "toast error" } else { "toast" };

    rsx! {
        div { class: "toast-wrap",
            div { class: "{toast_class}",
                if show_spinner {
                    span { class: "toast-spinner" }
                } else {
                    span { class: "toast-dot" }
                }
                span { class: "toast-label", "{label}" }

                if let Some(r) = retry {
                    button {
                        class: "toast-retry",
                        onclick: move |_| {
                            let db = db.clone();
                            let api =api.clone();
                            notice.set(None);
                            spawn(async move {
                                match r {
                                    Retry::Refresh => {
                                        if let Err(e) = sync_and_load(api, db, notice, tree, false).await {
                                            notice.set(Some(Notice::error(
                                                        format!("Retry failed: {e}"),
                                                        Some(Retry::Refresh),
                                            )));
                                        }
                                    }
                                }
                            });
                        },
                        "Retry"
                    }
                }
            }
        }
    }
}
