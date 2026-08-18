use std::collections::BTreeMap;

use dioxus::prelude::*;
use dioxus_router::Navigator;
use viewer_api_dioxus::{
    Card,
    CardGrid,
    CardSection,
    HamburgerIcon,
    PageHeader,
    Sidebar,
};

use crate::{
    components::spec_tree::SpecTree,
    types::SpecSummary,
};

use super::{
    super::Route,
    helpers::{
        close_or_toggle_sidebar,
        toggle_sidebar,
    },
};

pub(super) fn render_spec_list_header(
    sidebar_button_active: bool,
    sidebar_button_label: &'static str,
    sidebar_collapsed: Signal<bool>,
    mobile_sidebar_open: Signal<bool>,
    mut filter_panel_open: Signal<bool>,
    mut show_theme_settings: Signal<bool>,
    filter: Signal<String>,
    state_filter: Signal<String>,
    on_home: EventHandler<()>,
) -> Element {
    rsx! {
        PageHeader {
            lead: Some(rsx! {
                button {
                    class: if sidebar_button_active { "btn btn-icon btn-active" } else { "btn btn-icon" },
                    aria_label: sidebar_button_label,
                    title: sidebar_button_label,
                    onclick: move |_| toggle_sidebar(sidebar_collapsed, mobile_sidebar_open),
                    HamburgerIcon {}
                }
            }),
            icon: Some(rsx! { "📐" }),
            title: Some("Spec Viewer".to_string()),
            right_prefix: Some(rsx! {
                Link {
                    to: Route::SpecGraphPage {},
                    class: "btn-nav-link",
                    "🌐 Graph"
                }
            }),
            on_home: Some(on_home),
            on_filter_toggle: Some(EventHandler::new(move |_| {
                let next = !*filter_panel_open.read();
                filter_panel_open.set(next);
            })),
            on_theme_toggle: Some(EventHandler::new(move |_| {
                let next = !*show_theme_settings.read();
                show_theme_settings.set(next);
            })),
            filter_active: *filter_panel_open.read(),
            has_active_filters: !filter.read().trim().is_empty()
                || !state_filter.read().is_empty(),
        }
    }
}

pub(crate) fn render_spec_list_sidebar(
    sidebar_collapsed: Signal<bool>,
    mut mobile_sidebar_open: Signal<bool>,
    specs: Signal<Vec<SpecSummary>>,
    loading: Signal<bool>,
    list_error: Signal<Option<String>>,
    mut filter: Signal<String>,
    mut state_filter: Signal<String>,
    nav: Navigator,
    navigation_store: crate::store::SpecNavigationStore,
    selected_id: Option<String>,
) -> Element {
    let nav_to_detail = nav.clone();
    let initially_expanded = {
        let specs = specs.read();
        selected_id
            .as_ref()
            .and_then(|selected_id| {
                specs
                    .iter()
                    .find(|spec| spec.id == *selected_id)
                    .and_then(|spec| spec.component.as_deref())
                    .filter(|component| !component.is_empty())
                    .map(|component| vec![format!("__folder__{component}")])
            })
            .unwrap_or_default()
    };
    rsx! {
        Sidebar {
            title: "Specifications".to_string(),
            collapsed: *sidebar_collapsed.read(),
            on_toggle: move |_| close_or_toggle_sidebar(sidebar_collapsed, mobile_sidebar_open),
            mobile_open: Some(*mobile_sidebar_open.read()),
            on_mobile_open_change: move |open| mobile_sidebar_open.set(open),
            SpecTree {
                specs: specs.read().clone(),
                loading: *loading.read(),
                error: list_error.read().clone(),
                filter: filter.read().clone(),
                on_filter_change: move |value: String| filter.set(value),
                state_filter: state_filter.read().clone(),
                on_state_filter_change: move |value: String| state_filter.set(value),
                selected_id: selected_id.clone(),
                initially_expanded,
                on_select: move |id: String| {
                    nav_to_detail.push(navigation_store.resolve_spec_detail_path(&id));
                    mobile_sidebar_open.set(false);
                },
            }
        }
    }
}

pub(super) fn render_spec_list_content(
    specs: Signal<Vec<SpecSummary>>,
    loading: Signal<bool>,
    list_error: Signal<Option<String>>,
    filter: Signal<String>,
    state_filter: Signal<String>,
    nav: Navigator,
    navigation_store: crate::store::SpecNavigationStore,
) -> Element {
    rsx! {
        div {
            class: "content",
            div {
                class: "spec-tree-page",
                div {
                    class: "spec-tree-page__header",
                    h2 {
                        class: "spec-tree-page__title",
                        "Specifications"
                    }
                    p {
                        class: "spec-tree-page__subtitle",
                        "Browse by component or use the sidebar tree to open a specification."
                    }
                }
                div {
                    class: "spec-tree-page__list",
                    if *loading.read() {
                        p { class: "spec-detail__loading", "Loading…" }
                    } else if let Some(err) = list_error.read().as_deref() {
                        p { class: "spec-detail__error", "Failed to load specifications: {err}" }
                    } else if specs.read().is_empty() {
                        div {
                            class: "empty-state",
                            if filter.read().trim().is_empty() && state_filter.read().is_empty() {
                                "No specifications are available yet."
                            } else {
                                "No specifications match the current search or state filter."
                            }
                        }
                    } else {
                        {render_spec_sections(specs.read().clone(), nav, navigation_store)}
                    }
                }
            }
        }
    }
}

fn render_spec_sections(
    specs: Vec<SpecSummary>,
    nav: Navigator,
    navigation_store: crate::store::SpecNavigationStore,
) -> Element {
    let mut grouped: BTreeMap<String, Vec<SpecSummary>> = BTreeMap::new();
    for spec in specs {
        let category = spec
            .component
            .clone()
            .unwrap_or_else(|| "Uncategorised".to_string());
        grouped.entry(category).or_default().push(spec);
    }

    rsx! {
        for (category, items) in grouped.into_iter() {
            CardSection {
                key: "{category}",
                title: category.clone(),
                count: Some(items.len()),
                CardGrid {
                    for spec in items {
                        {
                            let id = spec.id.clone();
                            let title = spec
                                .title
                                .clone()
                                .unwrap_or_else(|| "Untitled".to_string());
                            let description = spec.state.clone();
                            let nav_to_detail = nav.clone();
                            rsx! {
                                Card {
                                    key: "{spec.id}",
                                    title,
                                    description,
                                    on_click: EventHandler::new(move |_| {
                                        nav_to_detail.push(navigation_store.resolve_spec_detail_path(&id));
                                    }),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
