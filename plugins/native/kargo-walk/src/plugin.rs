use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use std::path::PathBuf;

use crate::{
    analyze_relationships, check_project_status, extract_project_info, find_cargo_toml_files,
    generate_index_yaml,
};

pub struct WalkPlugin;

impl PluginCommand for WalkPlugin {
    fn clap(&self) -> Command {
        Command::new("walk")
            .about("Discover and inventory Rust projects in a directory")
            .long_about(
                "Scans directories for Cargo.toml files, analyzes project metadata, \
                 checks build status, and generates an inventory file."
            )
            .arg(
                Arg::new("path")
                    .help("Directory to scan (default: current directory)")
                    .value_name("PATH")
                    .index(1)
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .short('o')
                    .help("Output file path")
                    .value_name("FILE")
                    .default_value("index.yaml")
            )
            .arg(
                Arg::new("limit")
                    .long("limit")
                    .short('n')
                    .help("Limit number of projects to scan")
                    .value_name("N")
            )
            .arg(
                Arg::new("skip-check")
                    .long("skip-check")
                    .help("Skip cargo check status verification (faster)")
                    .action(clap::ArgAction::SetTrue)
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        Box::pin(async move {
            let plugin = WalkPlugin;
            let matches = plugin.clap().get_matches_from(&ctx.matched_args);

            // Parse arguments
            let scan_path = matches
                .get_one::<String>("path")
                .map(PathBuf::from)
                .unwrap_or_else(|| ctx.current_dir.clone());

            let output_file = matches
                .get_one::<String>("output")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("index.yaml"));

            let limit = matches
                .get_one::<String>("limit")
                .and_then(|s| s.parse::<usize>().ok());

            let skip_check = matches.get_flag("skip-check");

            println!("🔍 Scanning for Rust projects in {}...", scan_path.display());

            // Step 1: Find all Cargo.toml files
            let mut cargo_toml_paths = find_cargo_toml_files(&scan_path)?;
            println!("   Found {} Cargo.toml files", cargo_toml_paths.len());

            // Apply limit if specified
            if let Some(n) = limit {
                cargo_toml_paths.truncate(n);
                println!("   Limited to {} projects", n);
            }

            // Step 2: Extract project information
            println!("\n📦 Extracting project metadata...");
            let mut projects = extract_project_info(&cargo_toml_paths)?;

            // Step 3: Check project status (unless skipped)
            if !skip_check {
                println!("\n✅ Checking project status...");
                projects = check_project_status(projects).await?;
            }

            // Step 4: Analyze relationships
            println!("\n🔗 Analyzing project relationships...");
            let projects = analyze_relationships(projects);

            // Step 5: Generate output
            println!("\n💾 Generating inventory...");
            generate_index_yaml(&projects, &output_file)?;

            println!("\n✅ Completed! Inventory saved to {}", output_file.display());
            println!("   Scanned {} projects", projects.len());

            Ok(())
        })
    }
}

/// FFI export for plugin loading
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
#[allow(unsafe_code)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(WalkPlugin)
}
