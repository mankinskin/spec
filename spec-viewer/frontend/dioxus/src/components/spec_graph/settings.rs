use dioxus::prelude::*;
use viewer_api_dioxus::{
    graph3d::CameraMode,
    CameraCommand,
};

use crate::store::SpecGraphStore;

use super::model::{
    LayoutAlgorithm,
    LayoutParams,
    SELECTED_NODE_ZOOM_FACTOR_MAX,
    SELECTED_NODE_ZOOM_FACTOR_MIN,
};

pub(super) fn render_graph_settings_panel(
    mut store: SpecGraphStore,
    camera_cmd: Signal<Option<CameraCommand>>,
    camera_seq: Signal<u64>,
) -> Element {
    let draft_algo = *store.draft_algo.read();
    let draft_params = *store.draft_params.read();
    let draft_show_edges = *store.draft_show_edges.read();
    let camera_mode = *store.camera_mode.read();
    let center_camera_on_selected_node =
        *store.center_camera_on_selected_node.read();
    let zoom_to_selected_node = *store.zoom_to_selected_node.read();
    let selected_node_zoom_factor = *store.selected_node_zoom_factor.read();
    let auto_layout_selected_node = *store.auto_layout_selected_node.read();
    let auto_apply = *store.auto_apply.read();
    let apply_disabled = auto_apply
        || !has_draft_changes(
            store,
            draft_algo,
            draft_params,
            draft_show_edges,
        );

    rsx! {
        div {
            class: "graph-settings-panel",
            "data-testid": "graph-settings-panel",
            "data-graph-passthrough": "false",
            onclick: move |event: Event<MouseData>| event.stop_propagation(),
            onmousedown: move |event: Event<MouseData>| event.stop_propagation(),
            onwheel: move |event: Event<WheelData>| event.stop_propagation(),

            h3 { class: "graph-settings-panel__title", "Graph settings" }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label graph-settings-label--inline",
                    input {
                        r#type: "checkbox",
                        "data-testid": "graph-toggle-auto-apply",
                        checked: auto_apply,
                        onchange: move |event| set_auto_apply(store, event.checked()),
                    }
                    " Auto-apply"
                }
            }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label", "Layout algorithm" }
                select {
                    class: "graph-settings-select",
                    "data-testid": "graph-algo-select",
                    value: draft_algo.as_str(),
                    onchange: move |event| {
                        if let Some(algo) = LayoutAlgorithm::from_str_opt(&event.value()) {
                            set_draft_algorithm(store, algo);
                        }
                    },
                    for algo in LayoutAlgorithm::ALL.iter() {
                        option { value: algo.as_str(), "{algo.label()}" }
                    }
                }
            }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label", "Camera mode" }
                select {
                    class: "graph-settings-select",
                    "data-testid": "graph-camera-mode-select",
                    value: camera_mode.as_str(),
                    onchange: move |event| {
                        if let Some(mode) = CameraMode::from_str_opt(&event.value()) {
                            set_camera_mode(store, mode);
                        }
                    },
                    for &mode in CameraMode::ALL.iter() {
                        option { value: mode.as_str(), "{mode.label()}" }
                    }
                }
            }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label",
                    "Spread "
                    span { class: "graph-settings-value", "{draft_params.spread:.2}" }
                }
                input {
                    r#type: "range",
                    min: "0.0", max: "6.0", step: "0.05",
                    value: "{draft_params.spread}",
                    oninput: move |event| {
                        if let Ok(value) = event.value().parse::<f32>() {
                            set_draft_params(store, |params| params.spread = value);
                        }
                    },
                }
            }

            if matches!(draft_algo, LayoutAlgorithm::RingsByDepth | LayoutAlgorithm::Grid) {
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label",
                        "Y spacing "
                        span { class: "graph-settings-value", "{draft_params.y_spacing:.2}" }
                    }
                    input {
                        r#type: "range",
                        min: "0.0", max: "4.0", step: "0.05",
                        value: "{draft_params.y_spacing}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f32>() {
                                set_draft_params(store, |params| params.y_spacing = value);
                            }
                        },
                    }
                }
            }

            if matches!(draft_algo, LayoutAlgorithm::ForceDirected) {
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label",
                        "Iterations "
                        span { class: "graph-settings-value", "{draft_params.iterations}" }
                    }
                    input {
                        r#type: "range",
                        min: "10", max: "400", step: "10",
                        value: "{draft_params.iterations}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<u32>() {
                                set_draft_params(store, |params| params.iterations = value);
                            }
                        },
                    }
                }
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label",
                        "Link distance "
                        span { class: "graph-settings-value", "{draft_params.link_dist:.2}" }
                    }
                    input {
                        r#type: "range",
                        min: "1.0", max: "12.0", step: "0.1",
                        value: "{draft_params.link_dist}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f32>() {
                                set_draft_params(store, |params| params.link_dist = value);
                            }
                        },
                    }
                }
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label",
                        "Repulsion "
                        span { class: "graph-settings-value", "{draft_params.repulsion:.2}" }
                    }
                    input {
                        r#type: "range",
                        min: "0.0", max: "12.0", step: "0.1",
                        value: "{draft_params.repulsion}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f32>() {
                                set_draft_params(store, |params| params.repulsion = value);
                            }
                        },
                    }
                }
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label graph-settings-label--inline",
                        input {
                            r#type: "checkbox",
                            "data-testid": "graph-toggle-frustum-gravity",
                            checked: draft_params.frustum_gravity_enabled,
                            onchange: move |event| {
                                set_draft_params(store, |params| {
                                    params.frustum_gravity_enabled = event.checked();
                                });
                            },
                        }
                        " Enable frustum gravity"
                    }
                }
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label",
                        "Frustum gravity "
                        span { class: "graph-settings-value", "{draft_params.frustum_gravity:.2}" }
                    }
                    input {
                        r#type: "range",
                        min: "0.0", max: "4.0", step: "0.05",
                        value: "{draft_params.frustum_gravity}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f32>() {
                                set_draft_params(store, |params| params.frustum_gravity = value);
                            }
                        },
                    }
                }
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label",
                        "Frustum spread "
                        span { class: "graph-settings-value", "{draft_params.frustum_overfill:.2}x" }
                    }
                    input {
                        r#type: "range",
                        min: "0.50", max: "2.00", step: "0.05",
                        value: "{draft_params.frustum_overfill}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f32>() {
                                set_draft_params(store, |params| params.frustum_overfill = value);
                            }
                        },
                    }
                }
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label",
                        "Frustum settle "
                        span { class: "graph-settings-value", "{draft_params.frustum_settle:.2}x" }
                    }
                    input {
                        r#type: "range",
                        min: "0.00", max: "3.00", step: "0.05",
                        value: "{draft_params.frustum_settle}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f32>() {
                                set_draft_params(store, |params| params.frustum_settle = value);
                            }
                        },
                    }
                }
                div { class: "graph-settings-section",
                    label { class: "graph-settings-label",
                        "Frustum overlap repulsion "
                        span { class: "graph-settings-value", "{draft_params.frustum_overlap_repulsion:.2}x" }
                    }
                    input {
                        r#type: "range",
                        min: "0.25", max: "4.00", step: "0.05",
                        value: "{draft_params.frustum_overlap_repulsion}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<f32>() {
                                set_draft_params(store, |params| params.frustum_overlap_repulsion = value);
                            }
                        },
                    }
                }
            }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label graph-settings-label--inline",
                    input {
                        r#type: "checkbox",
                        "data-testid": "graph-toggle-edges",
                        checked: draft_show_edges,
                        onchange: move |event| set_draft_show_edges(store, event.checked()),
                    }
                    " Show edges"
                }
            }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label graph-settings-label--inline",
                    input {
                        r#type: "checkbox",
                        "data-testid": "graph-toggle-center-selected-node",
                        checked: center_camera_on_selected_node,
                        onchange: move |event| set_center_camera_on_selected_node(store, event.checked()),
                    }
                    " Center camera on selected node"
                }
            }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label graph-settings-label--inline",
                    input {
                        r#type: "checkbox",
                        "data-testid": "graph-toggle-zoom-selected-node",
                        checked: zoom_to_selected_node,
                        onchange: move |event| set_zoom_to_selected_node(store, event.checked()),
                    }
                    " Zoom to selected node"
                }
            }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label",
                    "Zoom factor "
                    span { class: "graph-settings-value", "{selected_node_zoom_factor:.2}x" }
                }
                input {
                    r#type: "range",
                    "data-testid": "graph-range-selected-node-zoom-factor",
                    min: "1.0", max: "3.0", step: "0.25",
                    value: "{selected_node_zoom_factor}",
                    disabled: !zoom_to_selected_node,
                    oninput: move |event| {
                        if let Ok(value) = event.value().parse::<f32>() {
                            set_selected_node_zoom_factor(store, value);
                        }
                    },
                }
            }

            div { class: "graph-settings-section",
                label { class: "graph-settings-label graph-settings-label--inline",
                    input {
                        r#type: "checkbox",
                        "data-testid": "graph-toggle-auto-layout-selected-node",
                        checked: auto_layout_selected_node,
                        onchange: move |event| set_auto_layout_selected_node(store, event.checked()),
                    }
                    " Auto-layout selected node"
                }
            }

            div { class: "graph-settings-section graph-settings-actions",
                button {
                    class: "graph-settings-apply",
                    "data-testid": "graph-settings-apply",
                    disabled: apply_disabled,
                    onclick: move |event: Event<MouseData>| {
                        event.stop_propagation();
                        commit_draft(store);
                    },
                    "Apply"
                }
                button {
                    class: "graph-settings-reset",
                    "data-testid": "graph-settings-reset",
                    onclick: move |event: Event<MouseData>| {
                        event.stop_propagation();
                        store.draft_params.set(LayoutParams::default());
                        if *store.auto_apply.read() {
                            commit_draft(store);
                        }
                    },
                    "Reset"
                }
            }

            div { class: "graph-settings-section",
                button {
                    class: "graph-settings-reset",
                    "data-testid": "graph-reset-camera",
                    onclick: move |event: Event<MouseData>| {
                        event.stop_propagation();
                        queue_camera_command(camera_cmd, camera_seq, (*store.committed_algo.read()).preferred_camera());
                    },
                    "Reset camera"
                }
            }
        }
    }
}

