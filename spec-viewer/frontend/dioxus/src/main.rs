mod api;
mod components;
mod routes;
mod sse;
mod store;
mod types;

use dioxus::prelude::*;
use viewer_api_dioxus::{
    Prefetcher,
    ThemeProvider,
    ViewerShell,
    WgpuOverlay,
};

use routes::Route;
use store::{
    SpecGraphStore,
    SpecNavigationStore,
};
use types::SpecFullResponse;

/// Type alias for the spec-detail prefetch cache shared across all routes.
///
/// Keyed by spec id, holds a fully-loaded [`SpecFullResponse`] so that
/// switching to a sibling tab renders without a network round-trip.
pub type SpecCache = Prefetcher<String, SpecFullResponse>;

fn main() {
    #[cfg(target_arch = "wasm32")]
    viewer_api_dioxus::tracing_setup::install();
    dioxus::launch(App);
}

/// Root application component for the spec viewer.
///
/// Mounts the shared `ViewerShell` (WebGPU canvas + UI overlay) from
/// `viewer-api-dioxus` and nests the spec-viewer SPA router inside the
/// overlay so all route components render on top of the GPU canvas layer.
#[component]
fn App() -> Element {
    // Provide a single shared Prefetcher cache (LRU, 32 entries) for the
    // entire app.  P5.7: SpecListPage warms it with siblings of the active
    // tab; SpecDetail consults it before falling back to the network.
    let graph_store = SpecGraphStore::use_store();
    let navigation_store = SpecNavigationStore::use_store();
    use_context_provider::<SpecCache>(|| Prefetcher::with_capacity(32));
    use_context_provider(|| graph_store);
    use_context_provider(|| navigation_store);

    rsx! {
        style { "html, body, #main {{ overflow: hidden; margin: 0; padding: 0; width: 100%; height: 100%; }}" }
        ThemeProvider {
            ViewerShell {
                WgpuOverlay {}
                Router::<Route> {}
            }
        }
    }
}
