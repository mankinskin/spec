use dioxus::prelude::*;
use viewer_api_dioxus::{
    graph3d::{
        camera::frame_distance,
        CameraMode,
    },
    Camera,
    CameraCommand,
    Layout3D,
    NodeViewTransform,
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    api,
    store::SpecGraphStore,
    types::SpecGraphNode,
};

use super::{
    cards::render_graph_node_cards,
    layouts::{
        build_layout,
        FrustumLayoutContext,
    },
    model::{
        LayoutAlgorithm,
        SELECTED_NODE_ZOOM_FACTOR_MAX,
        SELECTED_NODE_ZOOM_FACTOR_MIN,
    },
    preview::SpecPreviewSidebar,
    settings::{
        queue_camera_command,
        render_graph_settings_panel,
    },
};

enum GraphPageState {
    Error(String),
    Loading(&'static str),
    Ready {
        nodes_raw: Vec<SpecGraphNode>,
        layout: Layout3D,
    },
}

const SETTINGS_PANEL_VIEWPORT_INSET_LEFT: f32 = 284.0;
const PREVIEW_VIEWPORT_INSET_RIGHT: f32 = 368.0;

#[derive(Clone, PartialEq)]
struct CameraFocusRequest {
    selection_key: Option<String>,
    target: [f32; 3],
    distance: f32,
}

#[component]
pub fn SpecGraphPage() -> Element {
    let mut store = use_context::<SpecGraphStore>();
    let navigation_store = use_context::<crate::store::SpecNavigationStore>();
    let camera_cmd: Signal<Option<CameraCommand>> = use_signal(|| None);
    let camera_seq: Signal<u64> = use_signal(|| 0);
    let last_cam_algo: Signal<LayoutAlgorithm> =
        use_hook(|| Signal::new(LayoutAlgorithm::ForceDirected));
    let last_focus_request: Signal<Option<CameraFocusRequest>> =
        use_hook(|| Signal::new(None));
    let applied_frustum_context: Signal<Option<FrustumLayoutContext>> =
        use_hook(|| Signal::new(None));
    let mut preview_id: Signal<Option<String>> = use_signal(|| None);
    let hovered_id: Signal<Option<String>> = use_signal(|| None);
    let nav = use_navigator();
    let preview_open = preview_id.read().is_some();
    let viewport_insets =
        graph_viewport_insets(*store.panel_open.read(), preview_open);
    let node_view_transform = current_node_view_transform(store);

    use_graph_fetch(store);
    use_layout_sync(store, viewport_insets, applied_frustum_context);
    sync_camera_for_algorithm(store, last_cam_algo, camera_cmd, camera_seq);

    let (nodes_raw, layout) = match graph_page_state(store) {
        GraphPageState::Error(message) =>
            return render_status(
                "empty-state",
                Some("color: var(--error);"),
                &format!("Failed to load graph: {message}"),
            ),
        GraphPageState::Loading(message) =>
            return render_status("empty-state", None, message),
        GraphPageState::Ready { nodes_raw, layout } => (nodes_raw, layout),
    };

    let nodes = layout.nodes.clone();
    let node_count = nodes.len();
    let edge_count = layout.edges.len();
    let camera_mode = *store.camera_mode.read();
    let camera_command = *camera_cmd.read();
    let camera_command_seq = *camera_seq.read();
    let selected_node_id = preview_id.read().clone();
    let selection_auto_layout = *store.auto_layout_selected_node.read();
    sync_camera_for_selected_node(
        store,
        &layout,
        selected_node_id.as_deref(),
        last_focus_request,
        camera_cmd,
        camera_seq,
    );
    let hovered_node_id = hovered_id.read().clone();

    rsx! {
        div { class: "graph-overlay",
            viewer_api_dioxus::Graph3D {
                layout: layout.clone(),
                initial_camera: store.current_camera.read().clone(),
                camera_mode,
                selected_node_id,
                hovered_node_id,
                selection_auto_layout,
                node_view_transform,
                viewport_insets,
                container_id: "spec-graph3d-container".to_string(),
                container_style: "position: absolute; inset: 0; overflow: hidden; user-select: none; cursor: grab;".to_string(),
                camera_command,
                camera_command_seq,
                on_layout_change: Some(EventHandler::new(move |layout: Layout3D| {
                    store.current_layout.set(Some(layout));
                })),
                on_camera_change: Some(EventHandler::new(move |camera: Camera| {
                    store.current_camera.set(Some(camera));
                })),
                {render_graph_node_cards(&nodes, &nodes_raw, preview_id, hovered_id)}
                div {
                    class: "graph-controls-hint",
                    "{graph_controls_hint(camera_mode)}"
                }
                if node_count > 0 {
                    div {
                        class: "graph-count-badge",
                        "{node_count} specs \u{00b7} {edge_count} edges"
                    }
                }
                button {
                    class: "graph-settings-toggle",
                    "data-testid": "graph-settings-toggle",
                    "data-graph-passthrough": "false",
                    aria_label: "Toggle graph settings",
                    onclick: move |event: Event<MouseData>| {
                        event.stop_propagation();
                        let visible = *store.panel_open.read();
                        store.panel_open.set(!visible);
                    },
                    if *store.panel_open.read() { "\u{2715} Settings" } else { "\u{2699} Settings" }
                }
                if *store.panel_open.read() {
                    {render_graph_settings_panel(store, camera_cmd, camera_seq)}
                }
            }
            if let Some(spec_id) = preview_id.read().clone() {
                SpecPreviewSidebar {
                    spec_id: spec_id.clone(),
                    on_close: move |_| preview_id.set(None),
                    on_view_details: move |id: String| {
                        preview_id.set(None);
                        nav.push(navigation_store.resolve_spec_detail_path(&id));
                    },
                }
            }
        }
    }
}

fn graph_controls_hint(camera_mode: CameraMode) -> &'static str {
    match camera_mode {
        CameraMode::Orbit => {
            "Left-drag: orbit · Right-drag: pan · Scroll: zoom · Click card: open"
        },
        CameraMode::Free => {
            "Left-drag: look · Right-drag: pan · Scroll: forward/back · Click card: open"
        },
    }
}

