use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use std::path::PathBuf;

use crate::executor::KargoExecutor;

pub struct KuratePlugin;

impl PluginCommand for KuratePlugin {
    fn clap(&self) -> Command {
        Command::new("kurate")
            .about("Execute cargo commands with LLM-optimized output processing")
            .long_about("Wraps cargo commands and processes their output to be more readable for LLMs")
            .arg(
                Arg::new("cargo_args")
                    .help("Cargo command and arguments to execute")
                    .value_name("ARGS")
                    .num_args(1..)
                    .required(true)
                    .allow_hyphen_values(true)
            )
            .arg(
                Arg::new("working_dir")
                    .long("working-dir")
                    .short('C')
                    .help("Working directory for cargo command")
                    .value_name("DIR")
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        let cmd = self.clap();
        Box::pin(async move {
            let matches = cmd.get_matches_from(&ctx.matched_args);

            // Extract cargo arguments
            let cargo_args: Vec<String> = matches
                .get_many::<String>("cargo_args")
                .ok_or_else(|| anyhow::anyhow!("No cargo arguments provided"))?
                .map(|s| s.to_string())
                .collect();

            // Determine working directory
            let working_dir = if let Some(dir) = matches.get_one::<String>("working_dir") {
                PathBuf::from(dir)
            } else {
                ctx.current_dir.clone()
            };

            // Create executor and run command
            let executor = KargoExecutor::new()?;
            executor.run_async(&cargo_args, &working_dir).await?;

            Ok(())
        })
    }
}

// Export the plugin for dynamic loading
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
#[allow(unsafe_code)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(KuratePlugin)
}
