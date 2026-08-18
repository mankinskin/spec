use crate::types::SpecGraphNode;

pub(super) fn state_color(state: Option<&str>) -> &'static str {
    match state.unwrap_or("draft") {
        "draft" => "#9ca3af",
        "ready" => "#60a5fa",
        "reviewed" => "#f59e0b",
        "approved" => "#10b981",
        "archived" => "#6b7280",
        _ => "#a78bfa",
    }
}

pub(super) fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

pub(super) fn node_title(node: &SpecGraphNode) -> String {
    node.title
        .clone()
        .or_else(|| node.slug.clone())
        .unwrap_or_else(|| short_id(&node.id))
}

pub(super) fn node_summary(node: &SpecGraphNode) -> String {
    node.summary
        .clone()
        .unwrap_or_else(|| "No body summary yet.".to_string())
}

pub(super) fn metric_text(
    count: usize,
    singular: &str,
    plural: &str,
) -> String {
    let noun = if count == 1 { singular } else { plural };
    format!("{count} {noun}")
}