fn use_graph_fetch(store: SpecGraphStore) {
    let mut raw = store.raw;
    let mut error = store.error;

    use_effect(move || {
        if raw.read().is_some() || error.read().is_some() {
            return;
        }

        spawn_local(async move {
            match api::get_graph().await {
                Ok(response) => {
                    error.set(None);
                    raw.set(Some((response.nodes, response.edges)));
                },
                Err(message) => error.set(Some(message)),
            }
        });
    });
}

fn use_layout_sync(
    store: SpecGraphStore,
    viewport_insets: [f32; 4],
    mut applied_frustum_context: Signal<Option<FrustumLayoutContext>>,
) {
    let mut current_layout = store.current_layout;
    let mut applied_layout_generation = store.applied_layout_generation;
    use_effect(move || {
        let Some((nodes_raw, edges_raw)) = store.raw.read().clone() else {
            return;
        };

        let frustum_context =
            current_frustum_layout_context(store, viewport_insets);
        let generation = *store.layout_generation.read();
        let needs_rebuild = current_layout.peek().is_none()
            || generation != *applied_layout_generation.peek()
            || frustum_context != *applied_frustum_context.peek();
        if !needs_rebuild {
            return;
        }

        let edges_for_layout = if *store.committed_show_edges.read() {
            edges_raw.clone()
        } else {
            Vec::new()
        };
        let algo = *store.committed_algo.read();
        let params = *store.committed_params.read();
        let layout = build_layout(
            algo,
            params,
            &nodes_raw,
            &edges_for_layout,
            frustum_context.clone(),
        );
        current_layout.set(Some(layout));
        applied_layout_generation.set(generation);
        applied_frustum_context.set(frustum_context);
    });
}

fn current_frustum_layout_context(
    store: SpecGraphStore,
    viewport_insets: [f32; 4],
) -> Option<FrustumLayoutContext> {
    let algo = *store.committed_algo.read();
    let params = *store.committed_params.read();
    if !matches!(algo, LayoutAlgorithm::ForceDirected)
        || !params.frustum_gravity_enabled
        || params.frustum_gravity <= 0.0
    {
        return None;
    }

    let (viewport_width, viewport_height) =
        current_viewport_size(viewport_insets)?;
    let camera = canonical_frustum_layout_camera(
        store.current_camera.read().clone().unwrap_or_default(),
    );

    Some(FrustumLayoutContext {
        camera,
        aspect: (viewport_width / viewport_height).max(0.1),
        viewport_width,
        viewport_height,
    })
}

fn canonical_frustum_layout_camera(camera: Camera) -> Camera {
    // The force-side frustum pass only depends on view direction and viewport.
    // Ignore live target translation and distance so focus/zoom camera updates do
    // not retrigger the solver indefinitely.
    Camera {
        yaw: camera.yaw,
        pitch: camera.pitch,
        distance: 1.0,
        target: [0.0, 0.0, 0.0],
    }
}

