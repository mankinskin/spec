use dioxus::prelude::*;
use viewer_api_dioxus::is_mobile_sidebar_viewport;

pub(crate) fn sidebar_button_state(
    sidebar_collapsed: Signal<bool>,
    mobile_sidebar_open: Signal<bool>,
) -> (bool, &'static str) {
    if is_mobile_sidebar_viewport() {
        let open = *mobile_sidebar_open.read();
        return (
            open,
            if open {
                "Close specifications sidebar"
            } else {
                "Open specifications sidebar"
            },
        );
    }

    let collapsed = *sidebar_collapsed.read();
    (
        !collapsed,
        if collapsed {
            "Open specifications sidebar"
        } else {
            "Collapse specifications sidebar"
        },
    )
}

pub(crate) fn toggle_sidebar(
    mut sidebar_collapsed: Signal<bool>,
    mut mobile_sidebar_open: Signal<bool>,
) {
    if is_mobile_sidebar_viewport() {
        let next = !*mobile_sidebar_open.read();
        mobile_sidebar_open.set(next);
    } else {
        sidebar_collapsed.toggle();
    }
}

pub(super) fn close_or_toggle_sidebar(
    mut sidebar_collapsed: Signal<bool>,
    mut mobile_sidebar_open: Signal<bool>,
) {
    if is_mobile_sidebar_viewport() {
        mobile_sidebar_open.set(false);
    } else {
        let next = !*sidebar_collapsed.read();
        sidebar_collapsed.set(next);
    }
}
