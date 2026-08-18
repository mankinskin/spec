use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;

use super::view::state_color;
use crate::components::spec_markdown_surface::SpecMarkdownSurface;

#[component]
pub(super) fn SpecPreviewSidebar(
    spec_id: String,
    on_close: EventHandler<()>,
    on_view_details: EventHandler<String>,
) -> Element {
    let mut full: Signal<Option<crate::types::SpecFullResponse>> =
        use_signal(|| None);
    let mut load_err: Signal<Option<String>> = use_signal(|| None);

    let spec_id_load = spec_id.clone();
    use_effect(use_reactive!(|spec_id_load| {
        full.set(None);
        load_err.set(None);
        let id = spec_id_load.clone();
        spawn_local(async move {
            match api::get_spec_full(&id).await {
                Ok(response) => full.set(Some(response)),
                Err(error) => load_err.set(Some(error)),
            }
        });
    }));

    let spec_id_open = spec_id.clone();

    rsx! {
        aside {
            class: "spec-preview",
            "data-testid": "spec-preview",
            "data-graph-passthrough": "false",
            onclick: move |event: Event<MouseData>| event.stop_propagation(),
            onmousedown: move |event: Event<MouseData>| event.stop_propagation(),
            onwheel: move |event: Event<WheelData>| event.stop_propagation(),

            header { class: "spec-preview__header",
                h3 { class: "spec-preview__title", "{preview_title(&spec_id, &full.read())}" }
                button {
                    class: "spec-preview__close",
                    "data-testid": "spec-preview-close",
                    aria_label: "Close preview",
                    onclick: move |_| on_close.call(()),
                    "\u{2715}"
                }
            }
            div { class: "spec-preview__body",
                if let Some(error) = load_err.read().clone() {
                    p { class: "spec-preview__error", "Failed to load: {error}" }
                } else if let Some(full_spec) = full.read().clone() {
                    {
                        let state = full_spec.spec.fields.get("state")
                            .and_then(|value| value.as_str())
                            .unwrap_or("draft")
                            .to_string();
                        let color = state_color(Some(state.as_str()));
                        rsx! {
                            div { class: "spec-preview__meta",
                                span {
                                    class: "spec-preview__state",
                                    style: "color: {color}; border-color: {color};",
                                    "{state}"
                                }
                            }
                            SpecMarkdownSurface {
                                content: full_spec.body.clone(),
                                class: "spec-preview__markdown-surface".to_string(),
                            }
                        }
                    }
                } else {
                    p { class: "spec-preview__loading", "Loading\u{2026}" }
                }
            }
            footer { class: "spec-preview__footer",
                button {
                    class: "spec-preview__details",
                    "data-testid": "spec-preview-details",
                    onclick: move |_| on_view_details.call(spec_id_open.clone()),
                    "View details \u{2192}"
                }
            }
        }
    }
}

fn preview_title(
    spec_id: &str,
    full: &Option<crate::types::SpecFullResponse>,
) -> String {
    full.as_ref()
        .and_then(|response| {
            response
                .spec
                .fields
                .get("title")
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| spec_id.to_string())
}