pub(super) fn queue_camera_command(
    mut camera_cmd: Signal<Option<CameraCommand>>,
    mut camera_seq: Signal<u64>,
    command: CameraCommand,
) {
    camera_cmd.set(Some(command));
    let next = *camera_seq.peek() + 1;
    camera_seq.set(next);
}

fn has_draft_changes(
    store: SpecGraphStore,
    draft_algo: LayoutAlgorithm,
    draft_params: LayoutParams,
    draft_show_edges: bool,
) -> bool {
    draft_algo != *store.committed_algo.read()
        || draft_params != *store.committed_params.read()
        || draft_show_edges != *store.committed_show_edges.read()
}

fn set_auto_apply(
    mut store: SpecGraphStore,
    enabled: bool,
) {
    store.auto_apply.set(enabled);
    if enabled {
        commit_draft(store);
    }
}

fn set_draft_algorithm(
    mut store: SpecGraphStore,
    algo: LayoutAlgorithm,
) {
    store.draft_algo.set(algo);
    if *store.auto_apply.read() {
        commit_draft(store);
    }
}

fn set_draft_show_edges(
    mut store: SpecGraphStore,
    enabled: bool,
) {
    store.draft_show_edges.set(enabled);
    if *store.auto_apply.read() {
        commit_draft(store);
    }
}

