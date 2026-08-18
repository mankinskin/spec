use viewer_api_dioxus::Node3D;

use crate::types::SpecGraphNode;

use super::super::model::LayoutParams;

pub(super) fn layout_sphere(
    nodes: &[SpecGraphNode],
    params: LayoutParams,
) -> Vec<Node3D> {
    let node_count = nodes.len() as f32;
    let radius = (node_count.sqrt() * 0.9 + 4.0) * params.spread;
    let golden = std::f32::consts::PI * (1.0 + 5.0_f32.sqrt());

    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let t = (i as f32 + 0.5) / node_count;
            let phi = (1.0 - 2.0 * t).acos();
            let theta = golden * i as f32;
            Node3D {
                id: node.id.clone(),
                label: node.title.clone(),
                state: node.state.clone(),
                x: radius * phi.sin() * theta.cos(),
                y: radius * phi.cos(),
                z: radius * phi.sin() * theta.sin(),
            }
        })
        .collect()
}

pub(super) fn layout_grid(
    nodes: &[SpecGraphNode],
    params: LayoutParams,
) -> Vec<Node3D> {
    let node_count = nodes.len();
    let columns = (node_count as f32).sqrt().ceil() as usize;
    let columns = columns.max(1);
    let rows = node_count.div_ceil(columns);
    let cell = 2.5 * params.spread;
    let half_width = (columns as f32 - 1.0) * 0.5 * cell;
    let half_depth = (rows as f32 - 1.0) * 0.5 * cell;

    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let column = i % columns;
            let row = i / columns;
            Node3D {
                id: node.id.clone(),
                label: node.title.clone(),
                state: node.state.clone(),
                x: column as f32 * cell - half_width,
                y: ((row % 2) as f32 - 0.5) * params.y_spacing,
                z: row as f32 * cell - half_depth,
            }
        })
        .collect()
}
