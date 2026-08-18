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

    SpecStore::open(&index_root).unwrap_or_else(|error| {
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
    let cwd_spec = std::env::current_dir().ok().map(|dir| dir.join(".spec"));
    if let Some(path) = cwd_spec.filter(|path| path.exists()) {
        return path;
    }
    if let Ok(home) =
        std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
    {
        return PathBuf::from(home).join(".spec-index");
    }
    PathBuf::from(".spec")
}
