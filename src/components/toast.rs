use dioxus::prelude::*;

use rssh::prelude::*;

#[component]
pub fn Toast() -> Element {
    // auto-dismiss
    use_effect(move || {
        let Some(current) = NOTICE.read().clone() else {
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
            if NOTICE.peek().as_ref().map(|n| n.r#gen) == Some(r#gen) {
                *NOTICE.write() = None;
            }
        });
    });

    let Some(current) = NOTICE.read().clone() else {
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
                            let r = r.clone();
                            *NOTICE.write() = None;
                            spawn(async move {
                                match r {
                                    Retry::SyncApp => SERVER().sync_app(false).await,
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
