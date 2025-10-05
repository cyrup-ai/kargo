use anyhow::Result;
use std::{future::Future, path::PathBuf, pin::Pin};

pub type BoxFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub matched_args: Vec<String>,
    pub current_dir: PathBuf,
    pub config_dir: PathBuf,
    /// Tokio runtime handle for plugins that need to spawn tasks.
    /// Plugins should call `handle.enter()` before using libraries that
    /// call `tokio::spawn()` (e.g., watchexec) to set the runtime in TLS.
    pub runtime_handle: Option<tokio::runtime::Handle>,
}

pub trait PluginCommand: Send + Sync {
    fn clap(&self) -> clap::Command;
    fn run(&self, ctx: ExecutionContext) -> BoxFuture;
}

#[allow(improper_ctypes_definitions)]
pub type CreateFn = extern "C" fn() -> Box<dyn PluginCommand>;
