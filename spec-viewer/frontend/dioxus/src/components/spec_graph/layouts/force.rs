use std::collections::HashMap;

use viewer_api_dioxus::{
    graph3d::camera::CAMERA_FOV,
    Node3D,
};

use crate::types::{
    SpecGraphEdge,
    SpecGraphNode,
};

use super::{
    super::model::LayoutParams,
    FrustumLayoutContext,
};

const FRUSTUM_FILL_RATIO: f32 = 0.92;
const FRUSTUM_SCALE_MIN: f32 = 0.72;
const FRUSTUM_SCALE_MAX: f32 = 1.8;
const FRUSTUM_CLEARANCE_PASSES: usize = 6;
const FRUSTUM_CARD_BASE_WIDTH_PX: f32 = 245.0;
const FRUSTUM_CARD_BASE_HEIGHT_PX: f32 = 196.0;
const FRUSTUM_EDGE_GUTTER_X_PX: f32 = 22.0;
const FRUSTUM_EDGE_GUTTER_Y_PX: f32 = 18.0;

pub(super) fn layout_force(
    nodes: &[SpecGraphNode],
    edges: &[SpecGraphEdge],
    params: LayoutParams,
    frustum_context: Option<&FrustumLayoutContext>,
) -> Vec<Node3D> {
    let node_count = nodes.len();
    let index = node_index(nodes);
    let mut positions = initial_positions(node_count, params.spread);
    let indexed_edges = indexed_edges(edges, &index);
    let k = params.link_dist.max(0.1);
    let mut temperature = 0.6_f32 * params.spread;
    let frustum_gravity_active = params.frustum_gravity_enabled
        && frustum_context.is_some()
        && params.frustum_gravity > 0.0;
    let extra_frustum_iterations = if frustum_gravity_active {
        (params.frustum_gravity * params.frustum_settle.max(0.0) * 60.0).round()
            as u32
    } else {
        0
    };

    for _ in 0..(params.iterations.max(1) + extra_frustum_iterations) {
        let mut displacement = vec![[0.0_f32; 3]; node_count];
        apply_repulsion(&positions, &mut displacement, params.repulsion, k);
        apply_attraction(&positions, &mut displacement, &indexed_edges, k);
        if frustum_gravity_active {
            let Some(context) = frustum_context else {
                continue;
            };
            apply_frustum_gravity(
                &positions,
                &mut displacement,
                params.frustum_gravity,
                params.frustum_overlap_repulsion,
                k,
                context,
            );
        }
        step_positions(&mut positions, &displacement, temperature);
        temperature *= 0.97;
    }

    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| Node3D {
            id: node.id.clone(),
            label: node.title.clone(),
            state: node.state.clone(),
            x: positions[i][0],
            y: positions[i][1],
            z: positions[i][2],
        })
        .collect()
}

fn node_index(nodes: &[SpecGraphNode]) -> HashMap<&str, usize> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect()
}

fn initial_positions(
    node_count: usize,
    spread: f32,
) -> Vec<[f32; 3]> {
    (0..node_count)
        .map(|i| {
            let phi = (1.0 - 2.0 * (i as f32 + 0.5) / node_count as f32).acos();
            let theta =
                std::f32::consts::PI * (1.0 + 5.0_f32.sqrt()) * i as f32;
            let radius = 4.0 * spread;
            [
                radius * phi.sin() * theta.cos(),
                radius * phi.sin() * theta.sin(),
                radius * phi.cos(),
            ]
        })
        .collect()
}

fn indexed_edges(
    edges: &[SpecGraphEdge],
    index: &HashMap<&str, usize>,
) -> Vec<(usize, usize)> {
    edges
        .iter()
        .filter_map(|edge| {
            let &from = index.get(edge.from.as_str())?;
            let &to = index.get(edge.to.as_str())?;
            Some((from, to))
        })
        .collect()
}

