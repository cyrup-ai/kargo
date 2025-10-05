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
