use dioxus::prelude::*;

/// Swipe-gesture shell for tree rows: drag left for the "Read" action,
/// optionally drag right for star. A real drag suppresses the click.
#[component]
pub fn SwipeRow(
    row_class: String,
    on_click: EventHandler<()>,
    on_swipe_left: EventHandler<()>,
    on_swipe_right: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let mut start = use_signal(|| (0.0_f64, 0.0_f64));
    let mut dx = use_signal(|| 0.0_f64); // current horizontal offset
    let mut horizontal = use_signal(|| false); // gesture locked to horizontal?
    let mut moved = use_signal(|| false); // a real drag happened → suppress the click

    let threshold = 100.0;
    let offset = dx();
    let has_right = on_swipe_right.is_some();

    rsx! {
        div { class: "swipe-row",
            if has_right {
                div { class: "swipe-action swipe-action-left", "★" }
            }
            div { class: "swipe-action swipe-action-right", "Read" }
            div {
                class: "{row_class}",
                style: "transform: translateX({offset}px)",
                onpointerdown: move |e| {
                    let p = e.client_coordinates();
                    start.set((p.x, p.y));
                    horizontal.set(false);
                    moved.set(false);
                },
                onpointermove: move |e| {
                    let (sx, sy) = start();
                    let p = e.client_coordinates();
                    let (mx, my) = (p.x - sx, p.y - sy);
                    // lock the axis once, after a small dead-zone
                    if !horizontal() && mx.abs() > 8.0 && mx.abs() > my.abs() {
                        horizontal.set(true);
                    }
                    if horizontal() {
                        moved.set(true);
                        // without a right action, only allow dragging left
                        dx.set(if has_right { mx } else { mx.min(0.0) });
                    }
                },
                onpointerup: move |_| {
                    let v = dx();
                    if v <= -threshold {
                        on_swipe_left.call(());
                    } else if v >= threshold {
                        if let Some(right) = on_swipe_right {
                            right.call(());
                        }
                    }
                    dx.set(0.0);
                    horizontal.set(false);
                },
                onclick: move |_| {
                    if moved() {
                        return;
                    }
                    on_click.call(())
                },
                {children}
            }
        }
    }
}
