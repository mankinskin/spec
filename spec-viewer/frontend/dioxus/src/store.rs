//! Per-page UI state store for [`SpecListPage`].
//!
//! State is persisted to `localStorage` under the key `spec-viewer:ui`
//! and restored on mount for the browse filters shown on `/specs`.

use dioxus::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use viewer_api_dioxus::{
    graph3d::CameraMode,
    Camera,
    Layout3D,
};

use crate::{
    components::spec_graph::{
        LayoutAlgorithm,
        LayoutParams,
        SELECTED_NODE_ZOOM_FACTOR_DEFAULT,
    },
    types::{
        SpecGraphEdge,
        SpecGraphNode,
    },
};

// ── localStorage key ──────────────────────────────────────────────────────────

#[allow(dead_code)]
fn ls_key() -> &'static str {
    "spec-viewer:ui"
}

// ── Serialisable snapshot ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecUiState {
    /// Text filter applied to spec title / slug / component.
    #[serde(default)]
    pub filter: String,
    /// State filter (e.g. `"draft"`, `"reviewed"`, …).
    #[serde(default)]
    pub state_filter: String,
}

impl Default for SpecUiState {
    fn default() -> Self {
        Self {
            filter: String::new(),
            state_filter: String::new(),
        }
    }
}

// ── Store ─────────────────────────────────────────────────────────────────────

/// Reactive UI state store for the spec list page.
#[derive(Clone, Copy)]
pub struct SpecListStore {
    pub filter: Signal<String>,
    pub state_filter: Signal<String>,
}

impl SpecListStore {
    /// Mount and return the store, restoring persisted state if available.
    pub fn use_store() -> Self {
        let saved = Self::load_from_storage();

        let filter = use_signal(|| saved.filter.clone());
        let state_filter = use_signal(|| saved.state_filter.clone());

        Self {
            filter,
            state_filter,
        }
    }

    /// Flush current state to localStorage.
    pub fn persist(&self) {
        let snapshot = SpecUiState {
            filter: self.filter.read().clone(),
            state_filter: self.state_filter.read().clone(),
        };
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(json) = serde_json::to_string(&snapshot) {
                if let Some(win) = web_sys::window() {
                    if let Ok(Some(storage)) = win.local_storage() {
                        let _ = storage.set_item(ls_key(), &json);
                    }
                }
            }
        }
    }

    fn load_from_storage() -> SpecUiState {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(ls_key()).ok().flatten())
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            SpecUiState::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpecNavigationStore {
    pub global_spec_view: Signal<String>,
    pub last_view_by_spec: Signal<HashMap<String, String>>,
}

impl SpecNavigationStore {
    pub fn use_store() -> Self {
        Self {
            global_spec_view: use_signal(|| {
                crate::routes::DEFAULT_SPEC_VIEW.to_string()
            }),
            last_view_by_spec: use_signal(HashMap::new),
        }
    }

    pub fn remember_spec_view(
        mut self,
        spec_id: &str,
        view: &str,
    ) {
        let view = crate::routes::canonical_spec_view(Some(view)).to_string();

        if *self.global_spec_view.peek() != view {
            self.global_spec_view.set(view.clone());
        }

        let mut last_view_by_spec = self.last_view_by_spec.peek().clone();
        if last_view_by_spec.get(spec_id) != Some(&view) {
            last_view_by_spec.insert(spec_id.to_string(), view);
            self.last_view_by_spec.set(last_view_by_spec);
        }
    }

    pub fn resolve_spec_view(
        self,
        spec_id: &str,
    ) -> String {
        let global_view = crate::routes::canonical_spec_view(Some(
            self.global_spec_view.read().as_str(),
        ));
        if crate::routes::is_spec_detail_view_available(spec_id, global_view) {
            return global_view.to_string();
        }

        let remembered_view = self
            .last_view_by_spec
            .read()
            .get(spec_id)
            .map(String::as_str)
            .map(|view| crate::routes::canonical_spec_view(Some(view)));
        if let Some(view) = remembered_view {
            if crate::routes::is_spec_detail_view_available(spec_id, view) {
                return view.to_string();
            }
        }

        crate::routes::DEFAULT_SPEC_VIEW.to_string()
    }

    pub fn resolve_spec_detail_path(
        self,
        spec_id: &str,
    ) -> String {
        let view = self.resolve_spec_view(spec_id);
        crate::routes::Route::spec_detail_path(spec_id, Some(view.as_str()))
    }
}

/// Shared graph-page state kept alive above the router so the global spec
/// graph does not reset to defaults on every route transition.
#[derive(Clone, Copy)]
pub struct SpecGraphStore {
    pub raw: Signal<Option<(Vec<SpecGraphNode>, Vec<SpecGraphEdge>)>>,
    pub error: Signal<Option<String>>,
    pub draft_algo: Signal<LayoutAlgorithm>,
    pub draft_params: Signal<LayoutParams>,
    pub draft_show_edges: Signal<bool>,
    pub camera_mode: Signal<CameraMode>,
    pub center_camera_on_selected_node: Signal<bool>,
    pub zoom_to_selected_node: Signal<bool>,
    pub selected_node_zoom_factor: Signal<f32>,
    pub auto_layout_selected_node: Signal<bool>,
    pub committed_algo: Signal<LayoutAlgorithm>,
    pub committed_params: Signal<LayoutParams>,
    pub committed_show_edges: Signal<bool>,
    pub auto_apply: Signal<bool>,
    pub panel_open: Signal<bool>,
    pub current_layout: Signal<Option<Layout3D>>,
    pub current_camera: Signal<Option<Camera>>,
    pub layout_generation: Signal<u64>,
    pub applied_layout_generation: Signal<u64>,
}

impl SpecGraphStore {
    pub fn use_store() -> Self {
        Self {
            raw: use_signal(|| None),
            error: use_signal(|| None),
            draft_algo: use_signal(|| LayoutAlgorithm::ForceDirected),
            draft_params: use_signal(LayoutParams::default),
            draft_show_edges: use_signal(|| true),
            camera_mode: use_signal(CameraMode::default),
            center_camera_on_selected_node: use_signal(|| false),
            zoom_to_selected_node: use_signal(|| false),
            selected_node_zoom_factor: use_signal(|| {
                SELECTED_NODE_ZOOM_FACTOR_DEFAULT
            }),
            auto_layout_selected_node: use_signal(|| false),
            committed_algo: use_signal(|| LayoutAlgorithm::ForceDirected),
            committed_params: use_signal(LayoutParams::default),
            committed_show_edges: use_signal(|| true),
            auto_apply: use_signal(|| true),
            panel_open: use_signal(|| true),
            current_layout: use_signal(|| None),
            current_camera: use_signal(|| None),
            layout_generation: use_signal(|| 0),
            applied_layout_generation: use_signal(|| 0),
        }
    }

    pub fn mark_layout_dirty(self) {
        let mut layout_generation = self.layout_generation;
        let next = *layout_generation.peek() + 1;
        layout_generation.set(next);
    }
}