fn apply_repulsion(
    positions: &[[f32; 3]],
    displacement: &mut [[f32; 3]],
    repulsion: f32,
    k: f32,
) {
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let delta = [
                positions[i][0] - positions[j][0],
                positions[i][1] - positions[j][1],
                positions[i][2] - positions[j][2],
            ];
            let dist2 = (delta[0] * delta[0]
                + delta[1] * delta[1]
                + delta[2] * delta[2])
                .max(0.01);
            let dist = dist2.sqrt();
            let force = repulsion * k * k / dist2;
            let unit = [delta[0] / dist, delta[1] / dist, delta[2] / dist];
            for axis in 0..3 {
                displacement[i][axis] += unit[axis] * force;
                displacement[j][axis] -= unit[axis] * force;
            }
        }
    }
}

fn apply_attraction(
    positions: &[[f32; 3]],
    displacement: &mut [[f32; 3]],
    edges: &[(usize, usize)],
    k: f32,
) {
    for &(from, to) in edges {
        if from == to {
            continue;
        }
        let delta = [
            positions[from][0] - positions[to][0],
            positions[from][1] - positions[to][1],
            positions[from][2] - positions[to][2],
        ];
        let dist =
            (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2])
                .sqrt()
                .max(0.01);
        let force = dist * dist / k;
        let unit = [delta[0] / dist, delta[1] / dist, delta[2] / dist];
        for axis in 0..3 {
            displacement[from][axis] -= unit[axis] * force;
            displacement[to][axis] += unit[axis] * force;
        }
    }
}

fn step_positions(
    positions: &mut [[f32; 3]],
    displacement: &[[f32; 3]],
    temperature: f32,
) {
    for (position, delta) in positions.iter_mut().zip(displacement.iter()) {
        let magnitude =
            (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2])
                .sqrt()
                .max(0.001);
        let step = magnitude.min(temperature);
        for axis in 0..3 {
            position[axis] += delta[axis] / magnitude * step;
        }
    }
}

