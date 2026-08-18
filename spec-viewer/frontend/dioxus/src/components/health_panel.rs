//! `HealthPanel` — displays health-check issues for a spec.

use crate::types::HealthResponse;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct HealthPanelProps {
    pub health: Option<HealthResponse>,
    pub loading: bool,
    pub error: Option<String>,
}

#[component]
pub fn HealthPanel(props: HealthPanelProps) -> Element {
    if props.loading {
        return rsx! {
            p { style: "color: #6b7280; font-size: 13px; padding: 12px 0;", "Checking health…" }
        };
    }

    if let Some(err) = &props.error {
        return rsx! {
            p { style: "color: #f87171; font-size: 13px; padding: 12px 0;", "{err}" }
        };
    }

    let Some(h) = &props.health else {
        return rsx! { div {} };
    };

    let status_color = if h.issues_count == 0 {
        "#22c55e"
    } else {
        "#f59e0b"
    };
    let status_label = if h.issues_count == 0 {
        "Healthy"
    } else {
        "Issues found"
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 10px;",
            div {
                style: "display: flex; align-items: center; gap: 8px;",
                span {
                    style: "font-size: 13px; font-weight: 600; color: {status_color};",
                    "{status_label}"
                }
                span {
                    style: "font-size: 12px; color: #6b7280;",
                    "({h.specs_checked} spec(s) checked, {h.issues_count} issue(s))"
                }
            }
            if h.issues_count > 0 {
                ul {
                    style: "list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px;",
                    for issue in h.issues.iter() {
                        li {
                            style: "
                                background: #1f2937;
                                border-left: 3px solid #f59e0b;
                                border-radius: 4px;
                                padding: 6px 10px;
                                font-size: 12px;
                                color: #fbbf24;
                            ",
                            "{issue}"
                        }
                    }
                }
            }
        }
    }
}
