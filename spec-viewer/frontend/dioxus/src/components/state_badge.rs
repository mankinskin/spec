//! `StateBadge` — coloured pill that displays a spec state label.

use dioxus::prelude::*;

#[component]
pub fn StateBadge(state: String) -> Element {
    let modifier = match state.as_str() {
        "draft" => "state-badge state-badge--draft",
        "ready" => "state-badge state-badge--ready",
        "reviewed" => "state-badge state-badge--reviewed",
        "approved" => "state-badge state-badge--approved",
        "archived" => "state-badge state-badge--archived",
        _ => "state-badge",
    };
    rsx! {
        span {
            class: "{modifier}",
            "{state}"
        }
    }
}