fn apply_frustum_gravity(
    positions: &[[f32; 3]],
    displacement: &mut [[f32; 3]],
    frustum_gravity: f32,
    frustum_overlap_repulsion: f32,
    k: f32,
    context: &FrustumLayoutContext,
) {
    if frustum_gravity <= 0.0 || positions.is_empty() {
        return;
    }

    let tan_half_fov = (CAMERA_FOV * 0.5).tan().max(0.001);
    let aspect = context.aspect.max(0.6);
    let mut basis = CameraBasis::from_context(context);
    basis.eye =
        frustum_gravity_virtual_eye(positions, &basis, tan_half_fov, aspect);
    let mut ndc_positions = Vec::with_capacity(positions.len());
    let mut min_ndc_x = f32::MAX;
    let mut max_ndc_x = f32::MIN;
    let mut min_ndc_y = f32::MAX;
    let mut max_ndc_y = f32::MIN;
    let mut sum_depth = 0.0_f32;

    for &position in positions {
        let camera_position = world_to_camera_space(position, &basis);
        let ndc_position = camera_to_ndc(camera_position, tan_half_fov, aspect);
        min_ndc_x = min_ndc_x.min(ndc_position[0]);
        max_ndc_x = max_ndc_x.max(ndc_position[0]);
        min_ndc_y = min_ndc_y.min(ndc_position[1]);
        max_ndc_y = max_ndc_y.max(ndc_position[1]);
        sum_depth += camera_position[2];
        ndc_positions.push(ndc_position);
    }

    let count = positions.len() as f32;
    let center_ndc_x = (min_ndc_x + max_ndc_x) * 0.5;
    let center_ndc_y = (min_ndc_y + max_ndc_y) * 0.5;
    let half_ndc_w = ((max_ndc_x - min_ndc_x) * 0.5).max(0.001);
    let half_ndc_h = ((max_ndc_y - min_ndc_y) * 0.5).max(0.001);
    let scale_x = (FRUSTUM_FILL_RATIO / half_ndc_w)
        .clamp(FRUSTUM_SCALE_MIN, FRUSTUM_SCALE_MAX);
    let scale_y = (FRUSTUM_FILL_RATIO / half_ndc_h)
        .clamp(FRUSTUM_SCALE_MIN, FRUSTUM_SCALE_MAX);
    let fill_ratio = (half_ndc_w / FRUSTUM_FILL_RATIO)
        .max(half_ndc_h / FRUSTUM_FILL_RATIO)
        .clamp(0.55, 1.8);
    let viewport_width = context.viewport_width.max(320.0);
    let viewport_height = context.viewport_height.max(240.0);
    let mut target_depth = (sum_depth / count).max(0.1) * fill_ratio;
    let mut fill_ndc_positions = Vec::with_capacity(ndc_positions.len());
    for ndc_position in &ndc_positions {
        fill_ndc_positions.push([
            (ndc_position[0] - center_ndc_x) * scale_x,
            (ndc_position[1] - center_ndc_y) * scale_y,
        ]);
    }
    let mut clearance_ndc_positions = fill_ndc_positions.clone();
    relax_projected_clearance(
        clearance_ndc_positions.as_mut_slice(),
        &mut target_depth,
        viewport_width,
        viewport_height,
        tan_half_fov,
        aspect,
    );
    apply_projected_overlap_repulsion(
        positions,
        displacement,
        frustum_gravity * 1.8 * frustum_overlap_repulsion.max(0.0),
        &basis,
        viewport_width,
        viewport_height,
        tan_half_fov,
        aspect,
    );
    let fill_spring = frustum_gravity * k.max(0.1) * 0.03;
    let clearance_spring = frustum_gravity * k.max(0.1) * 0.32;

    for (index, position) in positions.iter().enumerate() {
        let fill_camera_position = ndc_to_camera(
            fill_ndc_positions[index],
            target_depth,
            tan_half_fov,
            aspect,
        );
        let clearance_camera_position = ndc_to_camera(
            clearance_ndc_positions[index],
            target_depth,
            tan_half_fov,
            aspect,
        );
        let fill_world = camera_to_world_space(fill_camera_position, &basis);
        let clearance_world =
            camera_to_world_space(clearance_camera_position, &basis);
        let fill_delta = [
            fill_world[0] - position[0],
            fill_world[1] - position[1],
            fill_world[2] - position[2],
        ];
        let clearance_delta = [
            clearance_world[0] - fill_world[0],
            clearance_world[1] - fill_world[1],
            clearance_world[2] - fill_world[2],
        ];
        for axis in 0..3 {
            displacement[index][axis] += fill_delta[axis] * fill_spring;
            displacement[index][axis] +=
                clearance_delta[axis] * clearance_spring;
        }
    }
}

