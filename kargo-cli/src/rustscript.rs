use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use toml_edit::DocumentMut;

use crate::project::CargoSection;
// TODO: These should use kargo-upgrade when integrated
// use kargo_upgrade::crates_io::get_latest_version;
// use kargo_upgrade::models::Dependency;
// use kargo_upgrade::types::DependencyUpdate;

/// Structure representing a Rust script with cargo dependencies
pub struct RustScript {
    /// Path to the script file
    pub path: PathBuf,
    /// Detected cargo sections
    pub sections: Vec<CargoSection>,
    /// Extracted dependencies
    pub dependencies: HashMap<String, String>,
    /// Original file content
    _content: String,
}

impl RustScript {
    /// Create a new RustScript instance by parsing a file
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path).await?;

        let (sections, dependencies) = Self::parse_cargo_sections(&content)?;

        Ok(Self {
            path,
            sections,
            dependencies,
            _content: content,
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

        for pattern in patterns {
            let regex = Regex::new(pattern)?;

            for captures in regex.captures_iter(content) {
                if let Some(cargo_match) = captures.get(1) {
                    let cargo_content = cargo_match.as_str();
                    let range = cargo_match.range();

                    // Clean up content if it has comment prefixes
                    let cleaned_content =
                        if cargo_content.starts_with("//!") || cargo_content.starts_with("//") {
                            let line_regex = Regex::new(r"^(//!?)\s?")?;
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
        if let Some(deps) = doc.get("dependencies") {
            if let Some(deps_table) = deps.as_table() {
                for (key, value) in deps_table.iter() {
                    // Extract version based on format
                    if let Some(version) = extract_version(value) {
                        dependencies.insert(key.to_string(), version);
                    }
                }
            }
        }

        // Also check dev-dependencies
        if let Some(deps) = doc.get("dev-dependencies") {
            if let Some(deps_table) = deps.as_table() {
                for (key, value) in deps_table.iter() {
                    // Extract version based on format
                    if let Some(version) = extract_version(value) {
                        dependencies.insert(key.to_string(), version);
                    }
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

    /* TODO: Uncomment when kargo-upgrade is integrated
    /// Update dependencies to their latest versions
    pub async fn update_dependencies(&mut self) -> Result<Vec<DependencyUpdate>> {
        let mut updates = Vec::new();
        let mut updated_content = self.content.clone();

        // Process each cargo section
        for section in &self.sections {
            let mut section_content = section.content.clone();
            let mut section_updates = Vec::new();

            // Update dependencies
            for (name, current_version) in &self.dependencies {
                // Get the latest version from crates.io
                if let Some(latest_version) = get_latest_version(name).await? {
                    // Skip if already at latest version
                    if current_version == &latest_version {
                        continue;
                    }

                    // Create a dummy dependency to use with the update
                    let dummy_dep = Dependency {
                        name: name.clone(),
                        version: current_version.clone(),
                        location: crate::up2date::models::DependencyLocation::RustScriptCargo {
                            section_range: (0, 0),
                        },
                    };

                    // Add to updates
                    section_updates.push(DependencyUpdate {
                        name: name.clone(),
                        from_version: current_version.clone(),
                        to_version: latest_version.clone(),
                        dependency: dummy_dep,
                    });

                    // Update in section content - handle different formats
                    update_dependency_in_content(
                        name,
                        current_version,
                        &latest_version,
                        &mut section_content,
                    );
                }
            }

            // If we have updates in this section, apply them to the file content
            if !section_updates.is_empty() {
                // Create the updated cargo section
                let original_section = &self.content[section.start..section.end];

                // Replace in the file content, handling comment-based sections
                if original_section.contains("//!") {
                    // Doc comment format
                    let doc_regex = Regex::new(r"^")?;
                    let updated_section =
                        doc_regex.replace_all(&section_content, "//! ").to_string();
                    updated_content.replace_range(section.start..section.end, &updated_section);
                } else if original_section.contains("//") {
                    // Line comment format
                    let line_regex = Regex::new(r"^")?;
                    let updated_section =
                        line_regex.replace_all(&section_content, "// ").to_string();
                    updated_content.replace_range(section.start..section.end, &updated_section);
                } else {
                    // Standard format
                    updated_content.replace_range(section.start..section.end, &section_content);
                }

                // Add updates to the result
                updates.extend(section_updates);
            }
        }

        // If we made updates, write the changes back to disk
        if !updates.is_empty() {
            fs::write(&self.path, &updated_content).await?;
            self.content = updated_content.clone();
        }

        Ok(updates)
    }
    */
}

/// Extract version from a TOML value
fn extract_version(value: &toml_edit::Item) -> Option<String> {
    match value {
        toml_edit::Item::Value(value) => {
            if let Some(version) = value.as_str() {
                Some(version.to_string())
            } else {
                None
            }
        }
        toml_edit::Item::Table(table) => {
            if let Some(version) = table.get("version") {
                if let Some(version_str) = version.as_str() {
                    Some(version_str.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
