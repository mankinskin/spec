//! Spec Viewer — single-process HTTP server that serves the SPA frontend
//! and the spec REST API together.
//!
//! The viewer imports `spec-http` as a library and mounts its router
//! directly, eliminating the need for a separate backend process or proxy.
//!
//! # Usage
//!
//! ```bash
//! # Serve the SPA + API on a single port (default 4002)
//! spec-viewer
//!
//! # Custom port and workspace root
//! spec-viewer --port 4002 --index-root /path/to/specs
//! ```
//!
//! # Environment variables
//! - `PORT`       — HTTP listen port (default: 4002)
//! - `STATIC_DIR` — Path to pre-built SPA static files

use std::{
    env,
    io::Write,
    path::PathBuf,
};
use tracing::info;
use viewer_api::{
    client_log::{
        client_log_router,
        ClientLogState,
    },
    display_host,
    init_tracing_full,
    with_static_files,
    TracingConfig,
};

use spec_api::SpecStore;
use spec_http::state::SpecAppState;

struct CliOptions {
    port: u16,
    static_dir: PathBuf,
    index_root: Option<PathBuf>,
}

fn parse_cli_options() -> CliOptions {
    let mut port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4002);

    let mut static_dir: PathBuf = env::var("STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let trunk_dist = manifest.join("frontend/dioxus/dist");
            if trunk_dist.exists() {
                trunk_dist
            } else {
                manifest.join("static")
            }
        });

    let mut index_root: Option<PathBuf> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" =>
                if let Some(value) = args.next() {
                    if let Ok(parsed) = value.parse::<u16>() {
                        port = parsed;
                    }
                },
            "--static-dir" =>
                if let Some(value) = args.next() {
                    static_dir = PathBuf::from(value);
                },
            "--index-root" => {
                index_root = args.next().map(PathBuf::from);
            },
            _ => {},
        }
    }

    CliOptions {
        port,
        static_dir,
        index_root,
    }
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )
        .expect("failed to register SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => {}
            _ = sigterm.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = ctrl_c.await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_cli_options();

    let workspace_root = std::env::var("WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            manifest_dir
                .parent() // memory-viewers/
                .and_then(|p| p.parent()) // context-engine/
                .map(|p| p.to_path_buf())
                .unwrap_or(manifest_dir)
        });
    let default_log_dir = workspace_root.join("target").join("logs");
    let config = TracingConfig::from_env("spec-viewer", default_log_dir);
    init_tracing_full(&config);
    info!(
        port = options.port,
        static_dir = %options.static_dir.display(),
        "Spec Viewer starting (single-process mode)"
    );

    // Resolve the spec index root.  Falls back to the `.spec` directory in the
    // current working directory if --index-root is not provided.
    let index_root = options.index_root.unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".spec")
    });

    info!(index_root = %index_root.display(), "Using spec index root");

    let store = SpecStore::open(&index_root).map_err(|error| {
        std::io::Error::other(format!(
            "Failed to open spec store at {}: {error}",
            index_root.display()
        ))
    })?;
    let state = SpecAppState::new(store);

    // Pre-scan so slugs are available immediately.
    {
        let mut s = state.store.lock().await;
        let _ = s.scan(false);
    }

    let api_router = spec_http::build_router(state);
    let api_router =
        api_router.merge(client_log_router(ClientLogState::default()));
    let app = with_static_files(api_router, Some(options.static_dir.clone()));

    let addr = format!("0.0.0.0:{}", options.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let actual_port = listener.local_addr()?.port();
    info!(
        "Listening on http://{}:{}",
        display_host("0.0.0.0"),
        actual_port
    );
    println!("SPEC_VIEWER_PORT={actual_port}");
    std::io::stdout().flush().expect("failed to flush stdout");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