#[cfg(target_arch = "wasm32")]
fn current_viewport_size(viewport_insets: [f32; 4]) -> Option<(f32, f32)> {
    let window = web_sys::window()?;
    let width = window.inner_width().ok()?.as_f64()? as f32
        - viewport_insets[0]
        - viewport_insets[2];
    let height = window.inner_height().ok()?.as_f64()? as f32
        - viewport_insets[1]
        - viewport_insets[3];
    Some((width.max(320.0), height.max(240.0)))
}

#[cfg(not(target_arch = "wasm32"))]
fn current_viewport_size(_viewport_insets: [f32; 4]) -> Option<(f32, f32)> {
    None
}

fn sync_camera_for_algorithm(
    store: SpecGraphStore,
    mut last_cam_algo: Signal<LayoutAlgorithm>,
    camera_cmd: Signal<Option<CameraCommand>>,
    camera_seq: Signal<u64>,
) {
    let current_algo = *store.committed_algo.read();
    if current_algo == *last_cam_algo.peek() {
        return;
    }

    last_cam_algo.set(current_algo);
    queue_camera_command(
        camera_cmd,
        camera_seq,
        current_algo.preferred_camera(),
    );
}

fn sync_camera_for_selected_node(
    store: SpecGraphStore,
    layout: &Layout3D,
    selected_node_id: Option<&str>,
    mut last_focus_request: Signal<Option<CameraFocusRequest>>,
    camera_cmd: Signal<Option<CameraCommand>>,
    camera_seq: Signal<u64>,
) {
    let center_camera_on_selected_node =
        *store.center_camera_on_selected_node.read();
    let zoom_to_selected_node = *store.zoom_to_selected_node.read();
    let selected_node_zoom_factor = *store.selected_node_zoom_factor.read();
    let current_camera = store.current_camera.read().clone();

    if !(center_camera_on_selected_node || zoom_to_selected_node) {
        if last_focus_request.peek().is_some() {
            last_focus_request.set(None);
        }
        return;
    }

    let next_request = if selected_node_id.is_some() {
        selection_camera_request(
            layout,
            selected_node_id,
            current_camera.as_ref(),
            center_camera_on_selected_node,
            zoom_to_selected_node,
            selected_node_zoom_factor,
        )
    } else if last_focus_request.peek().is_some() {
        selection_camera_request(
            layout,
            None,
            current_camera.as_ref(),
            center_camera_on_selected_node,
            zoom_to_selected_node,
            selected_node_zoom_factor,
        )
    } else {
        None
    };

    let Some(request) = next_request else {
        return;
    };

    if last_focus_request.peek().as_ref() == Some(&request) {
        return;
    }

    last_focus_request.set(Some(request.clone()));
    queue_camera_command(
        camera_cmd,
        camera_seq,
        CameraCommand::FocusOn {
            target: request.target,
            distance: request.distance,
        },
    );
}

fn selection_camera_request(
    layout: &Layout3D,
    selected_node_id: Option<&str>,
    current_camera: Option<&Camera>,
    center_camera_on_selected_node: bool,
    zoom_to_selected_node: bool,
    selected_node_zoom_factor: f32,
) -> Option<CameraFocusRequest> {
    let (_centre, radius) = layout.bounds();
    let framed_distance = frame_distance(radius);

    if let Some(selected_node_id) = selected_node_id {
        if !(center_camera_on_selected_node || zoom_to_selected_node) {
            return None;
        }

        let node = layout
            .nodes
            .iter()
            .find(|node| node.id == selected_node_id)?;
        let distance = if zoom_to_selected_node {
            (framed_distance
                / selected_node_zoom_factor.clamp(
                    SELECTED_NODE_ZOOM_FACTOR_MIN,
                    SELECTED_NODE_ZOOM_FACTOR_MAX,
                ))
            .clamp(6.0, 120.0)
        } else {
            current_camera
                .map(|camera| camera.distance)
                .unwrap_or(framed_distance)
                .clamp(6.0, 120.0)
        };
        return Some(CameraFocusRequest {
            selection_key: Some(selected_node_id.to_string()),
            target: [node.x, node.y, node.z],
            distance,
        });
    }

    if layout.nodes.is_empty() {
        return None;
    }

    let (centre, radius) = layout.bounds();
    Some(CameraFocusRequest {
        selection_key: None,
        target: centre,
        distance: frame_distance(radius),
    })
}

