//! `CodeRefList` — table displaying CodeRef entries for a spec.

use crate::types::CodeRefEntry;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CodeRefListProps {
    pub refs: Vec<CodeRefEntry>,
}

#[component]
pub fn CodeRefList(props: CodeRefListProps) -> Element {
    if props.refs.is_empty() {
        return rsx! {
            p {
                style: "color: #6b7280; font-size: 13px; padding: 12px 0;",
                "No code references."
            }
        };
    }

    rsx! {
        div {
            style: "overflow-x: auto;",
            table {
                style: "width: 100%; border-collapse: collapse; font-size: 12px; color: #d1d5db;",
                thead {
                    tr {
                        style: "border-bottom: 1px solid #374151;",
                        th { style: "text-align: left; padding: 6px 8px; color: #9ca3af; font-weight: 600;", "File" }
                        th { style: "text-align: left; padding: 6px 8px; color: #9ca3af; font-weight: 600;", "Symbol" }
                        th { style: "text-align: left; padding: 6px 8px; color: #9ca3af; font-weight: 600;", "Kind" }
                        th { style: "text-align: left; padding: 6px 8px; color: #9ca3af; font-weight: 600;", "Lines" }
                        th { style: "text-align: left; padding: 6px 8px; color: #9ca3af; font-weight: 600;", "Description" }
                    }
                }
                tbody {
                    for cref in props.refs.iter() {
                        {
                            let lines = match (cref.line_start, cref.line_end) {
                                (Some(s), Some(e)) => format!("{s}–{e}"),
                                (Some(s), None) => format!("{s}"),
                                _ => String::new(),
                            };
                            let desc = cref.description.as_deref().unwrap_or("—");
                            rsx! {
                                tr {
                                    style: "border-bottom: 1px solid #1f2937;",
                                    td {
                                        style: "padding: 6px 8px; font-family: monospace; color: #60a5fa; white-space: nowrap;",
                                        "{cref.file}"
                                    }
                                    td {
                                        style: "padding: 6px 8px; font-family: monospace;",
                                        "{cref.symbol}"
                                    }
                                    td {
                                        style: "padding: 6px 8px; color: #a78bfa;",
                                        "{cref.kind}"
                                    }
                                    td {
                                        style: "padding: 6px 8px; font-family: monospace; white-space: nowrap;",
                                        "{lines}"
                                    }
                                    td {
                                        style: "padding: 6px 8px; color: #9ca3af;",
                                        "{desc}"
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
