use std::collections::HashMap;

use viewer_api_dioxus::Node3D;

use crate::types::{
    SpecGraphEdge,
    SpecGraphNode,
};

use super::super::model::LayoutParams;

pub(super) fn layout_tree_2d(
    nodes: &[SpecGraphNode],
    edges: &[SpecGraphEdge],
    params: LayoutParams,
) -> Vec<Node3D> {
    let index = node_index(nodes);
    let (children, has_parent) = build_tree(edges, &index, nodes.len());
    let mut roots = root_nodes(&has_parent);
    let (width, depth, mut visited) = measure_forest(&roots, &children);
    append_cycle_members(&mut roots, &mut visited);
    let x_pos = assign_forest(&roots, &children, &width);
    build_tree_nodes(nodes, &roots, &depth, &width, &x_pos, params)
}

fn node_index(nodes: &[SpecGraphNode]) -> HashMap<&str, usize> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect()
}

fn build_tree(
    edges: &[SpecGraphEdge],
    index: &HashMap<&str, usize>,
    node_count: usize,
) -> (Vec<Vec<usize>>, Vec<bool>) {
    let mut children = vec![Vec::new(); node_count];
    let mut has_parent = vec![false; node_count];

    for edge in edges {
        if edge.kind != "parent" {
            continue;
        }

        let (Some(&from), Some(&to)) =
            (index.get(edge.from.as_str()), index.get(edge.to.as_str()))
        else {
            continue;
        };
        if from == to {
            continue;
        }

        children[from].push(to);
        has_parent[to] = true;
    }

    for branch in &mut children {
        branch.sort();
    }

    (children, has_parent)
}

fn root_nodes(has_parent: &[bool]) -> Vec<usize> {
    let mut roots: Vec<usize> = has_parent
        .iter()
        .enumerate()
        .filter_map(|(index, &value)| (!value).then_some(index))
        .collect();
    roots.sort();
    roots
}

fn measure_forest(
    roots: &[usize],
    children: &[Vec<usize>],
) -> (Vec<f32>, Vec<u32>, Vec<bool>) {
    let mut width = vec![1.0_f32; children.len()];
    let mut depth = vec![0_u32; children.len()];
    let mut visited = vec![false; children.len()];

    for &root in roots {
        measure_node(root, 0, children, &mut width, &mut depth, &mut visited);
    }

    (width, depth, visited)
}

fn measure_node(
    node: usize,
    depth_value: u32,
    children: &[Vec<usize>],
    width: &mut [f32],
    depth: &mut [u32],
    visited: &mut [bool],
) {
    if visited[node] {
        return;
    }

    visited[node] = true;
    depth[node] = depth_value;
    let mut sum = 0.0_f32;
    for &child in &children[node] {
        measure_node(child, depth_value + 1, children, width, depth, visited);
        sum += width[child];
    }

    if !children[node].is_empty() {
        width[node] = sum.max(1.0);
    }
}

fn append_cycle_members(
    roots: &mut Vec<usize>,
    visited: &mut [bool],
) {
    for (index, seen) in visited.iter_mut().enumerate() {
        if *seen {
            continue;
        }
        roots.push(index);
        *seen = true;
    }
}

fn assign_forest(
    roots: &[usize],
    children: &[Vec<usize>],
    width: &[f32],
) -> Vec<f32> {
    let mut x_pos = vec![0.0_f32; children.len()];
    let mut cursor = 0.0_f32;
    for &root in roots {
        assign_node(root, cursor, children, width, &mut x_pos);
        cursor += width[root];
    }
    x_pos
}

fn assign_node(
    node: usize,
    slot_start: f32,
    children: &[Vec<usize>],
    width: &[f32],
    x_pos: &mut [f32],
) {
    x_pos[node] = slot_start + width[node] * 0.5;
    let mut cursor = slot_start;
    for &child in &children[node] {
        assign_node(child, cursor, children, width, x_pos);
        cursor += width[child];
    }
}

fn build_tree_nodes(
    nodes: &[SpecGraphNode],
    roots: &[usize],
    depth: &[u32],
    width: &[f32],
    x_pos: &[f32],
    params: LayoutParams,
) -> Vec<Node3D> {
    let total_width: f32 = roots.iter().map(|&root| width[root]).sum();
    let center_offset = total_width * 0.5;
    let col_step = 2.5 * params.spread;
    let row_step = 2.5 * params.spread.max(0.4);

    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| Node3D {
            id: node.id.clone(),
            label: node.title.clone(),
            state: node.state.clone(),
            x: (x_pos[i] - center_offset) * col_step,
            y: 0.0,
            z: depth[i] as f32 * row_step,
        })
        .collect()
}
