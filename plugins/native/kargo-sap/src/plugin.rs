use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};

use crate::{list_directory, SapCommand};

impl PluginCommand for SapCommand {
    fn clap(&self) -> Command {
        Command::new("sap")
            .about("Smart Agent Protocol - AI-enhanced directory listing for LLM agents")
            .arg(
                Arg::new("path")
                    .help("Path to list (defaults to current directory)")
                    .value_name("PATH")
                    .index(1),
            )
            .arg(
                Arg::new("objective")
                    .long("objective")
                    .short('o')
                    .help("The objective or task the agent is trying to accomplish")
                    .value_name("TEXT"),
            )
            .arg(
                Arg::new("context")
                    .long("context")
                    .short('c')
                    .help("Additional context about the current work")
                    .value_name("TEXT"),
            )
            .arg(
                Arg::new("all")
                    .long("all")
                    .short('a')
                    .help("Show all files (including hidden)")
                    .action(clap::ArgAction::SetTrue),
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        Box::pin(async move {
            let cmd = SapCommand::new();
            let matches = cmd.clap().get_matches_from(&ctx.matched_args);

            let path = matches
                .get_one::<String>("path")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| ctx.current_dir.clone());

            let objective = matches.get_one::<String>("objective");
            let context = matches.get_one::<String>("context");
            let show_all = matches.get_flag("all");

            list_directory(&path, objective, context, show_all).await
        })
    }
}

/// FFI export for plugin loading
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
#[allow(unsafe_code)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(SapCommand::new())
}
