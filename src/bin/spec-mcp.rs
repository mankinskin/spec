use std::path::PathBuf;

use spec::mcp::server;
use spec_api::SpecStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("spec_mcp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    let resolution = resolve_index_root();
    tracing::info!(
        requested_server_root = %resolution.requested_server_root.display(),
        resolved_active_store_root = %resolution.active_store_root.display(),
        candidate_path_chain = ?resolution.candidate_path_chain,
        ancestor_lookup_used = resolution.ancestor_lookup_used,
        selection_reason = %resolution.selection_reason,
        resolution_diagnostics = ?resolution.resolution_diagnostics,
        "spec_mcp_startup_workspace_resolved"
    );
    let index_root = resolution.active_store_root;

    SpecStore::open_or_init(&index_root).unwrap_or_else(|error| {
        eprintln!(
            "Failed to open spec store at {}: {error}",
            index_root.display()
        );
        std::process::exit(1);
    });

    if let Err(error) = server::run_mcp_server(index_root).await {
        eprintln!("Fatal error: {error}");
        std::process::exit(1);
    }
}

struct StartupResolution {
    requested_server_root: PathBuf,
    active_store_root: PathBuf,
    candidate_path_chain: Vec<PathBuf>,
    ancestor_lookup_used: bool,
    selection_reason: String,
    resolution_diagnostics: Vec<String>,
}

fn resolve_index_root() -> StartupResolution {
    if let Ok(path) = std::env::var("SPEC_INDEX_ROOT") {
        let root = PathBuf::from(path);
        return StartupResolution {
            requested_server_root: root.clone(),
            active_store_root: root.clone(),
            candidate_path_chain: vec![root],
            ancestor_lookup_used: false,
            selection_reason: "SPEC_INDEX_ROOT override".to_string(),
            resolution_diagnostics: Vec::new(),
        };
    }
    if let Ok(path) = std::env::var("TICKET_INDEX_ROOT") {
        let root = PathBuf::from(path);
        return StartupResolution {
            requested_server_root: root.clone(),
            active_store_root: root.clone(),
            candidate_path_chain: vec![root],
            ancestor_lookup_used: false,
            selection_reason: "TICKET_INDEX_ROOT compatibility override".to_string(),
            resolution_diagnostics: Vec::new(),
        };
    }
    let requested_server_root = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let resolution =
        memory_kernel::workspace::resolve_store_root_from_with_diagnostics(
            &requested_server_root,
            ".spec",
        );
    let selected_workspace =
        memory_kernel::workspace::resolve_workspace_root_from_store_root(
            &resolution.store_root,
            ".spec",
        );
    let candidate_path_chain =
        workspace_candidate_path_chain(&requested_server_root, &selected_workspace);
    let ancestor_lookup_used = requested_server_root != selected_workspace;
    let selection_reason = if resolution.store_root
        == memory_kernel::workspace::canonical_store_root(
            &requested_server_root,
            ".spec",
        )
    {
        "no existing ancestor store; selected canonical store under requested server root"
    } else {
        "selected nearest existing store while walking workspace ancestors"
    };
    StartupResolution {
        requested_server_root,
        active_store_root: resolution.store_root,
        candidate_path_chain,
        ancestor_lookup_used,
        selection_reason: selection_reason.to_string(),
        resolution_diagnostics: resolution
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{diagnostic:?}"))
            .collect(),
    }
}

fn workspace_candidate_path_chain(start: &std::path::Path, selected: &std::path::Path) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut current = start;
    loop {
        chain.push(current.to_path_buf());
        if current == selected {
            break;
        }
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
    chain
}