fn apply_projected_overlap_repulsion(
    positions: &[[f32; 3]],
    displacement: &mut [[f32; 3]],
    strength: f32,
    basis: &CameraBasis,
    viewport_width: f32,
    viewport_height: f32,
    tan_half_fov: f32,
    aspect: f32,
) {
    if strength <= 0.0 || positions.len() < 2 {
        return;
    }

    let mut camera_positions = Vec::with_capacity(positions.len());
    let mut screen_positions = Vec::with_capacity(positions.len());
    let mut half_extents = Vec::with_capacity(positions.len());

    for &position in positions {
        let camera_position = world_to_camera_space(position, basis);
        let ndc = camera_to_ndc(camera_position, tan_half_fov, aspect);
        camera_positions.push(camera_position);
        screen_positions.push(ndc_to_screen_pixels(
            ndc,
            viewport_width,
            viewport_height,
        ));
        half_extents.push(projected_card_half_extents_px(camera_position));
    }

    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let dx = screen_positions[j][0] - screen_positions[i][0];
            let dy = screen_positions[j][1] - screen_positions[i][1];
            let overlap_x = half_extents[i][0]
                + half_extents[j][0]
                + FRUSTUM_EDGE_GUTTER_X_PX
                - dx.abs();
            let overlap_y = half_extents[i][1]
                + half_extents[j][1]
                + FRUSTUM_EDGE_GUTTER_Y_PX
                - dy.abs();

            if overlap_x <= 0.0 || overlap_y <= 0.0 {
                continue;
            }

            let avg_depth = ((camera_positions[i][2] + camera_positions[j][2])
                * 0.5)
                .max(0.1);
            if overlap_x < overlap_y {
                let direction = if dx >= 0.0 { 1.0 } else { -1.0 };
                let push_camera = screen_px_to_camera_x(
                    overlap_x * 0.55,
                    avg_depth,
                    viewport_width,
                    tan_half_fov,
                    aspect,
                ) * strength;
                for axis in 0..3 {
                    let world_push =
                        basis.right[axis] * push_camera * direction;
                    displacement[i][axis] -= world_push;
                    displacement[j][axis] += world_push;
                }
            } else {
                let direction = if dy >= 0.0 { 1.0 } else { -1.0 };
                let push_camera = screen_px_to_camera_y(
                    overlap_y * 0.55,
                    avg_depth,
                    viewport_height,
                    tan_half_fov,
                ) * strength;
                for axis in 0..3 {
                    let world_push = basis.up[axis] * push_camera * direction;
                    displacement[i][axis] -= world_push;
                    displacement[j][axis] += world_push;
                }
            }
        }
    }
}

fn relax_projected_clearance(
    ndc_positions: &mut [[f32; 2]],
    depth: &mut f32,
    viewport_width: f32,
    viewport_height: f32,
    tan_half_fov: f32,
    aspect: f32,
) {
    if ndc_positions.len() < 2 {
        return;
    }

    for _ in 0..FRUSTUM_CLEARANCE_PASSES {
        let mut pixel_offsets = vec![[0.0_f32; 2]; ndc_positions.len()];
        let mut max_depth_scale = 1.0_f32;
        let mut had_overlap = false;

        for i in 0..ndc_positions.len() {
            let camera_i =
                ndc_to_camera(ndc_positions[i], *depth, tan_half_fov, aspect);
            let screen_i = ndc_to_screen_pixels(
                ndc_positions[i],
                viewport_width,
                viewport_height,
            );
            let half_i = projected_card_half_extents_px(camera_i);

            for j in (i + 1)..ndc_positions.len() {
                let camera_j = ndc_to_camera(
                    ndc_positions[j],
                    *depth,
                    tan_half_fov,
                    aspect,
                );
                let screen_j = ndc_to_screen_pixels(
                    ndc_positions[j],
                    viewport_width,
                    viewport_height,
                );
                let half_j = projected_card_half_extents_px(camera_j);
                let dx = screen_j[0] - screen_i[0];
                let dy = screen_j[1] - screen_i[1];
                let overlap_x =
                    half_i[0] + half_j[0] + FRUSTUM_EDGE_GUTTER_X_PX - dx.abs();
                let overlap_y =
                    half_i[1] + half_j[1] + FRUSTUM_EDGE_GUTTER_Y_PX - dy.abs();

                if overlap_x <= 0.0 || overlap_y <= 0.0 {
                    continue;
                }

                had_overlap = true;
                max_depth_scale = max_depth_scale.max(required_depth_scale(
                    half_i[0] + half_j[0],
                    dx.abs(),
                    FRUSTUM_EDGE_GUTTER_X_PX,
                ));
                max_depth_scale = max_depth_scale.max(required_depth_scale(
                    half_i[1] + half_j[1],
                    dy.abs(),
                    FRUSTUM_EDGE_GUTTER_Y_PX,
                ));

                if overlap_x < overlap_y {
                    let direction = if dx >= 0.0 { 1.0 } else { -1.0 };
                    let push = overlap_x.min(96.0) * 0.5;
                    pixel_offsets[i][0] -= direction * push;
                    pixel_offsets[j][0] += direction * push;
                } else {
                    let direction = if dy >= 0.0 { 1.0 } else { -1.0 };
                    let push = overlap_y.min(96.0) * 0.5;
                    pixel_offsets[i][1] -= direction * push;
                    pixel_offsets[j][1] += direction * push;
                }
            }
        }

        if max_depth_scale > 1.02 {
            *depth *= max_depth_scale.clamp(1.0, 1.35);
        }

        if !had_overlap {
            break;
        }

        let mut max_adjustment = 0.0_f32;
        for (ndc_position, pixel_offset) in
            ndc_positions.iter_mut().zip(pixel_offsets.iter())
        {
            let ndc_delta_x =
                pixel_offset[0].clamp(-96.0, 96.0) * 2.0 / viewport_width;
            let ndc_delta_y =
                pixel_offset[1].clamp(-96.0, 96.0) * 2.0 / viewport_height;
            ndc_position[0] += ndc_delta_x;
            ndc_position[1] += ndc_delta_y;
            max_adjustment =
                max_adjustment.max(ndc_delta_x.abs()).max(ndc_delta_y.abs());
        }
        recenter_ndc_positions(ndc_positions);

        if max_adjustment < 0.001 && max_depth_scale <= 1.02 {
            break;
        }
    }
}

