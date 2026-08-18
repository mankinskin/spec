use std::collections::{
    BTreeMap,
    HashMap,
    HashSet,
    VecDeque,
};

use viewer_api_dioxus::Node3D;

use crate::types::{
    SpecGraphEdge,
    SpecGraphNode,
};

use super::super::model::LayoutParams;

pub(super) fn layout_rings(
    nodes: &[SpecGraphNode],
    edges: &[SpecGraphEdge],
    params: LayoutParams,
) -> Vec<Node3D> {
    let index = node_index(nodes);
    let (children, has_parent) = collect_parent_edges(edges, &index);
    let depth = compute_depths(nodes.len(), &children, &has_parent);
    let by_depth = group_by_depth(&depth);
    build_ring_nodes(nodes, &by_depth, params)
}

fn node_index(nodes: &[SpecGraphNode]) -> HashMap<&str, usize> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect()
}

fn collect_parent_edges(
    edges: &[SpecGraphEdge],
    index: &HashMap<&str, usize>,
) -> (HashMap<usize, Vec<usize>>, HashSet<usize>) {
    let mut children = HashMap::new();
    let mut has_parent = HashSet::new();

    for edge in edges {
        if edge.kind != "parent" {
            continue;
        }

        let (Some(&from), Some(&to)) =
            (index.get(edge.from.as_str()), index.get(edge.to.as_str()))
        else {
            continue;
        };
        children.entry(from).or_insert_with(Vec::new).push(to);
        has_parent.insert(to);
    }

    (children, has_parent)
}

fn compute_depths(
    node_count: usize,
    children: &HashMap<usize, Vec<usize>>,
    has_parent: &HashSet<usize>,
) -> Vec<u32> {
    let mut depth = vec![u32::MAX; node_count];
    let mut queue = VecDeque::new();

    for index in 0..node_count {
        if has_parent.contains(&index) {
            continue;
        }
        depth[index] = 0;
        queue.push_back(index);
    }

    while let Some(parent) = queue.pop_front() {
        let next_depth = depth[parent] + 1;
        for &child in children.get(&parent).into_iter().flatten() {
            if depth[child] != u32::MAX {
                continue;
            }
            depth[child] = next_depth;
            queue.push_back(child);
        }
    }

    depth
        .into_iter()
        .map(|value| if value == u32::MAX { 0 } else { value })
        .collect()
}

fn group_by_depth(depth: &[u32]) -> BTreeMap<u32, Vec<usize>> {
    let mut groups = BTreeMap::new();
    for (index, &value) in depth.iter().enumerate() {
        groups.entry(value).or_insert_with(Vec::new).push(index);
    }
    groups
}

fn build_ring_nodes(
    nodes: &[SpecGraphNode],
    by_depth: &BTreeMap<u32, Vec<usize>>,
    params: LayoutParams,
) -> Vec<Node3D> {
    let mut out = vec![
        Node3D {
            id: String::new(),
            label: None,
            state: None,
            x: 0.0,
            y: 0.0,
            z: 0.0
        };
        nodes.len()
    ];

    for (&depth, members) in by_depth {
        let count = members.len() as f32;
        let radius = ((count * 0.55 + 2.5).min(18.0)) * params.spread;
        let z = -(depth as f32) * 4.5 * params.spread;
        for (slot, &index) in members.iter().enumerate() {
            let theta = if count > 0.0 {
                slot as f32 / count * std::f32::consts::TAU
            } else {
                0.0
            };
            let node = &nodes[index];
            out[index] = Node3D {
                id: node.id.clone(),
                label: node.title.clone(),
                state: node.state.clone(),
                x: radius * theta.cos(),
                y: ((slot % 3) as f32 - 1.0) * params.y_spacing,
                z,
            };
        }
    }

    out
}
