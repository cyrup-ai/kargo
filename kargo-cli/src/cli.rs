use anyhow::Result;
use clap::{ArgMatches, Command};
use std::{env, path::PathBuf};
use which::which;

use crate::plugins::manager::PluginManager;
use kargo_plugin_api::ExecutionContext;

pub fn build_root_cli(pm: &PluginManager) -> Command {
    let mut root = Command::new("kargo")
        .about("Kargo Flux – cargo wrapper with zero-knowledge plugins")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            clap::Arg::new("alias")
                .long("alias")
                .help("Start an interactive shell with cargo aliased to kargo")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with("help"),
        )
        .subcommand_required(false) // Don't require subcommand when using --alias
        .arg_required_else_help(true)
        .allow_external_subcommands(true);

    root = root.subcommand(
        Command::new("cargo")
            .about("Forward arbitrary cargo sub-commands")
            .trailing_var_arg(true)
            .allow_external_subcommands(true),
    );

    for (_, plugin) in pm.plugins_iter() {
        root = root.subcommand(plugin.clap());
    }
    root
}

async fn proxy_to_cargo(command: &str, args: &ArgMatches) -> Result<()> {
    // Find cargo binary in PATH
    let cargo_path = which("cargo")
        .map_err(|e| anyhow::anyhow!("Failed to find cargo binary in PATH: {}", e))?;

    let mut cargo_args = vec![command.to_string()];

    // Gather additional arguments
    if let Some((_, sub_args)) = args.subcommand() {
        cargo_args.extend(gather_raw_args(sub_args));
    } else {
        cargo_args.extend(gather_raw_args(args));
    }

    let status = tokio::process::Command::new(&cargo_path)
        .args(cargo_args)
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("cargo exited with {:?}", status.code());
    }

    Ok(())
}

pub async fn dispatch(pm: &PluginManager, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("cargo", sub)) => {
            // Find cargo binary in PATH
            let cargo_path = which("cargo")
                .map_err(|e| anyhow::anyhow!("Failed to find cargo binary in PATH: {}", e))?;

            // Handle external subcommands from clap
            if let Some((ext_cmd, ext_args)) = sub.subcommand() {
                let mut args = vec![ext_cmd.to_string()];
                if let Some(values) = ext_args.get_many::<std::ffi::OsString>("") {
                    args.extend(values.map(|s| s.to_string_lossy().to_string()));
                }
                let status = tokio::process::Command::new(&cargo_path)
                    .args(args)
                    .status()
                    .await?;
                if !status.success() {
                    anyhow::bail!("cargo exited with {:?}", status.code());
                }
            } else {
                anyhow::bail!("No cargo subcommand provided");
            }
        }
        Some((name, sub)) => {
            // Check if this is a known plugin
            if let Some(plugin) = pm.get(name) {
                // Run the plugin
                let mut args = vec![name.to_string()];
                args.extend(gather_raw_args(sub));

                let ctx = ExecutionContext {
                    matched_args: args,
                    current_dir: env::current_dir()?,
                    config_dir: dirs::config_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join("kargo"),
                };
                plugin.run(ctx).await?;
            } else {
                // Not a plugin, proxy to cargo
                proxy_to_cargo(name, sub).await?;
            }
        }
        None => unreachable!(),
    }
    Ok(())
}

fn gather_raw_args(m: &ArgMatches) -> Vec<String> {
    // Get the original command line arguments, excluding the program name and subcommand
    let args: Vec<String> = std::env::args()
        .skip(2) // Skip "kargo" and "mddoc" (or whatever subcommand)
        .collect();

    // If no args were captured from env, fall back to reconstructing from ArgMatches
    if args.is_empty() {
        m.ids()
            .flat_map(|id| {
                m.get_raw(id.as_str())
                    .into_iter()
                    .flat_map(|vals| vals.map(|v| v.to_string_lossy().into_owned()))
            })
            .collect()
    } else {
        args
    }
}