fn recenter_ndc_positions(ndc_positions: &mut [[f32; 2]]) {
    if ndc_positions.is_empty() {
        return;
    }

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for position in ndc_positions.iter() {
        min_x = min_x.min(position[0]);
        max_x = max_x.max(position[0]);
        min_y = min_y.min(position[1]);
        max_y = max_y.max(position[1]);
    }

    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;
    for position in ndc_positions.iter_mut() {
        position[0] -= center_x;
        position[1] -= center_y;
    }
}

fn required_depth_scale(
    card_span_px: f32,
    separation_px: f32,
    gutter_px: f32,
) -> f32 {
    let remaining = separation_px - gutter_px;
    if remaining <= 1.0 {
        return 1.25;
    }
    (card_span_px / remaining).max(1.0)
}

fn ndc_to_screen_pixels(
    ndc_position: [f32; 2],
    viewport_width: f32,
    viewport_height: f32,
) -> [f32; 2] {
    [
        ndc_position[0] * viewport_width * 0.5,
        ndc_position[1] * viewport_height * 0.5,
    ]
}

fn screen_px_to_camera_x(
    pixels: f32,
    depth: f32,
    viewport_width: f32,
    tan_half_fov: f32,
    aspect: f32,
) -> f32 {
    (pixels * 2.0 / viewport_width) * depth * tan_half_fov * aspect
}

fn screen_px_to_camera_y(
    pixels: f32,
    depth: f32,
    viewport_height: f32,
    tan_half_fov: f32,
) -> f32 {
    (pixels * 2.0 / viewport_height) * depth * tan_half_fov
}

fn projected_card_half_extents_px(camera_position: [f32; 3]) -> [f32; 2] {
    let distance = (camera_position[0] * camera_position[0]
        + camera_position[1] * camera_position[1]
        + camera_position[2] * camera_position[2])
        .sqrt()
        .max(0.1);
    let pixel_scale = (22.0 / distance).clamp(0.14, 3.5);
    [
        FRUSTUM_CARD_BASE_WIDTH_PX * pixel_scale * 0.5,
        FRUSTUM_CARD_BASE_HEIGHT_PX * pixel_scale * 0.5,
    ]
}

