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

    let index_root = resolve_index_root();

    SpecStore::open_or_init(&index_root).unwrap_or_else(|error| {
        eprintln!(
            "Failed to open spec store at {}: {error}",
            index_root.display()
        );
        std::process::exit(1);
    });

    eprintln!("spec-mcp starting (store: {})", index_root.display());

    if let Err(error) = server::run_mcp_server(index_root).await {
        eprintln!("Fatal error: {error}");
        std::process::exit(1);
    }
}

fn resolve_index_root() -> PathBuf {
    if let Ok(path) = std::env::var("SPEC_INDEX_ROOT") {
        return PathBuf::from(path);
    }
    if let Ok(path) = std::env::var("TICKET_INDEX_ROOT") {
        return PathBuf::from(path);
    }
    std::env::current_dir()
        .map(|dir| memory_kernel::workspace::resolve_store_root_from(&dir, ".spec"))
        .unwrap_or_else(|_| PathBuf::from(".workflow-tools/spec"))
}
