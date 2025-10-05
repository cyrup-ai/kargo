use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use kargo_plugin_native::kargo_plugin;
use clap::{Arg, Command};
use anyhow::Result;

/// Your plugin implementation
pub struct {{plugin_name | pascal_case}}Plugin;

impl {{plugin_name | pascal_case}}Plugin {
    pub fn new() -> Self {
        Self
    }
}

/// Modern async plugin interface.
///
/// PluginCommand::run() returns a Future that integrates with kargo's async runtime.
/// Do NOT create your own runtime with Runtime::new() or use block_on().
///
/// For synchronous code, use tokio::task::spawn_blocking() within the async block.
impl PluginCommand for {{plugin_name | pascal_case}}Plugin {
    fn clap(&self) -> Command {
        Command::new("{{plugin_name}}")
            .about("{{plugin_description}}")
            .arg(
                Arg::new("example")
                    .short('e')
                    .long("example")
                    .help("An example argument")
                    .value_name("VALUE")
            )
            // TODO: Add more arguments as needed
    }
    
    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        let cmd = self.clap();
        Box::pin(async move {
            // Parse arguments from ExecutionContext
            let matches = cmd.get_matches_from(&ctx.matched_args);
            
            // TODO: Extract your arguments
            let example_value = matches.get_one::<String>("example");
            
            // TODO: Implement your plugin logic here
            println!("Hello from {{plugin_name}}!");
            
            if let Some(value) = example_value {
                println!("Example argument: {}", value);
            }
            
            // For blocking/sync code, use spawn_blocking:
            // tokio::task::spawn_blocking(move || {
            //     // Your sync code here
            // })
            // .await
            // .map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;
            
            Ok(())
        })
    }
}

// Generate the required extern "C" function and metadata
kargo_plugin! {
    name: "{{plugin_name}}",
    version: env!("CARGO_PKG_VERSION"),
    description: "{{plugin_description}}",
    author: "{{author_name}}",
    plugin_type: {{plugin_name | pascal_case}}Plugin
}

// The actual extern "C" function that kargo-cli will look for
#[no_mangle]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new({{plugin_name | pascal_case}}Plugin::new())
}