fn frustum_gravity_virtual_eye(
    positions: &[[f32; 3]],
    basis: &CameraBasis,
    tan_half_fov: f32,
    aspect: f32,
) -> [f32; 3] {
    let origin = positions_centroid(positions);
    let mut max_right = 0.0_f32;
    let mut max_up = 0.0_f32;
    let mut min_forward = f32::MAX;

    for &position in positions {
        let delta = [
            position[0] - origin[0],
            position[1] - origin[1],
            position[2] - origin[2],
        ];
        max_right = max_right.max(dot(delta, basis.right).abs());
        max_up = max_up.max(dot(delta, basis.up).abs());
        min_forward = min_forward.min(dot(delta, basis.forward));
    }

    let required_depth_x =
        max_right / (FRUSTUM_FILL_RATIO * tan_half_fov * aspect).max(0.001);
    let required_depth_y =
        max_up / (FRUSTUM_FILL_RATIO * tan_half_fov).max(0.001);
    let virtual_distance = required_depth_x
        .max(required_depth_y)
        .max((-min_forward).max(0.0) + 1.0)
        .max(1.0);

    [
        origin[0] - basis.forward[0] * virtual_distance,
        origin[1] - basis.forward[1] * virtual_distance,
        origin[2] - basis.forward[2] * virtual_distance,
    ]
}

fn positions_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    let mut centroid = [0.0_f32; 3];
    for &position in positions {
        centroid[0] += position[0];
        centroid[1] += position[1];
        centroid[2] += position[2];
    }

    let count = positions.len() as f32;
    [
        centroid[0] / count,
        centroid[1] / count,
        centroid[2] / count,
    ]
}

struct CameraBasis {
    eye: [f32; 3],
    forward: [f32; 3],
    right: [f32; 3],
    up: [f32; 3],
}

impl CameraBasis {
    fn from_context(context: &FrustumLayoutContext) -> Self {
        let eye = context.camera.eye();
        let forward = normalise([
            context.camera.target[0] - eye[0],
            context.camera.target[1] - eye[1],
            context.camera.target[2] - eye[2],
        ]);
        let right = normalise(cross(forward, [0.0, 1.0, 0.0]));
        let up = normalise(cross(right, forward));
        Self {
            eye,
            forward,
            right,
            up,
        }
    }
}

fn world_to_camera_space(
    position: [f32; 3],
    basis: &CameraBasis,
) -> [f32; 3] {
    let delta = [
        position[0] - basis.eye[0],
        position[1] - basis.eye[1],
        position[2] - basis.eye[2],
    ];
    [
        dot(delta, basis.right),
        dot(delta, basis.up),
        dot(delta, basis.forward).max(0.1),
    ]
}

fn camera_to_world_space(
    position: [f32; 3],
    basis: &CameraBasis,
) -> [f32; 3] {
    [
        basis.eye[0]
            + basis.right[0] * position[0]
            + basis.up[0] * position[1]
            + basis.forward[0] * position[2],
        basis.eye[1]
            + basis.right[1] * position[0]
            + basis.up[1] * position[1]
            + basis.forward[1] * position[2],
        basis.eye[2]
            + basis.right[2] * position[0]
            + basis.up[2] * position[1]
            + basis.forward[2] * position[2],
    ]
}

fn camera_to_ndc(
    camera_position: [f32; 3],
    tan_half_fov: f32,
    aspect: f32,
) -> [f32; 2] {
    [
        camera_position[0] / (camera_position[2] * tan_half_fov * aspect),
        camera_position[1] / (camera_position[2] * tan_half_fov),
    ]
}

fn ndc_to_camera(
    ndc_position: [f32; 2],
    depth: f32,
    tan_half_fov: f32,
    aspect: f32,
) -> [f32; 3] {
    [
        ndc_position[0] * depth * tan_half_fov * aspect,
        ndc_position[1] * depth * tan_half_fov,
        depth,
    ]
}

fn normalise(vector: [f32; 3]) -> [f32; 3] {
    let length =
        (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2])
            .sqrt();
    if length < 1e-6 {
        return [0.0, 0.0, 1.0];
    }
    [vector[0] / length, vector[1] / length, vector[2] / length]
}

fn cross(
    lhs: [f32; 3],
    rhs: [f32; 3],
) -> [f32; 3] {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}

fn dot(
    lhs: [f32; 3],
    rhs: [f32; 3],
) -> f32 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

#[cfg(test)]
mod tests {
    use viewer_api_dioxus::Camera;

    use super::*;

