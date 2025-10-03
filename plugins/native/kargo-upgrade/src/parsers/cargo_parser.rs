//! Parser for Cargo.toml files

use anyhow::{Result, anyhow};
use toml_edit::{DocumentMut as Document, Item, Table};

use crate::models::{Dependency, DependencyLocation, DependencyParser, DependencySource};

/// Parser for Cargo.toml files
#[derive(Clone)]
pub struct CargoParser;

impl DependencyParser for CargoParser {
    fn parse(&self, source: &DependencySource) -> Result<Vec<Dependency>> {
        match source {
            DependencySource::CargoToml { content, .. } => {
                let document = content
                    .parse::<Document>()
                    .map_err(|e| anyhow!("Failed to parse Cargo.toml: {}", e))?;

                let mut dependencies = Vec::new();

                // Parse regular dependencies
                if let Some(deps) = document.get("dependencies")
                    && let Some(deps_table) = deps.as_table()
                {
                    self.parse_dependencies_table(
                        deps_table,
                        &mut dependencies,
                        DependencyLocation::CargoTomlDirect,
                    )?;
                }

                // Parse dev-dependencies
                if let Some(deps) = document.get("dev-dependencies")
                    && let Some(deps_table) = deps.as_table()
                {
                    self.parse_dependencies_table(
                        deps_table,
                        &mut dependencies,
                        DependencyLocation::CargoTomlDev,
                    )?;
                }

                // Parse build-dependencies
                if let Some(deps) = document.get("build-dependencies")
                    && let Some(deps_table) = deps.as_table()
                {
                    self.parse_dependencies_table(
                        deps_table,
                        &mut dependencies,
                        DependencyLocation::CargoTomlBuild,
                    )?;
                }

                // Handle workspace dependencies if present
                if let Some(workspace) = document.get("workspace")
                    && let Some(workspace_table) = workspace.as_table()
                    && let Some(deps) = workspace_table.get("dependencies")
                    && let Some(deps_table) = deps.as_table()
                {
                    self.parse_dependencies_table(
                        deps_table,
                        &mut dependencies,
                        DependencyLocation::CargoTomlDirect,
                    )?;
                }

                Ok(dependencies)
            }
            _ => Err(anyhow!("Not a Cargo.toml source")),
        }
    }
}

impl CargoParser {
    /// Parse a dependencies table and add dependencies to the result vector
    fn parse_dependencies_table(
        &self,
        table: &Table,
        dependencies: &mut Vec<Dependency>,
        location: DependencyLocation,
    ) -> Result<()> {
        for (name, value) in table.iter() {
            // Skip system packages like "package" and "patch"
            if name == "package" || name == "patch" {
                continue;
            }

            if let Some(version) = self.extract_version(value) {
                dependencies.push(Dependency {
                    name: name.to_string(),
                    version,
                    location: location.clone(),
                });
            }
        }

        Ok(())
    }

    /// Extract the version from a dependency item
    fn extract_version(&self, item: &Item) -> Option<String> {
        match item {
            // Simple string version like: anyhow = "1.0.0"
            Item::Value(value) => {
                if let Some(version) = value.as_str() {
                    return Some(version.to_string());
                }

                if let Some(inline_table) = value.as_inline_table() {
                    // Handle inline table: tokio = { version = "1.0.0", features = [...] }
                    // Skip workspace dependencies
                    if inline_table.contains_key("workspace") {
                        return None;
                    }

                    // Extract version from inline table
                    if let Some(version) = inline_table.get("version")
                        && let Some(version_str) = version.as_str()
                    {
                        return Some(version_str.to_string());
                    }
                }

                None
            }

            // Table specification like [dependencies.tokio] with version = "1.0.0"
            Item::Table(table) => {
                // Skip workspace dependencies
                if table.contains_key("workspace") {
                    return None;
                }

                if let Some(version) = table.get("version")
                    && let Some(version_str) = version.as_str()
                {
                    Some(version_str.to_string())
                } else {
                    None
                }
            }

            // Other formats not supported
            _ => None,
        }
    }
}
