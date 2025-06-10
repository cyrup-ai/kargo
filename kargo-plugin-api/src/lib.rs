use anyhow::Result;
use clap::Command;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

/// Type alias for asynchronous plugin execution, adhering to the cyrup-ai/async_task style
/// of returning an awaitable future.
pub type KargoPluginExecuteFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

/// Context provided to a plugin when it's executed.
#[derive(Debug, Clone)]
pub struct PluginExecutionContext {
    /// Arguments matched by clap for this specific subcommand
    pub matched_args: Vec<String>,
    
    /// Path to kargo's configuration directory, for plugin-specific configs
    pub kargo_config_dir: PathBuf,
    
    /// Current working directory from which kargo was invoked
    pub current_dir: PathBuf,
}

/// The core trait that all kargo plugins must implement.
pub trait KargoPluginCommand: Send + Sync {
    /// Returns the clap::Command definition for this plugin's subcommand.
    /// Kargo calls this at startup to build its main CLI structure.
    /// The command name returned here will be used as the subcommand name in kargo.
    fn clap_command(&self) -> Command;

    /// Executes the plugin's primary logic.
    /// This method is called when the plugin's subcommand is invoked.
    /// 
    /// Implementations should return a future that can be awaited by the caller,
    /// consistent with the cyrup-ai/async_task pattern of returning awaitables
    /// rather than using async traits.
    fn execute(&self, context: PluginExecutionContext) -> KargoPluginExecuteFuture;
}

/// C ABI-compatible function signature for loading plugins from native Rust dynamic libraries.
/// 
/// Plugin libraries must export a function with this signature, typically named
/// `_kargo_plugin_create`, which returns a boxed instance of a type implementing
/// the `KargoPluginCommand` trait.
pub type KargoPluginCreateFn = extern "C" fn() -> Box<dyn KargoPluginCommand>;

// Re-export clap to ensure plugin implementations use compatible versions
pub use clap;