    #[test]
    fn frustum_gravity_reduces_camera_depth_variance() {
        let context = FrustumLayoutContext {
            camera: Camera {
                yaw: 0.0,
                pitch: 0.0,
                distance: 10.0,
                target: [0.0, 0.0, 0.0],
            },
            aspect: 16.0 / 9.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
        };
        let mut positions = vec![[0.0, 0.0, 0.0], [0.0, 0.0, -6.0]];
        let mut displacement = vec![[0.0_f32; 3]; positions.len()];

        let before = depth_range(&positions, &context);
        apply_frustum_gravity(
            &positions,
            &mut displacement,
            1.0,
            1.0,
            6.0,
            &context,
        );
        step_positions(&mut positions, &displacement, 8.0);
        let after = depth_range(&positions, &context);

        assert!(after < before);
    }

    #[test]
    fn frustum_gravity_increases_projected_fill() {
        let context = FrustumLayoutContext {
            camera: Camera {
                yaw: 0.0,
                pitch: 0.0,
                distance: 12.0,
                target: [0.0, 0.0, 0.0],
            },
            aspect: 16.0 / 9.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
        };
        let mut positions = vec![
            [-1.0, -0.8, 0.0],
            [1.0, -0.8, 0.0],
            [-1.0, 0.8, 0.0],
            [1.0, 0.8, 0.0],
        ];
        let mut displacement = vec![[0.0_f32; 3]; positions.len()];

        let before = projected_fill(&positions, &context);
        apply_frustum_gravity(
            &positions,
            &mut displacement,
            1.0,
            1.0,
            6.0,
            &context,
        );
        step_positions(&mut positions, &displacement, 8.0);
        let after = projected_fill(&positions, &context);

        assert!(after > before);
    }

    #[test]
    fn frustum_gravity_reduces_projected_card_overlap() {
        let context = FrustumLayoutContext {
            camera: Camera {
                yaw: 0.0,
                pitch: 0.0,
                distance: 12.0,
                target: [0.0, 0.0, 0.0],
            },
            aspect: 16.0 / 9.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
        };
        let mut positions = vec![
            [-0.22, -0.12, 0.0],
            [-0.08, -0.03, 0.0],
            [0.05, 0.04, 0.0],
            [0.18, 0.11, 0.0],
        ];

        let before = max_projected_overlap_px(&positions, &context);
        for _ in 0..12 {
            let mut displacement = vec![[0.0_f32; 3]; positions.len()];
            apply_frustum_gravity(
                &positions,
                &mut displacement,
                1.0,
                1.0,
                6.0,
                &context,
            );
            step_positions(&mut positions, &displacement, 3.0);
        }
        let after = max_projected_overlap_px(&positions, &context);

        assert!(after < before);
        assert!(after <= 1.0, "remaining projected overlap: {after}");
    }

    #[test]
    fn frustum_gravity_is_zoom_invariant_for_same_view_direction() {
        let near_context = FrustumLayoutContext {
            camera: Camera {
                yaw: 0.45,
                pitch: -0.3,
                distance: 8.0,
                target: [0.0, 0.0, 0.0],
            },
            aspect: 16.0 / 9.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
        };
        let far_context = FrustumLayoutContext {
            camera: Camera {
                distance: 28.0,
                ..near_context.camera
            },
            ..near_context.clone()
        };
        let positions = vec![
            [-3.2, -1.4, 0.5],
            [-1.0, 2.2, -1.7],
            [0.7, -0.6, 2.8],
            [2.6, 1.1, -0.9],
        ];
        let mut near_displacement = vec![[0.0_f32; 3]; positions.len()];
        let mut far_displacement = vec![[0.0_f32; 3]; positions.len()];

        apply_frustum_gravity(
            &positions,
            &mut near_displacement,
            1.0,
            1.0,
            6.0,
            &near_context,
        );
        apply_frustum_gravity(
            &positions,
            &mut far_displacement,
            1.0,
            1.0,
            6.0,
            &far_context,
        );

        for (near_delta, far_delta) in
            near_displacement.iter().zip(far_displacement.iter())
        {
            for axis in 0..3 {
                assert!(
                    (near_delta[axis] - far_delta[axis]).abs() < 1e-4,
                    "axis {axis} diverged: near={:?} far={:?}",
                    near_delta,
                    far_delta,
                );
            }
        }
    }