fn current_node_view_transform(store: SpecGraphStore) -> NodeViewTransform {
    let algo = *store.committed_algo.read();
    let params = *store.committed_params.read();
    if !matches!(algo, LayoutAlgorithm::ForceDirected)
        || !params.frustum_gravity_enabled
        || params.frustum_gravity <= 0.0
    {
        return NodeViewTransform::default();
    }

    let strength = frustum_gravity_transform_strength(params.frustum_gravity);
    if strength <= 0.01 {
        return NodeViewTransform::default();
    }

    let node_count = store
        .current_layout
        .read()
        .as_ref()
        .map(|layout| layout.nodes.len())
        .unwrap_or(0) as f32;
    let screen_fill = frustum_gravity_screen_fill(
        strength,
        node_count,
        params.frustum_overfill,
    );
    NodeViewTransform::camera_plane_view_direction(screen_fill, strength)
}

fn frustum_gravity_transform_strength(frustum_gravity: f32) -> f32 {
    let normalized = ((frustum_gravity - 0.95) / 1.4).clamp(0.0, 1.0);
    normalized * normalized * (3.0 - 2.0 * normalized)
}

fn frustum_gravity_screen_fill(
    strength: f32,
    node_count: f32,
    frustum_overfill: f32,
) -> f32 {
    let overfill = (node_count.max(12.0) / 12.0).sqrt().clamp(1.0, 3.5);
    overfill * frustum_overfill.clamp(0.5, 2.0) * (0.72 + strength * 0.22)
}

fn graph_viewport_insets(
    panel_open: bool,
    preview_open: bool,
) -> [f32; 4] {
    [
        if panel_open {
            SETTINGS_PANEL_VIEWPORT_INSET_LEFT
        } else {
            0.0
        },
        0.0,
        if preview_open {
            PREVIEW_VIEWPORT_INSET_RIGHT
        } else {
            0.0
        },
        0.0,
    ]
}

fn graph_page_state(store: SpecGraphStore) -> GraphPageState {
    if let Some(message) = store.error.read().clone() {
        return GraphPageState::Error(message);
    }

    let Some((nodes_raw, _)) = store.raw.read().clone() else {
        return GraphPageState::Loading("Loading graph\u{2026}");
    };
    let Some(layout) = store.current_layout.read().clone() else {
        return GraphPageState::Loading("Preparing graph layout\u{2026}");
    };

    GraphPageState::Ready { nodes_raw, layout }
}

fn render_status(
    class: &str,
    style: Option<&str>,
    message: &str,
) -> Element {
    rsx! {
        div {
            class: "{class}",
            style: style.unwrap_or_default(),
            "{message}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_frustum_layout_camera,
        selection_camera_request,
    };
    use viewer_api_dioxus::{
        graph3d::camera::frame_distance,
        Camera,
        Layout3D,
        Node3D,
    };

    #[test]
    fn canonical_frustum_layout_camera_ignores_focus_target_and_zoom() {
        let near_focus = Camera {
            yaw: 0.35,
            pitch: -0.2,
            distance: 9.0,
            target: [14.0, -3.5, 8.0],
        };
        let far_focus = Camera {
            distance: 38.0,
            target: [-11.0, 6.0, -27.0],
            ..near_focus
        };

        assert_eq!(
            canonical_frustum_layout_camera(near_focus),
            canonical_frustum_layout_camera(far_focus),
        );
    }

    #[test]
    fn canonical_frustum_layout_camera_preserves_view_direction() {
        let base = Camera {
            yaw: 0.35,
            pitch: -0.2,
            distance: 9.0,
            target: [14.0, -3.5, 8.0],
        };
        let rotated = Camera {
            yaw: base.yaw + 0.15,
            ..base
        };

        assert_ne!(
            canonical_frustum_layout_camera(base),
            canonical_frustum_layout_camera(rotated),
        );
    }

    #[test]
    fn selection_camera_request_uses_linear_zoom_scaling() {
        let layout = Layout3D::new(
            vec![
                Node3D {
                    id: "selected".to_string(),
                    label: Some("Selected".to_string()),
                    state: Some("draft".to_string()),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                Node3D {
                    id: "other".to_string(),
                    label: Some("Other".to_string()),
                    state: Some("draft".to_string()),
                    x: 12.0,
                    y: 0.0,
                    z: 0.0,
                },
            ],
            Vec::new(),
        );

        let request = selection_camera_request(
            &layout,
            Some("selected"),
            None,
            true,
            true,
            3.0,
        )
        .expect("request");

        let expected = frame_distance(layout.bounds().1) / 3.0;
        assert!((request.distance - expected).abs() < 1e-4);
    }
}
