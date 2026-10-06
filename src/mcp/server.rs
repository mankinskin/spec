use rmcp::{
    ErrorData as McpError,
    ServerHandler,
    ServiceExt,
    handler::server::{
        tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    tool,
    tool_handler,
    tool_router,
    transport::stdio,
};
use serde::Serialize;
use serde_json::{
    Value,
    json,
};
use spec_api::{
    SpecStore,
    error::SpecError,
};
use std::{
    borrow::Cow,
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
};
use tokio::sync::Mutex;
mod query;
mod sections;
mod types;
pub use self::types::*;
// ── Server ───────────────────────────────────────────────────────────────────
struct WorkspaceResolution {
    requested_workspace: String,
    resolved_workspace_root: PathBuf,
    active_index_root: PathBuf,
    ancestor_lookup_used: bool,
    candidate_path_chain: Vec<String>,
    selection_reason: String,
    resolution_diagnostics: Vec<String>,
}

#[derive(Clone)]
pub struct SpecServer {
    index_root: PathBuf,
    tool_router: ToolRouter<Self>,
    /// Serializes all SpecStore open/drop cycles so concurrent MCP calls
    /// never race on the SQLite write lock, while still releasing the lock
    /// between calls so the CLI can also access the database.
    store_lock: Arc<Mutex<()>>,
}
impl SpecServer {
    fn display_path(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "/")
    }

    fn workspace_context(
        resolution: &WorkspaceResolution,
        store_existed_before_call: bool,
        store_initialized: Option<bool>,
    ) -> Value {
        json!({
            "requested_workspace": resolution.requested_workspace,
            "resolved_workspace_root": Self::display_path(&resolution.resolved_workspace_root),
            "active_index_root": Self::display_path(&resolution.active_index_root),
            "store_existed_before_call": store_existed_before_call,
            "store_initialized": store_initialized,
            "ancestor_lookup_used": resolution.ancestor_lookup_used,
            "candidate_path_chain": resolution.candidate_path_chain,
            "selection_reason": resolution.selection_reason,
            "resolution_diagnostics": resolution.resolution_diagnostics,
        })
    }

    fn contextualize_error(
        mut error: McpError,
        resolution: &WorkspaceResolution,
        store_existed_before_call: bool,
        store_initialized: Option<bool>,
    ) -> McpError {
        let mut context = Self::workspace_context(
            resolution,
            store_existed_before_call,
            store_initialized,
        );
        if let Some(data) = error.data.take() {
            if let Some(object) = context.as_object_mut() {
                object.insert("cause_data".to_string(), data);
            }
        }
        error.message = Cow::Owned(format!(
            "{} (requested workspace '{}', resolved store '{}')",
            error.message,
            resolution.requested_workspace,
            Self::display_path(&resolution.active_index_root),
        ));
        error.data = Some(context);
        error
    }

    fn add_resolution_scope(
        result: CallToolResult,
        resolution: &WorkspaceResolution,
        store_existed_before_call: bool,
        store_initialized: bool,
    ) -> Result<CallToolResult, McpError> {
        let text = result.content.iter().find_map(|content| {
            if let RawContent::Text(text) = &content.raw {
                Some(text.text.clone())
            } else {
                None
            }
        });
        let Some(text) = text else {
            return Ok(result);
        };
        let mut value: Value = serde_json::from_str(&text).map_err(|error| {
            Self::contextualize_error(
                McpError::internal_error(
                    format!("spec MCP response was not valid JSON: {error}"),
                    None,
                ),
                resolution,
                store_existed_before_call,
                Some(store_initialized),
            )
        })?;
        if let Value::Object(object) = &mut value {
            let scope = object
                .entry("scope")
                .or_insert_with(|| json!({}));
            if let Value::Object(scope) = scope {
                scope.extend(
                    Self::workspace_context(
                        resolution,
                        store_existed_before_call,
                        Some(store_initialized),
                    )
                    .as_object()
                    .expect("workspace context object")
                    .clone(),
                );
                scope.insert(
                    "workspace".to_string(),
                    json!(resolution.requested_workspace),
                );
                scope.insert(
                    "workspace_root".to_string(),
                    json!(Self::display_path(&resolution.resolved_workspace_root)),
                );
            }
        }
        Self::json_result(&value)
    }

    pub fn new(index_root: PathBuf) -> Self {
        Self {
            index_root,
            tool_router: Self::tool_router(),
            store_lock: Arc::new(Mutex::new(())),
        }
    }
    fn json_result<T: Serialize>(
        value: &T
    ) -> Result<CallToolResult, McpError> {
        let text = serde_json::to_string(value).map_err(|e| {
            McpError::internal_error(format!("serialization: {e}"), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
    fn json_result_with_scope(
        value: Value,
        active_index_root: &Path,
        requested_workspace: Option<&str>,
    ) -> Result<CallToolResult, McpError> {
        let workspace_root =
            memory_kernel::workspace::resolve_workspace_root_from_store_root(
                active_index_root,
                ".spec",
            );
        let workspace = requested_workspace
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("default")
            .to_string();
        let mut value = value;
        if let Value::Object(map) = &mut value {
            map.insert(
                "scope".to_string(),
                json!({
                    "workspace": workspace,
                    "active_index_root": active_index_root
                        .to_string_lossy()
                        .replace('\\', "/"),
                    "workspace_root": workspace_root
                        .to_string_lossy()
                        .replace('\\', "/"),
                }),
            );
        }
        Self::json_result(&value)
    }
    fn spec_err(e: SpecError) -> McpError {
        match &e {
            SpecError::NotFound(_) =>
                McpError::invalid_params(e.to_string(), None),
            SpecError::InvalidSlug(_) =>
                McpError::invalid_params(e.to_string(), None),
            SpecError::DuplicateSlug(_) =>
                McpError::invalid_params(e.to_string(), None),
            SpecError::EmptyBody(_) =>
                McpError::invalid_params(e.to_string(), None),
            SpecError::NoOpUpdate(_) =>
                McpError::invalid_params(e.to_string(), None),
            _ => McpError::internal_error(format!("spec error: {e}"), None),
        }
    }
    fn storage_err(e: memory_kernel::error::StorageError) -> McpError {
        McpError::internal_error(format!("storage error: {e}"), None)
    }
    fn is_spec_store_root(path: &Path) -> bool {
        path.join("specs").is_dir()
            || path.join("entities.db").is_file()
            || path.join("search_index").is_dir()
    }
    fn is_explicit_store_path(path: &Path) -> bool {
        let name = path.file_name().and_then(|name| name.to_str());
        name == Some(".spec")
            || (name == Some("spec")
                && path
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|name| name.to_str())
                    == Some(".workflow-tools"))
            || Self::is_spec_store_root(path)
    }
    fn resolve_workspace_root(
        &self,
        workspace: Option<&str>,
    ) -> Result<WorkspaceResolution, McpError> {
        let requested_workspace = workspace
            .map(str::trim)
            .filter(|workspace| !workspace.is_empty())
            .unwrap_or("default")
            .to_string();
        if requested_workspace == "default" {
            let active_index_root = self.index_root.clone();
            return Ok(WorkspaceResolution {
                requested_workspace,
                resolved_workspace_root:
                    memory_kernel::workspace::resolve_workspace_root_from_store_root(
                        &active_index_root,
                        ".spec",
                    ),
                active_index_root,
                ancestor_lookup_used: false,
                candidate_path_chain: Vec::new(),
                selection_reason: "server_default_index_root".to_string(),
                resolution_diagnostics: Vec::new(),
            });
        }
        let requested_workspace =
            memory_kernel::workspace::normalize_explicit_workspace_selector(
                Some(&requested_workspace),
            )
            .map_err(|error| {
                McpError::invalid_params(error.to_string(), None)
            })?
            .to_string_lossy()
            .into_owned();
        let requested_path = Path::new(&requested_workspace);
        let store_resolution =
            memory_kernel::workspace::resolve_explicit_store_root_from(
                requested_path,
                ".spec",
            );
        if requested_path.is_dir() || Self::is_explicit_store_path(requested_path) {
            let active_index_root = store_resolution.store_root;
            let resolved_workspace_root =
                memory_kernel::workspace::resolve_workspace_root_from_store_root(
                    &active_index_root,
                    ".spec",
                );
            let selection_reason = if Self::is_explicit_store_path(requested_path) {
                "explicit_store_path"
            } else {
                "explicit_workspace_bounded"
            };
            let candidate_path_chain = vec![Self::display_path(requested_path)];
            return Ok(WorkspaceResolution {
                requested_workspace,
                resolved_workspace_root,
                active_index_root,
                ancestor_lookup_used: false,
                candidate_path_chain,
                selection_reason: selection_reason.to_string(),
                resolution_diagnostics: store_resolution
                    .diagnostics
                    .iter()
                    .map(|diagnostic| format!("{diagnostic:?}"))
                    .collect(),
            });
        }
        let expected_store = memory_kernel::workspace::canonical_store_root(
            requested_path,
            ".spec",
        );
        Err(McpError::invalid_params(
            format!(
                "invalid workspace '{requested_workspace}': expected an existing workspace directory or explicit spec store path; expected store location '{}'",
                expected_store.display()
            ),
            Some(json!({
                "requested_workspace": requested_workspace,
                "expected_store_location": expected_store.to_string_lossy(),
                "resolved_workspace_root": Value::Null,
                "active_index_root": Value::Null,
                "store_existed_before_call": false,
                "store_initialized": false,
                "ancestor_lookup_used": false,
            })),
        ))
    }
    /// Open a mutable SpecStore under the serialization lock, run the closure,
    /// then drop both store and lock before returning.
    ///
    /// Uses `&mut SpecStore` since create/update/delete/scan all mutate the
    /// slug index. The auto-scan ensures slug resolution works on every call.
    async fn with_store(
        &self,
        workspace: Option<&str>,
        f: impl FnOnce(&mut SpecStore, &Path) -> Result<CallToolResult, McpError>,
    ) -> Result<CallToolResult, McpError> {
        let resolution = self.resolve_workspace_root(workspace)?;
        let _guard = self.store_lock.lock().await;
        let index_root = &resolution.active_index_root;
        let store_existed_before_call = index_root.exists();
        tracing::info!(
            target: "spec_mcp::workspace",
            requested_workspace = %resolution.requested_workspace,
            resolved_workspace_root = %resolution.resolved_workspace_root.display(),
            active_index_root = %resolution.active_index_root.display(),
            store_existed_before_call,
            ancestor_lookup_used = resolution.ancestor_lookup_used,
            candidate_path_chain = ?resolution.candidate_path_chain,
            selection_reason = %resolution.selection_reason,
            resolution_diagnostics = ?resolution.resolution_diagnostics,
            "spec_mcp_workspace_resolved"
        );
        let (mut store, store_initialized) =
            SpecStore::open_or_init_with_status(index_root).map_err(|error| {
                Self::contextualize_error(
                    Self::spec_err(error),
                    &resolution,
                    store_existed_before_call,
                    None,
                )
            })?;
        store.scan(false).map_err(|error| {
            Self::contextualize_error(
                Self::spec_err(error),
                &resolution,
                store_existed_before_call,
                Some(store_initialized),
            )
        })?;
        let result = f(&mut store, index_root);
        drop(store);
        match result {
            Ok(result) => Self::add_resolution_scope(
                result,
                &resolution,
                store_existed_before_call,
                store_initialized,
            ),
            Err(error) => Err(Self::contextualize_error(
                error,
                &resolution,
                store_existed_before_call,
                Some(store_initialized),
            )),
        }
    }

    async fn with_write_store(
        &self,
        workspace: Option<&str>,
        f: impl FnOnce(&mut SpecStore, &Path) -> Result<CallToolResult, McpError>,
    ) -> Result<CallToolResult, McpError> {
        let workspace =
            memory_kernel::workspace::normalize_explicit_workspace_selector(
                workspace,
            )
            .map_err(|error| McpError::invalid_params(error.to_string(), None))?;
        let workspace = workspace.to_string_lossy().into_owned();
        self.with_store(Some(&workspace), f).await
    }
}
// ── Tool implementations ──────────────────────────────────────────────────────
#[tool_router]
impl SpecServer {
    #[tool(
        name = "spec_create",
        description = "Create a new spec with title, slug, component, and optional body."
    )]
    pub async fn spec_create(
        &self,
        Parameters(input): Parameters<CreateSpecInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_create_tool(input).await
    }
    #[tool(
        name = "spec_get",
        description = "Get a spec by ID or slug, optionally with body and sections."
    )]
    pub async fn spec_get(
        &self,
        Parameters(input): Parameters<GetSpecInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_get_tool(input).await
    }
    #[tool(
        name = "spec_update",
        description = "Update a spec's fields, state, or body. Omit untouched keys; the response returns only changed or directly relevant fields."
    )]
    pub async fn spec_update(
        &self,
        Parameters(input): Parameters<UpdateSpecInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_update_tool(input).await
    }
    #[tool(name = "spec_delete", description = "Permanently delete a spec.")]
    pub async fn spec_delete(
        &self,
        Parameters(input): Parameters<SpecRefInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_delete_tool(input).await
    }
    #[tool(
        name = "spec_list",
        description = "List specs with optional field=value filters."
    )]
    pub async fn spec_list(
        &self,
        Parameters(input): Parameters<ListSpecsInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_list_tool(input).await
    }
    #[tool(
        name = "spec_search",
        description = "Full-text search across specs."
    )]
    pub async fn spec_search(
        &self,
        Parameters(input): Parameters<SearchSpecsInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_search_tool(input).await
    }
    #[tool(
        name = "spec_tree",
        description = "Get hierarchy subtree for a spec, or list all root specs."
    )]
    pub async fn spec_tree(
        &self,
        Parameters(input): Parameters<TreeInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_tree_tool(input).await
    }
    #[tool(
        name = "spec_health",
        description = "Run health checks on specs (completeness of required fields)."
    )]
    pub async fn spec_health(
        &self,
        Parameters(input): Parameters<HealthInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_health_tool(input).await
    }
    #[tool(
        name = "spec_refs_validate",
        description = "Validate code references for a spec (check file existence and line ranges)."
    )]
    pub async fn spec_refs_validate(
        &self,
        Parameters(input): Parameters<RefsValidateInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_refs_validate_tool(input).await
    }
    #[tool(name = "spec_section_add", description = "Add a section to a spec.")]
    pub async fn spec_section_add(
        &self,
        Parameters(input): Parameters<SectionAddInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_section_add_tool(input).await
    }
    #[tool(
        name = "spec_section_list",
        description = "List sections of a spec."
    )]
    pub async fn spec_section_list(
        &self,
        Parameters(input): Parameters<SpecRefInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_section_list_tool(input).await
    }
    #[tool(name = "spec_section_get", description = "Get section content.")]
    pub async fn spec_section_get(
        &self,
        Parameters(input): Parameters<SectionRefInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_section_get_tool(input).await
    }
    #[tool(
        name = "spec_section_delete",
        description = "Delete a section from a spec."
    )]
    pub async fn spec_section_delete(
        &self,
        Parameters(input): Parameters<SectionRefInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_section_delete_tool(input).await
    }
    #[tool(
        name = "spec_scan",
        description = "Scan and reindex all spec roots."
    )]
    pub async fn spec_scan(
        &self,
        Parameters(input): Parameters<ScanInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_scan_tool(input).await
    }
    #[tool(
        name = "spec_add_root",
        description = "Register a directory as a scan root for specs."
    )]
    pub async fn spec_add_root(
        &self,
        Parameters(input): Parameters<AddRootInput>,
    ) -> Result<CallToolResult, McpError> {
        self.spec_add_root_tool(input).await
    }
    #[tool(
        name = "spec_move_preflight",
        description = "Read-only preflight plan for moving a spec to another workspace store."
    )]
    pub async fn spec_move_preflight(
        &self,
        Parameters(input): Parameters<SpecMoveInput>,
    ) -> Result<CallToolResult, McpError> {
        let to = memory_kernel::workspace::normalize_explicit_workspace_selector(
            Some(&input.to_workspace_root),
        )
        .map_err(|err| McpError::invalid_params(err.to_string(), None))?;
        self.with_store(input.workspace.as_deref(), move |store, _| {
            if let Some(ids) = &input.ids {
                let ids = ids
                    .iter()
                    .map(|id| store.resolve_id(id).map_err(Self::spec_err))
                    .collect::<Result<Vec<_>, _>>()?;
                let report = store.plan_move_set(&ids, &to).map_err(Self::spec_err)?;
                return Self::json_result(&json!({
                    "status": if report.supported() { "ok" } else { "blocked" },
                    "mode": "preflight",
                    "ids": report.entity_ids,
                    "supported": report.supported(),
                    "blockers": report.blockers(),
                    "plan": report,
                }));
            }
            let id = input.id.as_deref().ok_or_else(|| {
                McpError::invalid_params("move requires id or ids".to_string(), None)
            })?;
            let id = store.resolve_id(id).map_err(Self::spec_err)?;
            let report = store
                .plan_move_preflight(&id, &to)
                .map_err(Self::spec_err)?;
            Self::json_result(&json!({
                "status": "ok", "mode": "preflight", "id": id.to_string(),
                "supported": report.supported(), "blockers": report.blockers,
            }))
        })
        .await
    }
    #[tool(
        name = "spec_move_apply",
        description = "Execute a supported spec move to another workspace store."
    )]
    pub async fn spec_move_apply(
        &self,
        Parameters(input): Parameters<SpecMoveInput>,
    ) -> Result<CallToolResult, McpError> {
        let to = memory_kernel::workspace::normalize_explicit_workspace_selector(
            Some(&input.to_workspace_root),
        )
        .map_err(|err| McpError::invalid_params(err.to_string(), None))?;
        self.with_write_store(input.workspace.as_deref(), move |store, _| {
            if let Some(ids) = &input.ids {
                let ids = ids
                    .iter()
                    .map(|id| store.resolve_id(id).map_err(Self::spec_err))
                    .collect::<Result<Vec<_>, _>>()?;
                let report = store.plan_move_set(&ids, &to).map_err(Self::spec_err)?;
                if !report.supported() {
                    return Err(McpError::invalid_params(
                        "move set preflight blocked; run spec_move_preflight for details"
                            .to_string(),
                        None,
                    ));
                }
                let outcome = store.execute_move_set(&report).map_err(Self::spec_err)?;
                return Self::json_result(&json!({
                    "status": "ok", "mode": "apply", "ids": outcome.entity_ids,
                    "journal_id": outcome.journal.id, "phase": outcome.journal.phase,
                    "entity_journal_ids": outcome.journal.entity_journal_ids,
                }));
            }
            let id = input.id.as_deref().ok_or_else(|| {
                McpError::invalid_params("move requires id or ids".to_string(), None)
            })?;
            let id = store.resolve_id(id).map_err(Self::spec_err)?;
            let report = store.plan_move_preflight(&id, &to).map_err(Self::spec_err)?;
            if !report.supported() {
                return Err(McpError::invalid_params(
                    "move preflight blocked; run spec_move_preflight for details".to_string(),
                    None,
                ));
            }
            let outcome = store.execute_move_with_journal(&report).map_err(Self::spec_err)?;
            Self::json_result(&json!({
                "status": "ok", "mode": "apply", "id": id.to_string(),
                "journal_id": outcome.journal.id, "phase": outcome.journal.phase,
            }))
        })
        .await
    }
    #[tool(
        name = "spec_move_resume",
        description = "Resume an interrupted spec move from a journal id."
    )]
    pub async fn spec_move_resume(
        &self,
        Parameters(input): Parameters<SpecMoveJournalInput>,
    ) -> Result<CallToolResult, McpError> {
        let journal = input.id.parse::<uuid::Uuid>().map_err(|e| {
            McpError::invalid_params(format!("invalid journal id: {e}"), None)
        })?;
        self.with_write_store(input.workspace.as_deref(), move |store, _| {
            let outcome = store.resume_move_with_journal(journal).map_err(Self::spec_err)?;
            Self::json_result(&json!({"status":"ok","mode":"resume","journal_id":outcome.journal.id,"phase":outcome.journal.phase}))
        })
        .await
    }
    #[tool(
        name = "spec_move_rollback",
        description = "Roll back a spec move from a journal id."
    )]
    pub async fn spec_move_rollback(
        &self,
        Parameters(input): Parameters<SpecMoveJournalInput>,
    ) -> Result<CallToolResult, McpError> {
        let journal = input.id.parse::<uuid::Uuid>().map_err(|e| {
            McpError::invalid_params(format!("invalid journal id: {e}"), None)
        })?;
        self.with_write_store(input.workspace.as_deref(), move |store, _| {
            let outcome = store.rollback_move_with_journal(journal).map_err(Self::spec_err)?;
            Self::json_result(&json!({"status":"ok","mode":"rollback","journal_id":outcome.journal.id,"phase":outcome.journal.phase}))
        })
        .await
    }
}
// ── MCP handler trait ─────────────────────────────────────────────────────────
#[tool_handler]
impl ServerHandler for SpecServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "spec-mcp provides direct access to the spec store. No HTTP backend required. Use named tools for spec operations."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
// ── Server startup ────────────────────────────────────────────────────────────
pub async fn run_mcp_server(
    index_root: PathBuf
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = SpecServer::new(index_root);
    tracing::info!("Starting spec-mcp server on stdio (direct store access)");
    let service = server.serve(stdio()).await.inspect_err(|err| {
        eprintln!("Server error: {err:?}");
    })?;
    service.waiting().await?;
    Ok(())
}
