use dioxus::prelude::*;

use super::Route;

#[component]
pub fn SpecTreePage() -> Element {
    let nav = use_navigator();

    use_effect(move || {
        nav.replace(Route::SpecListPage {});
    });

    rsx! {
        div {
            class: "empty-state",
            "Redirecting to specifications…"
        }
    }
}
