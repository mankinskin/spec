use dioxus::prelude::*;
use viewer_api_dioxus::{
    BreadcrumbItem,
    Breadcrumbs,
    HamburgerIcon,
    Layout,
    Overlay,
    PageHeader,
    ThemeSettings,
};

use crate::{
    components::spec_detail::SpecDetail,
    sse::use_sse,
    store::SpecListStore,
    types::SpecSummary,
};

use super::{
    list::{
        persist_store,
        render_spec_list_sidebar,
        sidebar_button_state,
        toggle_sidebar,
        use_spec_list,
    },
    Route,
};

#[component]
pub fn SpecDetailPage(
    id: String,
    view: Option<String>,
) -> Element {
    let store = SpecListStore::use_store();
    let sidebar_collapsed = use_signal(|| false);
    let mobile_sidebar_open = use_signal(|| false);
    let mut show_theme_settings = use_signal(|| false);
    let specs: Signal<Vec<SpecSummary>> = use_signal(Vec::new);
    let loading: Signal<bool> = use_signal(|| true);
    let list_error: Signal<Option<String>> = use_signal(|| None);
    let filter = store.filter;
    let state_filter = store.state_filter;
    let navigation_store = use_context::<crate::store::SpecNavigationStore>();
    let nav = use_navigator();
    let title = id.clone();
    let active_tab = super::canonical_spec_view(view.as_deref()).to_string();

    persist_store(store);
    use_spec_list(specs, loading, list_error, filter, state_filter);
    use_sse(specs);

    let (sidebar_button_active, sidebar_button_label) =
        sidebar_button_state(sidebar_collapsed, mobile_sidebar_open);

    let nav_specs = nav.clone();
    let nav_graph = nav.clone();
    let nav_home = nav.clone();
    let crumbs: Vec<BreadcrumbItem> = vec![
        BreadcrumbItem::link(
            "Specs",
            EventHandler::new(move |_| {
                nav_specs.push(Route::SpecListPage {});
            }),
        )
        .with_href("/specs"),
        BreadcrumbItem::link(
            "Graph",
            EventHandler::new(move |_| {
                nav_graph.push(Route::SpecGraphPage {});
            }),
        )
        .with_href("/specs/graph"),
        BreadcrumbItem::current(title.clone()),
    ];

    let nav_normalize = nav.clone();
    let id_for_normalize = id.clone();
    let route_view = view.clone();
    use_effect(use_reactive!(|id_for_normalize, route_view| {
        if !super::is_canonical_spec_detail_view(route_view.as_deref()) {
            nav_normalize.replace(Route::spec_detail_path(
                &id_for_normalize,
                route_view.as_deref(),
            ));
        }
    }));

    let id_for_view_memory = id.clone();
    let active_tab_for_view_memory = active_tab.clone();
    use_effect(use_reactive!(
        |id_for_view_memory, active_tab_for_view_memory| {
            navigation_store.remember_spec_view(
                &id_for_view_memory,
                &active_tab_for_view_memory,
            );
        }
    ));

    let nav_tabs = nav;
    let id_for_tabs = id.clone();
    let active_tab_for_tabs = active_tab.clone();
    let navigation_store_for_tabs = navigation_store;
    rsx! {
        Layout {
            header: rsx! {
                PageHeader {
                    lead: Some(rsx! {
                        button {
                            class: if sidebar_button_active { "btn btn-icon btn-active" } else { "btn btn-icon" },
                            "data-testid": "spec-detail-spec-list-toggle",
                            aria_label: sidebar_button_label,
                            title: sidebar_button_label,
                            onclick: move |_| toggle_sidebar(sidebar_collapsed, mobile_sidebar_open),
                            HamburgerIcon {}
                        }
                    }),
                    left_extra: Some(rsx! {
                        Breadcrumbs {
                            items: crumbs,
                            class: "spec-detail__breadcrumbs".to_string(),
                        }
                    }),
                    right_prefix: Some(rsx! {
                        Link {
                            to: Route::SpecGraphPage {},
                            class: "btn-nav-link",
                            "\u{1F310} Graph"
                        }
                    }),
                    on_home: Some(EventHandler::new(move |_| {
                        nav_home.push(Route::SpecListPage {});
                    })),
                    on_theme_toggle: Some(EventHandler::new(move |_| {
                        let next = !*show_theme_settings.read();
                        show_theme_settings.set(next);
                    })),
                }
            },
            {render_spec_list_sidebar(
                sidebar_collapsed,
                mobile_sidebar_open,
                specs,
                loading,
                list_error,
                filter,
                state_filter,
                nav.clone(),
                navigation_store,
                Some(id.clone()),
            )}
            div {
                class: "spec-detail-page",
                SpecDetail {
                    spec_id: id,
                    active_tab: active_tab.clone(),
                    on_tab_change: move |tab: String| {
                        if tab == active_tab_for_tabs {
                            return;
                        }
                        navigation_store_for_tabs.remember_spec_view(
                            &id_for_tabs,
                            &tab,
                        );
                        nav_tabs.push(Route::spec_detail_path(
                            &id_for_tabs,
                            Some(tab.as_str()),
                        ));
                    },
                }
            }
        }
        Overlay {
            open: *show_theme_settings.read(),
            on_close: move |_| show_theme_settings.set(false),
            panel_class: "theme-settings-modal".to_string(),
            aria_label: "Theme settings".to_string(),
            ThemeSettings {
                on_close: move |_| show_theme_settings.set(false),
            }
        }
    }
}
