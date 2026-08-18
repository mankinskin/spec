#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(feature = "cli")]
pub use cli::{
    BootstrapArgs,
    commands::cmd_bootstrap,
};
