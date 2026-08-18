use dioxus::prelude::*;
use viewer_api_dioxus::{
    Layout,
    Overlay,
    PageHeader,
    ThemeSettings,
};

use crate::components::spec_graph::SpecGraphPage as SpecGraphView;

use super::Route;

#[component]
pub fn SpecGraphPage() -> Element {
    let nav = use_navigator();
    let nav_home = nav.clone();
    let mut show_theme_settings = use_signal(|| false);

    rsx! {
        Layout {
            class: "spec-graph-shell".to_string(),
            header: rsx! {
                PageHeader {
                    lead: Some(rsx! {
                        button {
                            class: "btn-back",
                            "data-testid": "spec-graph-back",
                            aria_label: "Back",
                            onclick: move |_| nav.go_back(),
                            "\u{2190} Back"
                        }
                    }),
                    icon: Some(rsx! { "\u{1F310}" }),
                    title: Some("Spec Graph".to_string()),
                    on_home: Some(EventHandler::new(move |_| {
                        nav_home.push(Route::SpecListPage {});
                    })),
                    on_theme_toggle: Some(EventHandler::new(move |_| {
                        let next = !*show_theme_settings.read();
                        show_theme_settings.set(next);
                    })),
                }
            },
            div {
                class: "spec-graph-page",
                SpecGraphView {}
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
