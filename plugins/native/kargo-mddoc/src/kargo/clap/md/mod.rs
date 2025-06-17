use crate::{Config, DocGenerator};
use clap::Parser;
use log::{debug, info};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = "rustdoc-md",
    version,
    about = "Generate Markdown documentation for Rust packages"
)]
struct Subcommand {
    /// Package name with optional version (e.g., 'tokio' or 'tokio@1.28.0')
    #[clap(name = "PACKAGE[@VERSION]")]
    package_spec: String,

    /// Output directory for documentation
    #[clap(short, long, default_value = "./rust_docs")]
    output_dir: PathBuf,

    /// Keep JSON documentation files (normally deleted after markdown conversion)
    #[clap(short = 'j', long)]
    keep_json: bool,

    /// Skip Markdown generation and only output JSON
    #[clap(long)]
    json_only: bool,

    /// Keep temporary directory after completion
    #[clap(short, long)]
    keep_temp: bool,

    /// Use specific temporary directory
    #[clap(long)]
    temp_dir: Option<PathBuf>,

    /// Skip checking/installing rustup components
    #[clap(long)]
    skip_component_check: bool,

    /// Enable verbose output
    #[clap(short, long)]
    verbose: bool,

    /// Include private items in documentation
    #[clap(long)]
    document_private_items: bool,
}

#[allow(dead_code)]
fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let cli = Subcommand::parse();

    // Initialize logger based on verbosity
    if cli.verbose {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // Create output directory if it doesn't exist
    if !cli.output_dir.exists() {
        std::fs::create_dir_all(&cli.output_dir)?;
    }

    // Build configuration from CLI options
    let config = Config {
        package_spec: cli.package_spec.clone(),
        output_dir: cli.output_dir.clone(),
        temp_dir: cli.temp_dir.clone(),
        keep_temp: cli.keep_temp,
        skip_component_check: cli.skip_component_check,
        verbose: cli.verbose,
        document_private_items: cli.document_private_items,
    };

    // Generate the documentation
    let mut generator = DocGenerator::new(config)?;
    let json_path = generator.run()?;

    // By default, we generate Markdown unless json_only is specified
    if !cli.json_only {
        debug!("Converting JSON to Markdown");
        let markdown_path = crate::markdown::convert_to_markdown(&json_path)?;
        info!(
            "Markdown documentation generated at: {}",
            markdown_path.display()
        );

        // Clean up JSON files if not needed
        if !cli.keep_json {
            debug!("Removing intermediate JSON file");
            if let Err(e) = std::fs::remove_file(&json_path) {
                debug!("Failed to remove JSON file: {}", e);
            }
        }
    } else {
        info!("JSON documentation generated at: {}", json_path.display());
    }

    Ok(())
}
