//! Spec graph page — full 3-D view of every spec with parent + code-ref edges.
//!
//! Fetches `/api/specs/graph`, lays out nodes via the user-selected algorithm,
//! and renders via the shared `viewer_api_dioxus::Graph3D` canvas.

mod cards;
mod layouts;
mod model;
mod page;
mod preview;
mod settings;
mod view;

pub use model::{
    LayoutAlgorithm,
    LayoutParams,
    SELECTED_NODE_ZOOM_FACTOR_DEFAULT,
};
pub use page::SpecGraphPage;
