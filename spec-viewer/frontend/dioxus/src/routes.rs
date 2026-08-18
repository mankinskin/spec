//! Dioxus Router route definitions for the spec viewer SPA.
//!
//! Route hierarchy:
//!
//!   /                → Redirect to /specs
//!   /specs           → SpecListPage (root browse surface)
//!   /specs/tree      → Redirect to /specs (legacy compatibility)
//!   /specs/graph     → SpecGraphPage (full-screen spec graph)
//!   /specs/:id       → SpecDetailPage (detail page navigated directly)
//!   /specs/:id?view=<tab> → SpecDetailPage with canonical detail view

mod detail;
mod graph;
mod list;
mod tree;

use dioxus::prelude::*;
use percent_encoding::{
    utf8_percent_encode,
    AsciiSet,
    CONTROLS,
};

pub use detail::SpecDetailPage;
pub use graph::SpecGraphPage;
pub use list::SpecListPage;
pub use tree::SpecTreePage;

const SPEC_ID_PATH_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'?')
    .add(b'\\')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'{')
    .add(b'}');

pub const DEFAULT_SPEC_VIEW: &str = "body";

pub fn canonical_spec_view(view: Option<&str>) -> &'static str {
    match view {
        Some("sections") => "sections",
        Some("coderefs") | Some("code-refs") => "coderefs",
        Some("health") => "health",
        _ => DEFAULT_SPEC_VIEW,
    }
}

pub fn is_canonical_spec_detail_view(view: Option<&str>) -> bool {
    matches!(
        view,
        None | Some("sections") | Some("coderefs") | Some("health")
    )
}

pub fn is_spec_detail_view_available(
    _spec_id: &str,
    view: &str,
) -> bool {
    matches!(
        canonical_spec_view(Some(view)),
        "body" | "sections" | "coderefs" | "health"
    )
}

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[redirect("/", || Route::SpecListPage {})]
    #[route("/specs")]
    SpecListPage {},

    #[route("/specs/tree")]
    SpecTreePage {},

    #[route("/specs/graph")]
    SpecGraphPage {},

    #[route("/specs/:id?:view")]
    SpecDetailPage { id: String, view: Option<String> },
}

impl Route {
    #[allow(dead_code)]
    pub fn spec_detail(
        id: impl Into<String>,
        view: Option<&str>,
    ) -> Self {
        let id = id.into();
        match canonical_spec_view(view) {
            DEFAULT_SPEC_VIEW => Self::SpecDetailPage { id, view: None },
            view => Self::SpecDetailPage {
                id,
                view: Some(view.to_string()),
            },
        }
    }

    pub fn spec_detail_path(
        id: &str,
        view: Option<&str>,
    ) -> String {
        let id = utf8_percent_encode(id, SPEC_ID_PATH_ENCODE_SET).to_string();
        match canonical_spec_view(view) {
            DEFAULT_SPEC_VIEW => format!("/specs/{id}"),
            view => format!("/specs/{id}?view={view}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omits_query_for_default_body_view() {
        assert_eq!(
            Route::spec_detail("spec:alpha", None),
            Route::SpecDetailPage {
                id: "spec:alpha".to_string(),
                view: None,
            }
        );
        assert_eq!(
            Route::spec_detail("spec:alpha", Some("body")),
            Route::SpecDetailPage {
                id: "spec:alpha".to_string(),
                view: None,
            }
        );
    }

    #[test]
    fn normalizes_legacy_and_named_views() {
        assert_eq!(canonical_spec_view(Some("code-refs")), "coderefs");
        assert_eq!(canonical_spec_view(Some("unknown")), DEFAULT_SPEC_VIEW);
        assert_eq!(
            Route::spec_detail("spec:beta", Some("coderefs")),
            Route::SpecDetailPage {
                id: "spec:beta".to_string(),
                view: Some("coderefs".to_string()),
            }
        );
        assert_eq!(
            Route::spec_detail("spec:beta", Some("code-refs")),
            Route::SpecDetailPage {
                id: "spec:beta".to_string(),
                view: Some("coderefs".to_string()),
            }
        );
    }

    #[test]
    fn formats_canonical_detail_paths_without_empty_query() {
        assert_eq!(
            Route::spec_detail_path("spec:alpha", None),
            "/specs/spec:alpha"
        );
        assert_eq!(
            Route::spec_detail_path("spec:alpha", Some("body")),
            "/specs/spec:alpha"
        );
        assert_eq!(
            Route::spec_detail_path("spec:alpha", Some("sections")),
            "/specs/spec:alpha?view=sections"
        );
        assert_eq!(
            Route::spec_detail_path("spec:alpha", Some("code-refs")),
            "/specs/spec:alpha?view=coderefs"
        );
        assert_eq!(
            Route::spec_detail_path("spec/alpha", Some("sections")),
            "/specs/spec%2Falpha?view=sections"
        );
    }

    #[test]
    fn recognizes_current_spec_views_as_available() {
        assert!(is_spec_detail_view_available("spec:alpha", "body"));
        assert!(is_spec_detail_view_available("spec:alpha", "code-refs"));
        assert!(!is_spec_detail_view_available("spec:alpha", "unknown"));
    }
}
