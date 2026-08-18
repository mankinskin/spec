use viewer_api_dioxus::CameraCommand;

pub const SELECTED_NODE_ZOOM_FACTOR_MIN: f32 = 1.0;
pub const SELECTED_NODE_ZOOM_FACTOR_MAX: f32 = 3.0;
pub const SELECTED_NODE_ZOOM_FACTOR_DEFAULT: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutAlgorithm {
    RingsByDepth,
    ForceDirected,
    Sphere,
    Grid,
    Tree2D,
}

impl LayoutAlgorithm {
    pub const ALL: &'static [LayoutAlgorithm] = &[
        LayoutAlgorithm::RingsByDepth,
        LayoutAlgorithm::ForceDirected,
        LayoutAlgorithm::Sphere,
        LayoutAlgorithm::Grid,
        LayoutAlgorithm::Tree2D,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LayoutAlgorithm::RingsByDepth => "Rings by depth",
            LayoutAlgorithm::ForceDirected => "Force-directed",
            LayoutAlgorithm::Sphere => "Sphere",
            LayoutAlgorithm::Grid => "Grid",
            LayoutAlgorithm::Tree2D => "2D Tree",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<Self> {
        match value {
            "rings" => Some(LayoutAlgorithm::RingsByDepth),
            "force" => Some(LayoutAlgorithm::ForceDirected),
            "sphere" => Some(LayoutAlgorithm::Sphere),
            "grid" => Some(LayoutAlgorithm::Grid),
            "tree2d" => Some(LayoutAlgorithm::Tree2D),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LayoutAlgorithm::RingsByDepth => "rings",
            LayoutAlgorithm::ForceDirected => "force",
            LayoutAlgorithm::Sphere => "sphere",
            LayoutAlgorithm::Grid => "grid",
            LayoutAlgorithm::Tree2D => "tree2d",
        }
    }

    pub fn preferred_camera(self) -> CameraCommand {
        match self {
            LayoutAlgorithm::Tree2D => CameraCommand::ResetTo {
                yaw: 0.0,
                pitch: std::f32::consts::FRAC_PI_2 - 0.001,
            },
            _ => CameraCommand::ResetToDefault,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutParams {
    pub spread: f32,
    pub y_spacing: f32,
    pub iterations: u32,
    pub link_dist: f32,
    pub repulsion: f32,
    pub frustum_gravity_enabled: bool,
    pub frustum_gravity: f32,
    pub frustum_overfill: f32,
    pub frustum_settle: f32,
    pub frustum_overlap_repulsion: f32,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            spread: 2.5,
            y_spacing: 1.0,
            iterations: 120,
            link_dist: 6.0,
            repulsion: 6.5,
            frustum_gravity_enabled: true,
            frustum_gravity: 2.0,
            frustum_overfill: 1.35,
            frustum_settle: 2.0,
            frustum_overlap_repulsion: 1.45,
        }
    }
}
