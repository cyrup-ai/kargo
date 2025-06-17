use anyhow::Result;
use std::{future::Future, path::PathBuf, pin::Pin};

pub type BoxFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub matched_args: Vec<String>,
    pub current_dir: PathBuf,
    pub config_dir: PathBuf,
}

pub trait PluginCommand: Send + Sync {
    fn clap(&self) -> clap::Command;
    fn run(&self, ctx: ExecutionContext) -> BoxFuture;
}

#[allow(improper_ctypes_definitions)]
pub type CreateFn = extern "C" fn() -> Box<dyn PluginCommand>;
