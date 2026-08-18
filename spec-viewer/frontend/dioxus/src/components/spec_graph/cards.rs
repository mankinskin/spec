use dioxus::prelude::*;
use viewer_api_dioxus::Node3D;

use crate::{
    components::spec_markdown_surface::SpecMarkdownSurface,
    types::SpecGraphNode,
};

use super::view::{
    metric_text,
    node_summary,
    node_title,
    short_id,
    state_color,
};

pub(super) fn render_graph_node_cards(
    nodes: &[Node3D],
    nodes_raw: &[SpecGraphNode],
    mut preview_id: Signal<Option<String>>,
    mut hovered_id: Signal<Option<String>>,
) -> Element {
    rsx! {
        div {
            id: "spec-graph3d-nodes",
            class: "graph-nodes-layer",
            for (index, node) in nodes.iter().enumerate() {
                {
                    let id = node.id.clone();
                    let spec = nodes_raw[index].clone();
                    let title = node_title(&spec);
                    let slug = spec.slug.clone().unwrap_or_else(|| short_id(&id));
                    let summary = spec
                        .summary_markdown
                        .clone()
                        .unwrap_or_else(|| node_summary(&spec));
                    let component = spec.component.clone().unwrap_or_else(|| "uncategorized".to_string());
                    let scope = spec.scope.clone().unwrap_or_else(|| "unspecified scope".to_string());
                    let state = node.state.clone().unwrap_or_else(|| "draft".to_string());
                    let color = state_color(Some(state.as_str()));
                    let child_metric = metric_text(spec.metrics.child_count, "child", "children");
                    let ref_metric = metric_text(spec.metrics.code_ref_count, "ref", "refs");
                    let section_metric = metric_text(spec.metrics.section_count, "section", "sections");
                    let id_click = id.clone();
                    let id_hover_enter = id.clone();
                    let id_hover_leave = id.clone();
                    let is_selected = preview_id.read().as_deref() == Some(id.as_str());
                    let card_class = if is_selected {
                        "graph-node-card node-card-selected"
                    } else {
                        "graph-node-card"
                    };
                    rsx! {
                        div {
                            key: "{id}",
                            "data-node-idx": "{index}",
                            class: "{card_class}",
                            style: "border-left: 4px solid {color}; min-width: 17.5rem; max-width: 19.5rem; padding: 0.8rem 0.95rem; border-radius: 14px; border: 1px solid var(--graph-node-border, rgba(255,255,255,0.08)); background: var(--graph-node-surface, rgba(17,24,39,0.94)); box-shadow: var(--graph-node-shadow, 0 16px 34px rgba(0,0,0,0.26)); color: var(--graph-node-text, #f8fafc); display: flex; flex-direction: column; gap: 0.5rem;",
                            onmouseenter: move |_event: Event<MouseData>| {
                                hovered_id.set(Some(id_hover_enter.clone()));
                            },
                            onmouseleave: move |_event: Event<MouseData>| {
                                if hovered_id.read().as_deref() == Some(id_hover_leave.as_str()) {
                                    hovered_id.set(None);
                                }
                            },
                            onclick: move |event: Event<MouseData>| {
                                event.stop_propagation();
                                preview_id.set(Some(id_click.clone()));
                            },
                            div {
                                style: "display: flex; align-items: flex-start; justify-content: space-between; gap: 0.75rem;",
                                div {
                                    style: "display: flex; flex-direction: column; gap: 0.2rem; min-width: 0;",
                                    div {
                                        style: "font-size: 0.7rem; letter-spacing: 0.08em; text-transform: uppercase; color: var(--graph-node-muted-text, rgba(226,232,240,0.68)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                        "{component}"
                                    }
                                    div {
                                        style: "font-size: 0.72rem; color: var(--graph-node-muted-text, rgba(148,163,184,0.92)); font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                        "{slug}"
                                    }
                                }
                                span {
                                    class: "graph-node-card__state",
                                    style: "color: {color}; border: 1px solid {color}; border-radius: 999px; padding: 0.18rem 0.45rem; font-size: 0.7rem; white-space: nowrap;",
                                    "{state}"
                                }
                            }
                            div {
                                class: "graph-node-card__title",
                                style: "font-size: 1rem; font-weight: 700; line-height: 1.2; color: var(--graph-node-text, #f8fafc); display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;",
                                "{title}"
                            }
                            div {
                                class: "graph-node-card__summary-slot",
                                SpecMarkdownSurface {
                                    content: summary,
                                    class: "graph-node-card__summary-surface".to_string(),
                                }
                            }
                            div {
                                class: "graph-node-card__meta",
                                style: "font-size: 0.72rem; color: var(--graph-node-muted-text, rgba(148,163,184,0.95)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                "scope: {scope}"
                            }
                            div {
                                style: "display: flex; flex-wrap: wrap; gap: 0.35rem;",
                                span {
                                    style: "font-size: 0.72rem; color: #dbeafe; background: rgba(59,130,246,0.14); border: 1px solid rgba(96,165,250,0.22); border-radius: 999px; padding: 0.18rem 0.48rem;",
                                    "{child_metric}"
                                }
                                span {
                                    style: "font-size: 0.72rem; color: #fef3c7; background: rgba(245,158,11,0.13); border: 1px solid rgba(245,158,11,0.22); border-radius: 999px; padding: 0.18rem 0.48rem;",
                                    "{ref_metric}"
                                }
                                span {
                                    style: "font-size: 0.72rem; color: #dcfce7; background: rgba(16,185,129,0.12); border: 1px solid rgba(16,185,129,0.2); border-radius: 999px; padding: 0.18rem 0.48rem;",
                                    "{section_metric}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
