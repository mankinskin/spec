use std::collections::HashMap;

use viewer_api_dioxus::{
    Camera,
    EdgeRef3D,
    Layout3D,
};

use crate::types::{
    SpecGraphEdge,
    SpecGraphNode,
};

use super::model::{
    LayoutAlgorithm,
    LayoutParams,
};

mod force;
mod rings;
mod simple;
mod tree;

use force::layout_force;
use rings::layout_rings;
use simple::{
    layout_grid,
    layout_sphere,
};
use tree::layout_tree_2d;

#[derive(Clone, Debug, PartialEq)]
pub struct FrustumLayoutContext {
    pub camera: Camera,
    pub aspect: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

pub fn build_layout(
    algo: LayoutAlgorithm,
    params: LayoutParams,
    nodes: &[SpecGraphNode],
    edges: &[SpecGraphEdge],
    frustum_context: Option<FrustumLayoutContext>,
) -> Layout3D {
    if nodes.is_empty() {
        return Layout3D::default();
    }

    let positioned = match algo {
        LayoutAlgorithm::RingsByDepth => layout_rings(nodes, edges, params),
        LayoutAlgorithm::ForceDirected =>
            layout_force(nodes, edges, params, frustum_context.as_ref()),
        LayoutAlgorithm::Sphere => layout_sphere(nodes, params),
        LayoutAlgorithm::Grid => layout_grid(nodes, params),
        LayoutAlgorithm::Tree2D => layout_tree_2d(nodes, edges, params),
    };

    let index: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect();

    let out_edges = edges
        .iter()
        .filter_map(|edge| {
            let &from_idx = index.get(edge.from.as_str())?;
            let &to_idx = index.get(edge.to.as_str())?;
            Some(EdgeRef3D {
                from_idx,
                to_idx,
                kind: edge.kind.clone(),
            })
        })
        .collect();

    Layout3D::new(positioned, out_edges)
}
