use dioxus::prelude::*;
use viewer_api_dioxus::{
    Layout,
    Overlay,
    ThemeSettings,
};

use crate::{
    sse::use_sse,
    store::SpecListStore,
    types::SpecSummary,
};

use super::{
    super::Route,
    effects::{
        persist_store,
        use_spec_list,
    },
    helpers::sidebar_button_state,
    render::{
        render_spec_list_content,
        render_spec_list_header,
        render_spec_list_sidebar,
    },
};

#[component]
pub fn SpecListPage() -> Element {
    let store = SpecListStore::use_store();
    let sidebar_collapsed = use_signal(|| false);
    let mobile_sidebar_open = use_signal(|| false);
    let mut show_theme_settings = use_signal(|| false);
    let filter_panel_open = use_signal(|| false);
    let specs: Signal<Vec<SpecSummary>> = use_signal(Vec::new);
    let loading: Signal<bool> = use_signal(|| true);
    let list_error: Signal<Option<String>> = use_signal(|| None);
    let filter = store.filter;
    let state_filter = store.state_filter;
    let navigation_store = use_context::<crate::store::SpecNavigationStore>();
    let nav = use_navigator();

    persist_store(store);
    use_spec_list(specs, loading, list_error, filter, state_filter);
    use_sse(specs);

    let (sidebar_button_active, sidebar_button_label) =
        sidebar_button_state(sidebar_collapsed, mobile_sidebar_open);
    let nav_home = nav.clone();
    let on_home = EventHandler::new(move |_| {
        nav_home.push(Route::SpecListPage {});
    });

    rsx! {
        Layout {
            header: rsx! {
                {render_spec_list_header(
                    sidebar_button_active,
                    sidebar_button_label,
                    sidebar_collapsed,
                    mobile_sidebar_open,
                    filter_panel_open,
                    show_theme_settings,
                    filter,
                    state_filter,
                    on_home,
                )}
            },
            {render_spec_list_sidebar(
                sidebar_collapsed,
                mobile_sidebar_open,
                specs,
                loading,
                list_error,
                filter,
                state_filter,
                nav,
                navigation_store,
                None,
            )}
            {render_spec_list_content(
                specs,
                loading,
                list_error,
                filter,
                state_filter,
                nav,
                navigation_store,
            )}
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