    #[test]
    fn frustum_gravity_zero_disables_force() {
        let context = FrustumLayoutContext {
            camera: Camera {
                yaw: 0.2,
                pitch: -0.1,
                distance: 12.0,
                target: [0.0, 0.0, 0.0],
            },
            aspect: 16.0 / 9.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
        };
        let positions =
            vec![[0.0, 0.0, 0.0], [1.2, -0.8, 0.6], [-0.9, 0.5, -1.1]];
        let mut displacement = vec![[0.0_f32; 3]; positions.len()];

        apply_frustum_gravity(
            &positions,
            &mut displacement,
            0.0,
            1.0,
            6.0,
            &context,
        );

        for delta in displacement {
            assert_eq!(delta, [0.0, 0.0, 0.0]);
        }
    }

    fn depth_range(
        positions: &[[f32; 3]],
        context: &FrustumLayoutContext,
    ) -> f32 {
        let basis = CameraBasis::from_context(context);
        let mut min_depth = f32::MAX;
        let mut max_depth = f32::MIN;
        for &position in positions {
            let depth = world_to_camera_space(position, &basis)[2];
            min_depth = min_depth.min(depth);
            max_depth = max_depth.max(depth);
        }
        max_depth - min_depth
    }

    fn projected_fill(
        positions: &[[f32; 3]],
        context: &FrustumLayoutContext,
    ) -> f32 {
        let basis = CameraBasis::from_context(context);
        let tan_half_fov = (CAMERA_FOV * 0.5).tan();
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for &position in positions {
            let ndc = camera_to_ndc(
                world_to_camera_space(position, &basis),
                tan_half_fov,
                context.aspect,
            );
            min_x = min_x.min(ndc[0]);
            max_x = max_x.max(ndc[0]);
            min_y = min_y.min(ndc[1]);
            max_y = max_y.max(ndc[1]);
        }

        ((max_x - min_x) * (max_y - min_y)).abs()
    }

    fn max_projected_overlap_px(
        positions: &[[f32; 3]],
        context: &FrustumLayoutContext,
    ) -> f32 {
        let basis = CameraBasis::from_context(context);
        let tan_half_fov = (CAMERA_FOV * 0.5).tan();
        let mut screen_positions = Vec::with_capacity(positions.len());
        let mut half_extents = Vec::with_capacity(positions.len());

        for &position in positions {
            let camera_position = world_to_camera_space(position, &basis);
            let ndc =
                camera_to_ndc(camera_position, tan_half_fov, context.aspect);
            screen_positions.push(ndc_to_screen_pixels(
                ndc,
                context.viewport_width,
                context.viewport_height,
            ));
            half_extents.push(projected_card_half_extents_px(camera_position));
        }

        let mut max_overlap = 0.0_f32;
        for i in 0..screen_positions.len() {
            for j in (i + 1)..screen_positions.len() {
                let dx =
                    (screen_positions[j][0] - screen_positions[i][0]).abs();
                let dy =
                    (screen_positions[j][1] - screen_positions[i][1]).abs();
                let overlap_x = half_extents[i][0]
                    + half_extents[j][0]
                    + FRUSTUM_EDGE_GUTTER_X_PX
                    - dx;
                let overlap_y = half_extents[i][1]
                    + half_extents[j][1]
                    + FRUSTUM_EDGE_GUTTER_Y_PX
                    - dy;
                if overlap_x > 0.0 && overlap_y > 0.0 {
                    max_overlap = max_overlap.max(overlap_x.min(overlap_y));
                }
            }
        }

        max_overlap
    }
}
