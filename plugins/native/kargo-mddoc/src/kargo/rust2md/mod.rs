mod markdown;
mod types;

use crate::config::Config;
use anyhow::{Context, Result};
use log::{info, warn};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit;

pub use markdown::rustdoc_json_to_markdown;
pub use types::*;

/// Generator for package documentation
pub struct DocGenerator {
    #[allow(dead_code)]
    config: Config,
}

impl DocGenerator {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Generate Markdown documentation for a crate
    pub async fn generate_markdown_docs(&self, crate_path: &Path) -> Result<PathBuf> {
        info!("Generating documentation for {}", crate_path.display());

        // 1. Get package name from Cargo.toml
        let cargo_toml_path = crate_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .context(format!("Failed to read {}", cargo_toml_path.display()))?;

        let package_name = self
            .extract_package_name(&content)
            .context("Failed to extract package name from Cargo.toml")?;

        info!("Package name: {}", package_name);

        // 2. Run cargo rustdoc with nightly to generate JSON
        let json_path = self.run_cargo_doc(&package_name, crate_path)?;

        // 3. Parse JSON and generate markdown
        let markdown = rustdoc_json_to_markdown(&json_path)
            .await
            .context("Failed to convert rustdoc JSON to markdown")?;

        // 4. Write markdown to file in the knowledge base
        let output_path = self.get_output_path(&package_name)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).context("Failed to create output directory")?;
        }

        fs::write(&output_path, markdown).context(format!(
            "Failed to write markdown to {}",
            output_path.display()
        ))?;

        // Report success
        info!(
            "Generated documentation for {} at {}",
            package_name,
            output_path.display()
        );

        Ok(output_path)
    }

    /// Extract package name from Cargo.toml content
    fn extract_package_name(&self, content: &str) -> Result<String> {
        let doc = content
            .parse::<toml_edit::DocumentMut>()
            .context("Failed to parse Cargo.toml")?;

        let package = doc
            .get("package")
            .and_then(|p| p.as_table())
            .context("No package section in Cargo.toml")?;

        let name = package
            .get("name")
            .and_then(|n| n.as_str())
            .context("No name in package section")?;

        Ok(name.to_string())
    }

    /// Run cargo rustdoc with nightly to generate JSON documentation
    fn run_cargo_doc(&self, package_name: &str, crate_path: &Path) -> Result<PathBuf> {
        // Run the cargo +nightly command to generate JSON documentation
        let output = Command::new("cargo")
            .current_dir(crate_path)
            .arg("+nightly")
            .arg("-Zunstable-options")
            .arg("rustdoc")
            .arg("--output-format")
            .arg("json")
            .arg("--manifest-path")
            .arg("./Cargo.toml")
            .arg("--package")
            .arg(package_name)
            .output()
            .context("Failed to execute cargo +nightly rustdoc command")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("cargo rustdoc failed: {}", error_message));
        }

        // The JSON file is generated at target/doc/{package_name}.json
        let json_path = crate_path
            .join("target/doc")
            .join(format!("{}.json", package_name));

        if !json_path.exists() {
            return Err(anyhow::anyhow!(
                "Expected JSON file not found at {}",
                json_path.display()
            ));
        }

        info!("Found rustdoc JSON file at {}", json_path.display());

        Ok(json_path)
    }

    /// Get the output path for markdown documentation
    fn get_output_path(&self, package_name: &str) -> Result<PathBuf> {
        // Use environment variable for knowledge base root, or default
        let kb_root = env::var("KNOWLEDGE_BASE_ROOT_DIR").unwrap_or_else(|_| {
            warn!("KNOWLEDGE_BASE_ROOT_DIR not set, using default location");
            let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}/knowledge_base", home)
        });

        let output_path = PathBuf::from(kb_root)
            .join("rust")
            .join("crates")
            .join(package_name)
            .join("README.md");

        Ok(output_path)
    }
}
