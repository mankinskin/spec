//! API request and response types for the spec viewer.
//!
//! Mirrors the JSON shapes returned by `spec-http` REST endpoints.

use serde::Deserialize;

// ── Spec summary (list endpoint) ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SpecSummary {
    pub id: String,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub state: Option<String>,
    pub component: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SpecListResponse {
    pub count: usize,
    pub items: Vec<SpecSummary>,
}

// ── Spec detail ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CodeRefEntry {
    pub file: String,
    pub symbol: String,
    pub kind: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub line_start: Option<u32>,
    #[serde(default)]
    pub line_end: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpecDetail {
    pub id: String,
    pub created_at: String,
    pub fields: serde_json::Value,
    #[serde(default)]
    pub code_refs: Vec<CodeRefEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SpecDetailResponse {
    pub spec: SpecDetail,
}

// ── Spec full (with body and sections list) ───────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SpecFullResponse {
    pub spec: SpecDetail,
    pub body: String,
    pub sections: Vec<String>,
}

// ── Sections ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SectionsResponse {
    pub spec: String,
    pub count: usize,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SectionResponse {
    pub spec: String,
    pub name: String,
    pub content: String,
}

// ── Tree ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(dead_code)]
pub struct TreeNode {
    pub id: String,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub state: Option<String>,
    #[serde(default)]
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SpecTreeResponse {
    pub root: TreeNode,
    pub descendants: Vec<TreeNode>,
}

// ── Search ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SearchResponse {
    pub count: usize,
    pub items: Vec<SpecSummary>,
}

// ── Health ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HealthIssue {
    pub id: String,
    pub issue: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HealthResponse {
    pub specs_checked: usize,
    pub issues_count: usize,
    pub issues: Vec<serde_json::Value>,
}

// ── Graph ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SpecGraphMetrics {
    pub child_count: usize,
    pub code_ref_count: usize,
    pub section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SpecGraphNode {
    pub id: String,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub state: Option<String>,
    pub component: Option<String>,
    pub scope: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub summary_markdown: Option<String>,
    pub metrics: SpecGraphMetrics,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SpecGraphEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpecGraphResponse {
    pub nodes: Vec<SpecGraphNode>,
    pub edges: Vec<SpecGraphEdge>,
}

// ── SSE ───────────────────────────────────────────────────────────────────────

/// Minimal spec snapshot embedded in `spec.updated` SSE events.
#[derive(Debug, Clone, Deserialize)]
pub struct SseSpec {
    pub id: String,
    pub state: Option<String>,
    pub title: Option<String>,
    pub slug: Option<String>,
}

// ── State colours ─────────────────────────────────────────────────────────────

/// Returns `(background, foreground)` CSS colour pair for a state badge.
#[allow(dead_code)]
pub fn state_colors(state: &str) -> (&'static str, &'static str) {
    match state {
        "draft" => ("#4b5563", "#d1d5db"),
        "ready" => ("#1e40af", "#bfdbfe"),
        "reviewed" => ("#15803d", "#bbf7d0"),
        "approved" => ("#065f46", "#6ee7b7"),
        "archived" => ("#374151", "#9ca3af"),
        _ => ("#374151", "#9ca3af"),
    }
}

/// Returns a CSS accent colour for a state (used for left-border / ring).
#[allow(dead_code)]
pub fn state_accent(state: Option<&str>) -> &'static str {
    match state.unwrap_or("") {
        "draft" => "#6b7280",
        "ready" => "#3b82f6",
        "reviewed" => "#22c55e",
        "approved" => "#10b981",
        "archived" => "#6b7280",
        _ => "#6b7280",
    }
}
