use std::any::Any;

/// Native plugin trait for kargo
///
/// Implementations have full access to:
/// - Direct memory access
/// - OS resources (filesystem, network, etc.)  
/// - Thread spawning and async runtimes
/// - Shared memory between threads
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
/// kargo_plugin! {
///     name: "my-plugin",
///     version: "0.1.0",
///     description: "My awesome plugin",
///     author: "Me",
///     plugin_type: MyPluginStruct
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
