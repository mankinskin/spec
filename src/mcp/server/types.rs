use std::collections::BTreeMap;

use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SpecRefInput {
    /// Workspace root or .spec store path. Read tools accept omission/default for the server store; spec_delete requires an explicit path and rejects omitted/empty/default/..; use . for server cwd.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Spec UUID, prefix, or slug.
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSpecInput {
    /// Concrete workspace path, repo root, .spec store path, or path inside that store. Do not use omitted, empty, 'default', or '..' for entity creation; use '.' explicitly to target the MCP server process's current working directory.
    pub workspace: String,
    /// Spec title.
    pub title: String,
    /// Hierarchical slug (e.g. "ticket-api/storage/store").
    pub slug: String,
    /// Component this spec belongs to.
    pub component: String,
    /// Parent spec ID or slug.
    #[serde(default)]
    pub parent: Option<String>,
    /// Scope (e.g. "public", "internal").
    #[serde(default)]
    pub scope: Option<String>,
    /// Body content (markdown).
    #[serde(default)]
    pub body: Option<String>,
    /// Additional structured fields to write into the manifest.
    #[serde(default)]
    pub fields: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSpecInput {
    /// Optional workspace root or .spec store path; omit/default uses the server store.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Spec UUID, prefix, or slug.
    pub id: String,
    /// Include body and sections in output.
    #[serde(default)]
    pub full: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSpecInput {
    /// Required explicit workspace root or .spec store path; omitted/empty/default/.. are rejected. Use . for server cwd.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Spec UUID, prefix, or slug.
    pub id: String,
    /// Field patches as key=value pairs (e.g. ["title=New Title", "state=active"]).
    #[serde(default)]
    pub fields: Option<Vec<String>>,
    /// Optional state to transition to.
    #[serde(default)]
    pub to_state: Option<String>,
    /// Optional body content to replace.
    #[serde(default)]
    pub body: Option<String>,
    /// Allow an empty body update.
    #[serde(default)]
    pub force_body: bool,
    /// Additional structured field patches.
    #[serde(default)]
    pub field_map: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListSpecsInput {
    /// Optional workspace root or .spec store path; omit/default uses the server store.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Filter by field=value predicates.
    #[serde(default)]
    pub where_clauses: Vec<String>,
    /// Maximum results.
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchSpecsInput {
    /// Optional workspace root or .spec store path; omit/default uses the server store.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Full-text search query.
    pub query: String,
    /// Maximum results.
    #[serde(default = "default_search_limit")]
    pub limit: usize,
}

fn default_search_limit() -> usize {
    20
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TreeInput {
    /// Optional workspace root or .spec store path; omit/default uses the server store.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Root spec ID or slug (omit for all roots).
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HealthInput {
    /// Optional workspace root or .spec store path; omit/default uses the server store.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Spec UUID, prefix, or slug (omit with all=true for all specs).
    #[serde(default)]
    pub id: Option<String>,
    /// Check all specs.
    #[serde(default)]
    pub all: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RefsValidateInput {
    /// Optional workspace root or .spec store path; omit/default uses the server store.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Spec UUID, prefix, or slug.
    pub id: String,
    /// Filesystem base for code_ref paths, not a store selector. Default . means server cwd.
    #[serde(default = "default_workspace_root")]
    pub workspace_root: String,
}

fn default_workspace_root() -> String {
    ".".to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SectionAddInput {
    /// Required explicit workspace root or .spec store path; omitted/empty/default/.. are rejected. Use . for server cwd.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Spec UUID, prefix, or slug.
    pub id: String,
    /// Section name.
    pub name: String,
    /// Section content (markdown).
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SectionRefInput {
    /// Workspace root or .spec store path. Reads accept omission/default for the server store; section deletion requires explicit selection and rejects omitted/empty/default/..; use . for server cwd.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Spec UUID, prefix, or slug.
    pub id: String,
    /// Section name.
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScanInput {
    /// Required explicit workspace root or .spec store path; omitted/empty/default/.. are rejected. Use . for server cwd.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Force full reindex.
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddRootInput {
    /// Required explicit workspace root or .spec store path; omitted/empty/default/.. are rejected. Use . for server cwd.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Directory path to register as a scan root.
    pub path: String,
    /// Optional label for this root.
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SpecMoveInput {
    /// Source workspace root or .spec store path. Preflight accepts omission/default for the server store; apply requires explicit selection and rejects omitted/empty/default/..; use . for server cwd.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Spec UUID, prefix, or slug to move when `ids` is absent.
    #[serde(default)]
    pub id: Option<String>,
    /// Optional normalized set of spec UUIDs, prefixes, or slugs to move.
    #[serde(default)]
    pub ids: Option<Vec<String>>,
    /// Destination workspace root.
    pub to_workspace_root: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SpecMoveJournalInput {
    /// Required explicit source workspace root or .spec store path; omitted/empty/default/.. are rejected. Use . for server cwd.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Move journal UUID returned as journal_id by spec_move_apply, not a spec ID.
    pub id: String,
}
