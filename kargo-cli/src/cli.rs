use anyhow::Result;
use clap::{ArgMatches, Command};
use std::{env, path::PathBuf};

use crate::plugins::manager::PluginManager;
use kargo_plugin_api::ExecutionContext;

pub fn build_root_cli(pm: &PluginManager) -> Command {
    let mut root = Command::new("kargo")
        .about("Kargo Flux – cargo wrapper with zero-knowledge plugins")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true);

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

pub async fn dispatch(pm: &PluginManager, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("cargo", sub)) => {
            // Handle external subcommands from clap
            if let Some((ext_cmd, ext_args)) = sub.subcommand() {
                let mut args = vec![ext_cmd.to_string()];
                if let Some(values) = ext_args.get_many::<std::ffi::OsString>("") {
                    args.extend(values.map(|s| s.to_string_lossy().to_string()));
                }
                let status = tokio::process::Command::new("cargo")
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
            let plugin = pm.get(name)
                .ok_or_else(|| anyhow::anyhow!("Plugin '{}' was registered but not found in manager", name))?;
            
            // Gather args and prepend the subcommand name
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
        }
        None => unreachable!(),
    }
    Ok(())
}

fn gather_raw_args(m: &ArgMatches) -> Vec<String> {
    // Get the original command line arguments, excluding the program name and subcommand
    let args: Vec<String> = std::env::args()
        .skip(2)  // Skip "kargo" and "mddoc" (or whatever subcommand)
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
