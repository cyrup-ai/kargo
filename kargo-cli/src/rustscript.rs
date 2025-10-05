use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use toml_edit::DocumentMut;

use crate::project::CargoSection;
use kargo_upgrade::models::{DependencyParser, DependencySource, DependencyUpdate, DependencyUpdater, DependencyWriter};
use kargo_upgrade::parsers::RustScriptParser;
use kargo_upgrade::types::UpdateOptions;
use kargo_upgrade::updater::CratesIoUpdater;
use kargo_upgrade::writers::RustScriptWriter;

/// Structure representing a Rust script with cargo dependencies
pub struct RustScript {
    /// Path to the script file
    pub path: PathBuf,
    /// Detected cargo sections
    pub sections: Vec<CargoSection>,
    /// Extracted dependencies
    pub dependencies: HashMap<String, String>,
    /// Original file content
    pub content: String,
}

impl RustScript {
    /// Create a new `RustScript` instance by parsing a file
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path).await?;

        let (sections, dependencies) = Self::parse_cargo_sections(&content)?;

        Ok(Self {
            path,
            sections,
            dependencies,
            content,
        })
    }

    /// Parse the cargo sections from Rust script content
    fn parse_cargo_sections(content: &str) -> Result<(Vec<CargoSection>, HashMap<String, String>)> {
        let mut sections = Vec::new();
        let mut dependencies = HashMap::new();

        // Multiple formats of cargo sections
        let patterns = [
            // Standard format
            r"```cargo\s*\n([\s\S]*?)```",
            // Doc comment format
            r"//!\s*```cargo\s*\n(//!\s*[\s\S]*?)```",
            // Line comment format
            r"//\s*```cargo\s*\n(//\s*[\s\S]*?)```",
        ];

        // Compile the line regex once outside the loop
        let line_regex = Regex::new(r"^(//!?)\s?")?;

        for pattern in patterns {
            let regex = Regex::new(pattern)?;

            for captures in regex.captures_iter(content) {
                if let Some(cargo_match) = captures.get(1) {
                    let cargo_content = cargo_match.as_str();
                    let range = cargo_match.range();

                    // Clean up content if it has comment prefixes
                    let cleaned_content =
                        if cargo_content.starts_with("//!") || cargo_content.starts_with("//") {
                            line_regex.replace_all(cargo_content, "").to_string()
                        } else {
                            cargo_content.to_string()
                        };

                    // Add to cargo sections
                    sections.push(CargoSection {
                        start: range.start,
                        end: range.end,
                        content: cleaned_content.clone(),
                    });

                    // Parse as TOML to extract dependencies
                    if let Ok(doc) = cleaned_content.parse::<DocumentMut>() {
                        Self::extract_dependencies_from_document(&doc, &mut dependencies);
                    } else {
                        // Fallback to regex for simpler formats if TOML parsing fails
                        Self::extract_dependencies_with_regex(&cleaned_content, &mut dependencies)?;
                    }
                }
            }
        }

        Ok((sections, dependencies))
    }

    /// Extract dependencies from a TOML document
    fn extract_dependencies_from_document(
        doc: &DocumentMut,
        dependencies: &mut HashMap<String, String>,
    ) {
        // Check standard dependencies section
        if let Some(deps) = doc.get("dependencies")
            && let Some(deps_table) = deps.as_table()
        {
            for (key, value) in deps_table {
                // Extract version based on format
                if let Some(version) = extract_version(value) {
                    dependencies.insert(key.to_string(), version);
                }
            }
        }

        // Also check dev-dependencies
        if let Some(deps) = doc.get("dev-dependencies")
            && let Some(deps_table) = deps.as_table()
        {
            for (key, value) in deps_table {
                // Extract version based on format
                if let Some(version) = extract_version(value) {
                    dependencies.insert(key.to_string(), version);
                }
            }
        }
    }

    /// Extract dependencies using regex for simple formats
    fn extract_dependencies_with_regex(
        content: &str,
        dependencies: &mut HashMap<String, String>,
    ) -> Result<()> {
        // Pattern for simple dependency declarations
        let regex = Regex::new(r#"(\w+)\s*=\s*["']([^"']+)["']"#)
            .context("Failed to compile dependency regex")?;

        for captures in regex.captures_iter(content) {
            if captures.len() >= 3 {
                let name = captures
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("Missing capture group 1"))?
                    .as_str()
                    .to_string();
                let version = captures
                    .get(2)
                    .ok_or_else(|| anyhow::anyhow!("Missing capture group 2"))?
                    .as_str()
                    .to_string();
                dependencies.insert(name, version);
            }
        }
        Ok(())
    }

    /// Update dependencies to their latest versions using kargo-upgrade
    ///
    /// This method:
    /// 1. Creates a `DependencySource` from the rust script
    /// 2. Parses dependencies using `RustScriptParser`
    /// 3. Updates to latest versions using `CratesIoUpdater`
    /// 4. Writes changes back using `RustScriptWriter`
    ///
    /// Returns a Vec of `DependencyUpdate` describing what was updated
    pub async fn update_dependencies(&mut self) -> Result<Vec<DependencyUpdate>> {
        // Create a dependency source from the current rust script
        let mut source = DependencySource::RustScript {
            path: self.path.clone(),
            content: self.content.clone(),
        };

        // Parse dependencies from the rust script
        let parser = RustScriptParser;
        let dependencies = parser.parse(&source)?;

        if dependencies.is_empty() {
            // No dependencies to update
            return Ok(Vec::new());
        }

        // Configure update options
        let options = UpdateOptions {
            update_workspace: false, // Rust scripts don't have workspaces
            compatible_only: true,    // Respect semver, skip major version bumps
        };

        // Create updater for getting latest versions from crates.io
        let updater = CratesIoUpdater::new(options);

        // Update all dependencies concurrently
        let mut updates = Vec::new();
        for dep in dependencies {
            // Get the update for this dependency
            let update_result = updater.update(&dep).await;
            
            if let Ok(Some(update)) = update_result {
                updates.push(update);
            }
        }

        if updates.is_empty() {
            // All dependencies already at latest versions
            return Ok(Vec::new());
        }

        // Apply updates to the source
        let writer = RustScriptWriter;
        writer.apply_updates(&mut source, &updates)?;

        // Write the updated content back to disk
        let write_future = writer.write(&source)?;
        write_future.await?;

        // Update our internal content to match what was written
        self.content = source.content().to_string();

        // Re-parse to update our sections and dependencies
        let (sections, dependencies) = Self::parse_cargo_sections(&self.content)?;
        self.sections = sections;
        self.dependencies = dependencies;

        Ok(updates)
    }
}

/// Extract version from a TOML value
fn extract_version(value: &toml_edit::Item) -> Option<String> {
    match value {
        toml_edit::Item::Value(value) => value.as_str().map(std::string::ToString::to_string),
        toml_edit::Item::Table(table) => table
            .get("version")
            .and_then(|version| version.as_str().map(std::string::ToString::to_string)),
        _ => None,
    }
}
