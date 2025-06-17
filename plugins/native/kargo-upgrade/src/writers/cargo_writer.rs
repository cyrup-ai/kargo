//! Writer for Cargo.toml files

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tokio::fs;
use toml_edit::{DocumentMut as Document, Item, Value};

use crate::models::{DependencyLocation, DependencySource, DependencyUpdate, DependencyWriter};
use crate::types::PendingWrite;

/// Writer for Cargo.toml files
#[derive(Clone)]
pub struct CargoWriter;

impl DependencyWriter for CargoWriter {
    fn apply_updates(
        &self,
        source: &mut DependencySource,
        updates: &[DependencyUpdate],
    ) -> Result<()> {
        match source {
            DependencySource::CargoToml { content, .. } => {
                let mut document = content
                    .parse::<Document>()
                    .map_err(|e| anyhow!("Failed to parse Cargo.toml: {}", e))?;

                // Group updates by section
                let mut direct_updates = HashMap::new();
                let mut dev_updates = HashMap::new();
                let mut build_updates = HashMap::new();

                for update in updates {
                    match update.dependency.location {
                        DependencyLocation::CargoTomlDirect => {
                            direct_updates.insert(update.name.clone(), update.to_version.clone());
                        }
                        DependencyLocation::CargoTomlDev => {
                            dev_updates.insert(update.name.clone(), update.to_version.clone());
                        }
                        DependencyLocation::CargoTomlBuild => {
                            build_updates.insert(update.name.clone(), update.to_version.clone());
                        }
                        _ => {} // Ignore other location types
                    }
                }

                // Update workspace dependencies if present
                if let Some(workspace) = document.get_mut("workspace") {
                    if let Some(workspace_table) = workspace.as_table_mut() {
                        if let Some(deps) = workspace_table.get_mut("dependencies") {
                            if let Some(deps_table) = deps.as_table_mut() {
                                for (name, version) in &direct_updates {
                                    self.update_dependency_in_table(deps_table, name, version)?;
                                }
                            }
                        }
                    }
                }

                // Update regular dependencies
                if let Some(deps) = document.get_mut("dependencies") {
                    if let Some(deps_table) = deps.as_table_mut() {
                        for (name, version) in &direct_updates {
                            self.update_dependency_in_table(deps_table, name, version)?;
                        }
                    }
                }

                // Update dev-dependencies
                if let Some(deps) = document.get_mut("dev-dependencies") {
                    if let Some(deps_table) = deps.as_table_mut() {
                        for (name, version) in &dev_updates {
                            self.update_dependency_in_table(deps_table, name, version)?;
                        }
                    }
                }

                // Update build-dependencies
                if let Some(deps) = document.get_mut("build-dependencies") {
                    if let Some(deps_table) = deps.as_table_mut() {
                        for (name, version) in &build_updates {
                            self.update_dependency_in_table(deps_table, name, version)?;
                        }
                    }
                }

                // Update the source with the new content
                source.update_content(document.to_string());

                Ok(())
            }
            _ => Err(anyhow!("Not a Cargo.toml source")),
        }
    }

    fn write(&self, source: &DependencySource) -> Result<PendingWrite> {
        match source {
            DependencySource::CargoToml { path, content, .. } => {
                let path = path.clone();
                let content = content.clone();

                // Create a future that will write the file asynchronously
                let write_future = async move {
                    fs::write(path, content).await?;
                    Ok(())
                };

                // Return a domain-specific type that can be awaited
                Ok(PendingWrite::new(write_future))
            }
            _ => Err(anyhow!("Not a Cargo.toml source")),
        }
    }
}

impl CargoWriter {
    /// Update a dependency version in a TOML table
    fn update_dependency_in_table(
        &self,
        table: &mut toml_edit::Table,
        name: &str,
        version: &str,
    ) -> Result<()> {
        if let Some(item) = table.get_mut(name) {
            match item {
                // Simple string version
                Item::Value(value) => {
                    if value.is_str() {
                        *value = Value::String(toml_edit::Formatted::new(version.to_string()));
                    }
                }

                // Table format
                Item::Table(dep_table) => {
                    // Skip if it's a workspace dependency
                    if dep_table.contains_key("workspace") {
                        return Ok(());
                    }

                    if let Some(ver_item) = dep_table.get_mut("version") {
                        if let Some(ver_value) = ver_item.as_value_mut() {
                            if ver_value.is_str() {
                                *ver_value =
                                    Value::String(toml_edit::Formatted::new(version.to_string()));
                            }
                        }
                    }
                }

                // Other formats not supported
                _ => {}
            }
        }

        Ok(())
    }
}