fn set_zoom_to_selected_node(
    mut store: SpecGraphStore,
    enabled: bool,
) {
    store.zoom_to_selected_node.set(enabled);
}

fn set_camera_mode(
    mut store: SpecGraphStore,
    mode: CameraMode,
) {
    store.camera_mode.set(mode);
}

fn set_center_camera_on_selected_node(
    mut store: SpecGraphStore,
    enabled: bool,
) {
    store.center_camera_on_selected_node.set(enabled);
}

fn set_selected_node_zoom_factor(
    mut store: SpecGraphStore,
    value: f32,
) {
    store.selected_node_zoom_factor.set(
        value.clamp(
            SELECTED_NODE_ZOOM_FACTOR_MIN,
            SELECTED_NODE_ZOOM_FACTOR_MAX,
        ),
    );
}

fn set_auto_layout_selected_node(
    mut store: SpecGraphStore,
    enabled: bool,
) {
    store.auto_layout_selected_node.set(enabled);
}

fn set_draft_params(
    mut store: SpecGraphStore,
    update: impl FnOnce(&mut LayoutParams),
) {
    let mut params = *store.draft_params.read();
    update(&mut params);
    store.draft_params.set(params);
    if *store.auto_apply.read() {
        commit_draft(store);
    }
}

fn commit_draft(mut store: SpecGraphStore) {
    store.committed_algo.set(*store.draft_algo.read());
    store.committed_params.set(*store.draft_params.read());
    store
        .committed_show_edges
        .set(*store.draft_show_edges.read());
    store.mark_layout_dirty();
}
