use dioxus::prelude::*;

#[component]
pub fn SkeletonRow(width: Option<&'static str>) -> Element {
    let cls = width.unwrap_or("medium");
    rsx! {
        div { class: "skeleton-row pulse",
            div { class: "skeleton-source-bar" }
            div { class: "skeleton-bar {cls}" }
        }
    }
}
