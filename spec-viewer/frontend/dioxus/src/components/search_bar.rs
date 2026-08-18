//! `SearchBar` — text input that triggers spec search.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SearchBarProps {
    pub value: String,
    pub on_change: EventHandler<String>,
    #[props(default = "Search specs…".to_string())]
    pub placeholder: String,
}

#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    rsx! {
        div {
            class: "sidebar-search",
            input {
                r#type: "text",
                value: "{props.value}",
                placeholder: "{props.placeholder}",
                oninput: move |e| props.on_change.call(e.value()),
            }
        }
    }
}
