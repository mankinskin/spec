//! `SpecCard` — compact row in the spec list / tree sidebar.

use super::state_badge::StateBadge;
use crate::types::SpecSummary;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SpecCardProps {
    pub spec: SpecSummary,
    pub selected: bool,
    pub on_click: EventHandler<String>,
    /// Indentation level for tree display (0 = root).
    #[props(default = 0)]
    pub indent: usize,
}

#[component]
pub fn SpecCard(props: SpecCardProps) -> Element {
    let card_class = if props.selected {
        "spec-card spec-card--selected"
    } else {
        "spec-card"
    };
    let title = props.spec.title.as_deref().unwrap_or("Untitled");
    let slug = props.spec.slug.as_deref().unwrap_or("-");
    let state = props.spec.state.clone().unwrap_or_default();
    let id = props.spec.id.clone();
    let indent_style = if props.indent > 0 {
        format!("padding-left: {}px", props.indent * 16 + 12)
    } else {
        String::new()
    };

    rsx! {
        button {
            class: "{card_class}",
            style: "{indent_style}",
            onclick: move |_| props.on_click.call(id.clone()),
            span {
                class: "spec-card__title",
                "{title}"
            }
            div {
                class: "spec-card__meta",
                span {
                    class: "spec-card__slug",
                    "{slug}"
                }
                if !state.is_empty() {
                    StateBadge { state }
                }
            }
        }
    }
}
