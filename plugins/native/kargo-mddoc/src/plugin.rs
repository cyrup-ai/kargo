#![allow(unsafe_code)]
use crate::{Config, DocGenerator};
use anyhow::anyhow;
use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use std::path::PathBuf;

pub struct MddocPlugin;

impl PluginCommand for MddocPlugin {
    fn clap(&self) -> Command {
        Command::new("mddoc")
            .about("Generate Markdown documentation for Rust packages")
            .long_about("Creates Markdown documentation from any Rust crate's API by leveraging rustdoc's JSON output format")
            .arg(
                Arg::new("package")
                    .help("Package name with optional version (e.g., 'tokio' or 'tokio@1.28.0')")
                    .required(true)
                    .index(1)
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .help("Output directory for documentation (default: ./docs/{package_name})")
                    .value_name("DIR")
            )
            .arg(
                Arg::new("keep-json")
                    .short('j')
                    .long("keep-json")
                    .help("Keep JSON documentation files (normally deleted after markdown conversion)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("json-only")
                    .long("json-only")
                    .help("Skip Markdown generation and only output JSON")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("keep-temp")
                    .short('k')
                    .long("keep-temp")
                    .help("Keep temporary directory after completion")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("temp-dir")
                    .long("temp-dir")
                    .help("Use specific temporary directory")
                    .value_name("DIR")
            )
            .arg(
                Arg::new("skip-component-check")
                    .long("skip-component-check")
                    .help("Skip checking/installing rustup components")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("document-private-items")
                    .long("document-private-items")
                    .help("Include private items in documentation")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Enable verbose output")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("multipage")
                    .short('m')
                    .long("multipage")
                    .help("Generate multi-page markdown with cross-references (better for RAG)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("base-url")
                    .long("base-url")
                    .help("Base URL for cross-references in multi-page mode")
                    .value_name("URL")
                    .default_value("")
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        let cmd = self.clap();
        Box::pin(async move {
            // Initialize logger
            let matches = cmd.get_matches_from(&ctx.matched_args);

            let verbose = matches.get_flag("verbose");
            if verbose {
                env_logger::Builder::new()
                    .filter_level(log::LevelFilter::Debug)
                    .init();
            } else {
                env_logger::Builder::new()
                    .filter_level(log::LevelFilter::Info)
                    .init();
            }

            // Build configuration from arguments
            let package_spec = matches
                .get_one::<String>("package")
                .ok_or_else(|| anyhow!("Package argument is required"))?
                .clone();
            // Parse package name from package_spec
            let package_name = package_spec.split('@').next().unwrap_or(&package_spec);
            
            let output_dir = matches
                .get_one::<String>("output")
                .map(|s| PathBuf::from(s))
                .unwrap_or_else(|| PathBuf::from("./docs").join(package_name));
            let temp_dir = matches.get_one::<String>("temp-dir").map(PathBuf::from);
            let keep_temp = matches.get_flag("keep-temp");
            let skip_component_check = matches.get_flag("skip-component-check");
            let document_private_items = matches.get_flag("document-private-items");
            let _keep_json = matches.get_flag("keep-json");
            let json_only = matches.get_flag("json-only");
            let multipage = matches.get_flag("multipage");
            let base_url = matches
                .get_one::<String>("base-url")
                .unwrap_or(&String::new())
                .clone();

            // Create output directory if it doesn't exist
            if !output_dir.exists() {
                std::fs::create_dir_all(&output_dir)?;
            }

            let config = Config {
                package_spec: package_spec.clone(),
                output_dir: output_dir.clone(),
                temp_dir: temp_dir.clone(),
                keep_temp,
                skip_component_check,
                verbose,
                document_private_items,
            };

            // Generate the documentation
            let mut generator = DocGenerator::new(config)?;
            let json_path = generator.run()?;

            // By default, we generate Markdown unless json_only is specified
            if !json_only {
                if multipage {
                    log::debug!("Converting JSON to multi-page Markdown");
                    let multipage_config = crate::multipage_markdown::MultipageConfig {
                        output_dir: output_dir.clone(),
                        base_url,
                        generate_index: true,
                        max_items_per_page: 50,
                    };
                    let generated_files = crate::multipage_markdown::convert_to_multipage_markdown(
                        &json_path,
                        multipage_config,
                    )?;
                    log::info!(
                        "Multi-page Markdown documentation generated: {} files in {}",
                        generated_files.len(),
                        output_dir.display()
                    );
                } else {
                    log::debug!("Converting JSON to single-page Markdown");
                    let markdown_path = crate::markdown::convert_to_markdown(&json_path)?;
                    log::info!(
                        "Markdown documentation generated at: {}",
                        markdown_path.display()
                    );
                }

                // Clean up JSON files if not needed
                // TODO: UNCOMMENT THIS AFTER DEBUGGING IS COMPLETE
                // if !keep_json {
                //     log::debug!("Removing intermediate JSON file");
                //     if let Err(e) = std::fs::remove_file(&json_path) {
                //         log::debug!("Failed to remove JSON file: {}", e);
                //     }
                // }
            } else {
                log::info!("JSON documentation generated at: {}", json_path.display());
            }

            Ok(())
        })
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
#[allow(unsafe_code)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(MddocPlugin)
}
