use std::path::PathBuf;

use memory_kernel::runtime::init_transport_tracing;
use spec::http::{
    ServeConfig,
    SpecAppState,
    start_server,
};
use spec_api::SpecStore;

#[tokio::main]
async fn main() {
    init_transport_tracing("spec_http=info", None, None, "info");

    let mut port: u16 = 4001;
    let mut host = "127.0.0.1".to_string();
    let mut index_root: Option<String> = None;

    let args: Vec<String> = std::env::args().collect();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--port" => {
                index += 1;
                port = args[index].parse().expect("invalid port");
            },
            "--host" => {
                index += 1;
                host = args[index].clone();
            },
            "--index-root" => {
                index += 1;
                index_root = Some(args[index].clone());
            },
            _ => {},
        }
        index += 1;
    }

    let root = index_root
        .map(PathBuf::from)
        .or_else(|| std::env::var("SPEC_INDEX_ROOT").ok().map(PathBuf::from))
        .or_else(|| std::env::var("TICKET_INDEX_ROOT").ok().map(PathBuf::from))
        .unwrap_or_else(|| {
            let cwd = std::env::current_dir().expect("cwd");
            let spec_dir = cwd.join(".spec");
            if spec_dir.exists() {
                spec_dir
            } else {
                cwd.join(".ticket")
            }
        });

    let store = SpecStore::open_or_init(&root).unwrap_or_else(|error| {
        eprintln!("Failed to open spec store at {}: {error}", root.display());
        std::process::exit(1);
    });

    let state = SpecAppState::new(store);
    let config = ServeConfig { host, port };

    if let Err(error) = start_server(config, state).await {
        eprintln!("Fatal error: {error}");
        std::process::exit(1);
    }
}
