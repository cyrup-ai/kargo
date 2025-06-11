use kargo_plugin_api::{ExecutionContext, PluginCommand, BoxFuture};
use clap::{Command, Arg};
use crate::{Config, DocGenerator};
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
                    .help("Output directory for documentation")
                    .value_name("DIR")
                    .default_value("./rust_docs")
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
            let package_spec = matches.get_one::<String>("package").unwrap().clone();
            let output_dir = PathBuf::from(matches.get_one::<String>("output").unwrap());
            let temp_dir = matches.get_one::<String>("temp-dir").map(PathBuf::from);
            let keep_temp = matches.get_flag("keep-temp");
            let skip_component_check = matches.get_flag("skip-component-check");
            let document_private_items = matches.get_flag("document-private-items");
            let keep_json = matches.get_flag("keep-json");
            let json_only = matches.get_flag("json-only");

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
                log::debug!("Converting JSON to Markdown");
                let markdown_path = crate::markdown::convert_to_markdown(&json_path)?;
                log::info!("Markdown documentation generated at: {}", markdown_path.display());
                
                // Clean up JSON files if not needed
                if !keep_json {
                    log::debug!("Removing intermediate JSON file");
                    if let Err(e) = std::fs::remove_file(&json_path) {
                        log::debug!("Failed to remove JSON file: {}", e);
                    }
                }
            } else {
                log::info!("JSON documentation generated at: {}", json_path.display());
            }

            Ok(())
        })
    }
}

#[no_mangle]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(MddocPlugin)
}