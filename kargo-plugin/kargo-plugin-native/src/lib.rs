use std::any::Any;

/// Native plugin trait for kargo (LEGACY - DO NOT USE)
///
/// This trait is deprecated in favor of `PluginCommand` which provides proper async integration.
///
/// **Why deprecated:**
/// - Synchronous `execute()` method encourages `Runtime::new()` + `block_on` anti-pattern
/// - Not actually called by kargo-cli (dead interface)
/// - `PluginCommand` provides better async integration with kargo's runtime
///
/// **Use `PluginCommand` instead:**
/// ```ignore
/// impl PluginCommand for MyPlugin {
///     fn clap(&self) -> Command { /* ... */ }
///     fn run(&self, ctx: ExecutionContext) -> BoxFuture {
///         Box::pin(async move { /* ... */ })
///     }
/// }
/// ```
#[deprecated(
    since = "0.2.0",
    note = "Use PluginCommand trait instead. NativePlugin encourages block_on anti-pattern and is not called by kargo-cli."
)]
pub trait NativePlugin: Any + Send + Sync {
    /// Get the clap command definition for this plugin
    fn command(&self) -> clap::Command;

    /// Execute the plugin with the given arguments
    ///
    /// This is called on the main thread but plugins can:
    /// - Spawn threads
    /// - Use tokio/async-std/etc
    /// - Access filesystem/network/OS resources
    /// - Share memory between threads
    ///
    /// # Errors
    ///
    /// Returns an error if plugin execution fails
    fn execute(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>>;

    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// Macro to generate the plugin discovery metadata
///
/// Usage:
/// ```
/// use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
/// use kargo_plugin_native::kargo_plugin;
///
/// struct MyPlugin;
///
/// impl PluginCommand for MyPlugin {
///     fn clap(&self) -> clap::Command {
///         clap::Command::new("my-plugin")
///             .about("My awesome plugin")
///     }
///
///     fn run(&self, _ctx: ExecutionContext) -> BoxFuture {
///         Box::pin(async move { Ok(()) })
///     }
/// }
///
/// kargo_plugin! {
///     name: "my-plugin",
///     version: "0.1.0",
///     description: "My awesome plugin",
///     author: "Me",
///     plugin_type: MyPlugin
/// }
///
/// #[no_mangle]
/// pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
///     Box::new(MyPlugin)
/// }
/// ```
#[macro_export]
macro_rules! kargo_plugin {
    (
        name: $name:expr,
        version: $version:expr,
        description: $desc:expr,
        author: $author:expr,
        plugin_type: $type:ty
    ) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static KARGO_PLUGIN_DECLARATION: &str =
            concat!("kargo_native_plugin:", env!("CARGO_PKG_NAME"));

        #[doc(hidden)]
        pub static KARGO_PLUGIN_TYPE: &str = stringify!($type);
    };
}
