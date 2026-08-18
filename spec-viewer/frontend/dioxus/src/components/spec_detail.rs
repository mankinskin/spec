//! `SpecDetail` — right-side panel showing spec body, sections, CodeRefs, and health.
//!
//! Tabs: `body` | `sections` | `coderefs` | `health`

use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::{
    code_ref_list::CodeRefList,
    health_panel::HealthPanel,
    spec_markdown_surface::SpecMarkdownSurface,
    state_badge::StateBadge,
};
use crate::{
    api,
    types::{
        HealthResponse,
        SectionResponse,
        SpecFullResponse,
    },
};

// ── CSS class constants ───────────────────────────────────────────────────────

const TAB: &str = "spec-detail__tab";
const TAB_ACTIVE: &str = "spec-detail__tab spec-detail__tab--active";

// ── Tabs ──────────────────────────────────────────────────────────────────────

const TABS: &[(&str, &str)] = &[
    ("body", "Body"),
    ("sections", "Sections"),
    ("coderefs", "CodeRefs"),
    ("health", "Health"),
];

// ── Props ─────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct SpecDetailProps {
    pub spec_id: String,
    pub active_tab: String,
    pub on_tab_change: EventHandler<String>,
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn SpecDetail(props: SpecDetailProps) -> Element {
    // ── Full spec (body + sections list + code_refs) ──────────────────────
    let mut full: Signal<Option<SpecFullResponse>> = use_signal(|| None);
    let mut full_loading: Signal<bool> = use_signal(|| true);
    let mut full_error: Signal<Option<String>> = use_signal(|| None);

    // Shared LRU cache; siblings of the active spec are pre-warmed by
    // SpecListPage (P5.7) so a tab switch generally renders without I/O.
    let cache = use_context::<crate::SpecCache>();

    let spec_id_fetch = props.spec_id.clone();
    use_effect(use_reactive!(|spec_id_fetch| {
        let id = spec_id_fetch.clone();
        // Cache hit: skip the loading flicker and short-circuit.
        if let Some(cached) = cache.get(&id) {
            full.set(Some(cached));
            full_loading.set(false);
            full_error.set(None);
            return;
        }
        full.set(None);
        full_loading.set(true);
        full_error.set(None);
        let cache = cache.clone();
        spawn_local(async move {
            match api::get_spec_full(&id).await {
                Ok(resp) => {
                    cache.insert(id, resp.clone());
                    full.set(Some(resp));
                    full_loading.set(false);
                },
                Err(e) => {
                    full_error.set(Some(e));
                    full_loading.set(false);
                },
            }
        });
    }));

    // ── Expanded section body ─────────────────────────────────────────────
    let mut expanded_section: Signal<Option<String>> = use_signal(|| None);
    let mut section_body: Signal<Option<SectionResponse>> = use_signal(|| None);
    let mut section_loading: Signal<bool> = use_signal(|| false);
    let mut section_error: Signal<Option<String>> = use_signal(|| None);

    // ── Health ────────────────────────────────────────────────────────────
    let mut health: Signal<Option<HealthResponse>> = use_signal(|| None);
    let mut health_loading: Signal<bool> = use_signal(|| false);
    let mut health_error: Signal<Option<String>> = use_signal(|| None);

    let spec_id_health = props.spec_id.clone();
    let active_tab_clone = props.active_tab.clone();
    use_effect(use_reactive!(|spec_id_health, active_tab_clone| {
        if active_tab_clone == "health"
            && health.read().is_none()
            && !*health_loading.read()
        {
            health_loading.set(true);
            health_error.set(None);
            let id = spec_id_health.clone();
            spawn_local(async move {
                match api::health_check(Some(&id)).await {
                    Ok(resp) => {
                        health.set(Some(resp));
                        health_loading.set(false);
                    },
                    Err(e) => {
                        health_error.set(Some(e));
                        health_loading.set(false);
                    },
                }
            });
        }
    }));

    let full_data = full.read();
    let title = full_data
        .as_ref()
        .and_then(|f| f.spec.fields.get("title").and_then(|v| v.as_str()))
        .unwrap_or("Untitled")
        .to_string();
    let state = full_data
        .as_ref()
        .and_then(|f| f.spec.fields.get("state").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();

    rsx! {
        div {
            class: "spec-detail",

            // ── Header ────────────────────────────────────────────────────
            div {
                class: "spec-detail__header",
                span {
                    class: "spec-detail__title",
                    "{title}"
                }
                if !state.is_empty() {
                    StateBadge { state }
                }
            }

            // ── Tab bar ───────────────────────────────────────────────────
            div {
                class: "spec-detail__tabs",
                for (key, label) in TABS.iter() {
                    {
                        let key_str = key.to_string();
                        let label_str = label.to_string();
                        let cls = if props.active_tab == *key { TAB_ACTIVE } else { TAB };
                        rsx! {
                            button {
                                key: "{key_str}",
                                class: "{cls}",
                                onclick: move |_| props.on_tab_change.call(key_str.clone()),
                                "{label_str}"
                            }
                        }
                    }
                }
            }

            // ── Tab content ───────────────────────────────────────────────
            div {
                class: "spec-detail__content",

                if *full_loading.read() {
                    p { class: "spec-detail__loading", "Loading…" }
                } else if let Some(err) = full_error.read().as_deref() {
                    p { class: "spec-detail__error", "{err}" }
                } else if let Some(data) = full_data.as_ref() {
                    {
                        match props.active_tab.as_str() {
                            "body" => rsx! {
                                SpecMarkdownSurface {
                                    content: data.body.clone(),
                                    class: "spec-detail__markdown-surface".to_string(),
                                }
                            },
                            "sections" => {
                                let sections = data.sections.clone();
                                let spec_id_sec = props.spec_id.clone();
                                rsx! {
                                    div {
                                        class: "accordion",
                                        if sections.is_empty() {
                                            p { class: "spec-detail__loading", "No sections." }
                                        } else {
                                            for section_name in sections.iter() {
                                                {
                                                    let sn = section_name.clone();
                                                    let sn2 = sn.clone();
                                                    let id_clone = spec_id_sec.clone();
                                                    let is_expanded = expanded_section.read().as_deref() == Some(&sn);
                                                    rsx! {
                                                        div {
                                                            key: "{sn}",
                                                            class: "accordion-item",
                                                            button {
                                                                class: "accordion-toggle",
                                                                onclick: move |_| {
                                                                    if is_expanded {
                                                                        expanded_section.set(None);
                                                                        section_body.set(None);
                                                                    } else {
                                                                        expanded_section.set(Some(sn.clone()));
                                                                        section_body.set(None);
                                                                        section_loading.set(true);
                                                                        section_error.set(None);
                                                                        let id = id_clone.clone();
                                                                        let name = sn.clone();
                                                                        spawn_local(async move {
                                                                            match api::get_section(&id, &name).await {
                                                                                Ok(resp) => {
                                                                                    section_body.set(Some(resp));
                                                                                    section_loading.set(false);
                                                                                }
                                                                                Err(e) => {
                                                                                    section_error.set(Some(e));
                                                                                    section_loading.set(false);
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                span {
                                                                    class: "accordion-chevron",
                                                                    if is_expanded { "▾" } else { "▸" }
                                                                }
                                                                "{sn2}"
                                                            }
                                                            if is_expanded {
                                                                div {
                                                                    class: "accordion-body",
                                                                    if *section_loading.read() {
                                                                        p { class: "spec-detail__loading", "Loading section…" }
                                                                    } else if let Some(err) = section_error.read().as_deref() {
                                                                        p { class: "spec-detail__error", "{err}" }
                                                                    } else if let Some(sb) = section_body.read().as_ref() {
                                                                        SpecMarkdownSurface {
                                                                            content: sb.content.clone(),
                                                                            class: "spec-detail__markdown-surface".to_string(),
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            "coderefs" => rsx! {
                                CodeRefList { refs: data.spec.code_refs.clone() }
                            },
                            "health" => rsx! {
                                HealthPanel {
                                    health: health.read().clone(),
                                    loading: *health_loading.read(),
                                    error: health_error.read().clone(),
                                }
                            },
                            _ => rsx! { div {} },
                        }
                    }
                }
            }
        }
    }
}
