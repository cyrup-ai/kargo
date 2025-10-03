#![allow(unsafe_code)]
use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use crate::*;
use std::path::PathBuf;

pub struct TurdPlugin;

impl PluginCommand for TurdPlugin {
    fn clap(&self) -> Command {
        Command::new("turd")
            .about("Find stubbed, incomplete, and non-production quality code")
            .long_about(
                "Analyzes Rust projects to detect stubbed code, incomplete implementations, \
                and non-production quality patterns. Generates task files for LLM-assisted remediation."
            )
            .arg(
                Arg::new("watch")
                    .long("watch")
                    .help("Watch mode: monitor for file changes and re-analyze")
                    .value_name("PATH")
                    .num_args(0..=1)
                    .value_parser(clap::value_parser!(PathBuf))
            )
            .arg(
                Arg::new("exclude")
                    .long("exclude")
                    .help("Exclude files matching glob pattern (can be repeated)")
                    .value_name("PATTERN")
                    .num_args(0..)
                    .action(clap::ArgAction::Append)
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        let cmd = self.clap();
        Box::pin(async move {
            // Parse arguments from ExecutionContext
            let matches = cmd.get_matches_from(&ctx.matched_args);

            // Extract arguments
            let watch_path = matches.get_one::<PathBuf>("watch").cloned();
            let exclude_patterns: Vec<String> = matches
                .get_many::<String>("exclude")
                .unwrap_or_default()
                .map(|s| s.to_string())
                .collect();

            // Initialize logging
            env_logger::Builder::new()
                .filter_level(log::LevelFilter::Info)
                .try_init()
                .ok();

            // Build configuration
            let mut config = Config::default();
            config.exclude_patterns.extend(exclude_patterns);
            config.watch_path = watch_path.clone();

            // Route to watch mode or normal analysis
            if watch_path.is_some() {
                // Watch mode (async)
                run_watch_mode(&config).await?;
            } else {
                // Normal mode - run once (sync in async context)
                // Use spawn_blocking to run sync code in async context
                tokio::task::spawn_blocking(move || {
                    run_analysis_sync(&config)
                })
                .await
                .map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;
            }

            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(TurdPlugin)
}
