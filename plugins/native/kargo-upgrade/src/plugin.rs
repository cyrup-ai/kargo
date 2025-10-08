use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use std::path::PathBuf;

use crate::{
    finder::{find_cargo_toml_files, find_rust_script_files},
    models::{DependencyParser, DependencySource, DependencyUpdater, DependencyWriter},
    parsers::{CargoParser, RustScriptParser},
    types::UpdateOptions,
    updater::CratesIoUpdater,
    writers::{CargoWriter, RustScriptWriter},
};

pub struct UpgradePlugin;

impl PluginCommand for UpgradePlugin {
    fn clap(&self) -> Command {
        Command::new("upgrade")
            .about("Check and upgrade Rust dependencies to their latest versions")
            .long_about(
                "Scans Cargo.toml files and Rust scripts for dependencies, \
                 checks crates.io for newer versions, and optionally applies updates."
            )
            .arg(
                Arg::new("path")
                    .help("Path to scan for dependencies (default: current directory)")
                    .value_name("PATH")
                    .index(1)
            )
            .arg(
                Arg::new("compatible-only")
                    .long("compatible-only")
                    .help("Only show compatible updates (no major version bumps)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("apply")
                    .long("apply")
                    .help("Apply updates to files (default: preview only)")
                    .action(clap::ArgAction::SetTrue)
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        Box::pin(async move {
            let plugin = UpgradePlugin;
            let matches = plugin.clap().get_matches_from(&ctx.matched_args);
            
            // Parse arguments
            let scan_path = matches
                .get_one::<String>("path")
                .map(PathBuf::from)
                .unwrap_or_else(|| ctx.current_dir.clone());
            
            let compatible_only = matches.get_flag("compatible-only");
            let apply = matches.get_flag("apply");
            
            // Create updater with options
            let options = UpdateOptions {
                update_workspace: true,
                compatible_only,
            };
            let updater = CratesIoUpdater::new(options);
            
            println!("🔍 Scanning for dependencies in {}...", scan_path.display());
            
            // Find all files
            let cargo_files = find_cargo_toml_files(&scan_path)?;
            let rust_scripts = find_rust_script_files(&scan_path)?;
            
            let mut total_updates = 0;
            
            // Create parsers and writers
            let cargo_parser = CargoParser;
            let cargo_writer = CargoWriter;
            let script_parser = RustScriptParser;
            let script_writer = RustScriptWriter;
            
            // Process Cargo.toml files
            for path in cargo_files {
                let mut source = DependencySource::from_path(&path).await?;
                let dependencies = cargo_parser.parse(&source)?;
                
                let mut file_updates = Vec::new();
                for dep in dependencies {
                    if let Some(update) = updater.update(&dep).await? {
                        file_updates.push(update);
                    }
                }
                
                if !file_updates.is_empty() {
                    println!("\n📦 {}", path.display());
                    for upd in &file_updates {
                        println!("   {} {} → {}", upd.name, upd.from_version, upd.to_version);
                        total_updates += 1;
                    }
                    
                    if apply {
                        cargo_writer.apply_updates(&mut source, &file_updates)?;
                        let pending = cargo_writer.write(&source)?;
                        pending.await?;
                    }
                }
            }
            
            // Process Rust scripts
            for path in rust_scripts {
                let mut source = DependencySource::from_path(&path).await?;
                let dependencies = script_parser.parse(&source)?;
                
                let mut file_updates = Vec::new();
                for dep in dependencies {
                    if let Some(update) = updater.update(&dep).await? {
                        file_updates.push(update);
                    }
                }
                
                if !file_updates.is_empty() {
                    println!("\n📝 {}", path.display());
                    for upd in &file_updates {
                        println!("   {} {} → {}", upd.name, upd.from_version, upd.to_version);
                        total_updates += 1;
                    }
                    
                    if apply {
                        script_writer.apply_updates(&mut source, &file_updates)?;
                        let pending = script_writer.write(&source)?;
                        pending.await?;
                    }
                }
            }
            
            // Display summary
            if total_updates == 0 {
                println!("\n✅ All dependencies are up to date!");
            } else if apply {
                println!("\n✅ Applied {} updates", total_updates);
            } else {
                println!("\n💡 {} updates available (use --apply to update)", total_updates);
            }
            
            Ok(())
        })
    }
}

/// FFI export for plugin loading
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
#[allow(unsafe_code)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(UpgradePlugin)
}
