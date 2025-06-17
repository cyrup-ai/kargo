use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use kargo_plugin_native::{kargo_plugin, NativePlugin, PluginMetadata};
use clap::{Arg, Command};
use anyhow::Result;
use log::info;

/// Your plugin implementation
pub struct {{plugin_name | pascal_case}}Plugin;

impl {{plugin_name | pascal_case}}Plugin {
    pub fn new() -> Self {
        Self
    }
    
    async fn run_async(&self, ctx: ExecutionContext) -> Result<()> {
        info!("Running {{plugin_name}} plugin");
        
        // Parse arguments from the matched args
        let args = ctx.matched_args;
        
        // TODO: Implement your plugin logic here
        println!("Hello from {{plugin_name}}!");
        
        if args.len() > 1 {
            println!("Arguments received: {:?}", &args[1..]);
        }
        
        Ok(())
    }
}

// Implement the NativePlugin trait for better developer experience
impl NativePlugin for {{plugin_name | pascal_case}}Plugin {
    fn command(&self) -> Command {
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
    
    fn execute(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // Convert sync execute to async
        let ctx = ExecutionContext {
            matched_args: args,
            current_dir: std::env::current_dir()?,
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("kargo"),
        };
        
        // Block on async execution
        tokio::runtime::Runtime::new()?
            .block_on(self.run_async(ctx))?;
        
        Ok(())
    }
    
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "{{plugin_name}}".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "{{plugin_description}}".to_string(),
            author: "{{author_name}}".to_string(),
        }
    }
}

// Implement PluginCommand for kargo-cli compatibility
impl PluginCommand for {{plugin_name | pascal_case}}Plugin {
    fn clap(&self) -> Command {
        self.command()
    }
    
    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        Box::pin(self.run_async(ctx))
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