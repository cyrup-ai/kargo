//! Module containing the update logic for different file types

use crate::{
    types::{CrateType, DependencyUpdate, UpdateOptions, UpdateResult},
    models::{Dependency, DependencyLocation},
    crates_io::get_latest_version,
};
use anyhow::Result;
use regex::Regex;
use std::path::Path;
use tokio::fs;
use toml_edit::{DocumentMut, Item, Value as TomlValue};

/// Update dependencies in a Cargo.toml file
pub async fn update_cargo_toml(path: impl AsRef<Path>, options: &UpdateOptions) -> Result<UpdateResult> {
    let path = path.as_ref().to_owned();

    // Read the Cargo.toml content
    let content = fs::read_to_string(&path).await?;
    let mut document = content.parse::<DocumentMut>()?;

    // Track updates for reporting
    let mut updates = Vec::new();

    // Check if this is a workspace Cargo.toml
    let is_workspace = document.get("workspace").is_some();
    let crate_type = if is_workspace {
        CrateType::Workspace
    } else {
        CrateType::Standard
    };

    // Update workspace dependencies if this is a workspace and the option is enabled
    if is_workspace && options.update_workspace {
        if let Some(workspace_deps) = document.get_mut("workspace.dependencies") {
            if let Some(workspace_deps_table) = workspace_deps.as_table_mut() {
                // Convert the iterator to a Vec first to make it Send
                let deps: Vec<_> = workspace_deps_table.iter_mut()
                    .map(|(key, value)| (key.to_string(), value))
                    .collect();

                for (name, value) in deps {
                    if let Some(update) = update_dependency(&name, value, DependencyLocation::CargoTomlDirect).await? {
                        updates.push(update);
                    }
                }
            }
        }
    }

    // Update regular dependencies
    if let Some(deps) = document.get_mut("dependencies") {
        if let Some(deps_table) = deps.as_table_mut() {
            // Convert the iterator to a Vec first to make it Send
            let deps: Vec<_> = deps_table.iter_mut()
                .map(|(key, value)| (key.to_string(), value))
                .collect();

            for (name, value) in deps {
                // Skip workspace dependencies
                if value.as_table().and_then(|t| t.get("workspace")).is_some() {
                    continue;
                }

                if let Some(update) = update_dependency(&name, value, DependencyLocation::CargoTomlDirect).await? {
                    updates.push(update);
                }
            }
        }
    }

    // Update dev-dependencies
    if let Some(deps) = document.get_mut("dev-dependencies") {
        if let Some(deps_table) = deps.as_table_mut() {
            // Convert the iterator to a Vec first to make it Send
            let deps: Vec<_> = deps_table.iter_mut()
                .map(|(key, value)| (key.to_string(), value))
                .collect();

            for (name, value) in deps {
                if let Some(update) = update_dependency(&name, value, DependencyLocation::CargoTomlDev).await? {
                    updates.push(update);
                }
            }
        }
    }

    // If we made any updates, write the changes back to disk
    if !updates.is_empty() {
        fs::write(&path, document.to_string()).await?;
    }

    Ok(UpdateResult {
        path,
        updates,
        crate_type,
        error: None,
    })
}

/// Update a single dependency to its latest version
async fn update_dependency(name: &str, value: &mut Item, location: DependencyLocation) -> Result<Option<DependencyUpdate>> {
    // Extract the current version
    let current_version = match value {
        // Simple string version like version = "1.0.0"
        Item::Value(TomlValue::String(version)) => version.to_string(),

        // Table specification like version = { version = "1.0.0", features = ["..."]] }
        Item::Table(table) => {
            if let Some(version) = table.get("version") {
                if let Some(ver_str) = version.as_str() {
                    ver_str.to_string()
                } else {
                    return Ok(None); // Not a string version
                }
            } else {
                return Ok(None); // No version key
            }
        },

        // Other formats not supported
        _ => return Ok(None),
    };

    // Get the latest version
    if let Some(latest_version) = get_latest_version(name).await? {
        // Skip if already at latest version
        if current_version == latest_version {
            return Ok(None);
        }

        // Update the version in the toml
        match value {
            Item::Value(TomlValue::String(version)) => {
                *version = toml_edit::Formatted::new(latest_version.clone());
            },
            Item::Table(table) => {
                if let Some(version_item) = table.get_mut("version") {
                    if let Some(version_str) = version_item.as_value_mut() {
                        *version_str = TomlValue::String(toml_edit::Formatted::new(latest_version.clone()));
                    }
                }
            },
            _ => {}, // Shouldn't reach here
        }

        // Return the update information
        Ok(Some(DependencyUpdate {
            name: name.to_string(),
            from_version: current_version.clone(),
            to_version: latest_version,
            dependency: Dependency {
                name: name.to_string(),
                version: current_version,
                location,
            },
        }))
    } else {
        Ok(None)
    }
}

/// Update dependencies in a rust-script file
pub async fn update_rust_script(path: impl AsRef<Path>, options: &UpdateOptions) -> Result<UpdateResult> {
    let path = path.as_ref().to_owned();
    let content = fs::read_to_string(&path).await?;
    let mut updates = Vec::new();
    let mut updated_content = content.clone();

    // 1. Handle embedded cargo manifest format: ```cargo ... ```
    let cargo_section_regex = Regex::new(r"```cargo\n([\s\S]*?)```").unwrap();
    if let Some(captures) = cargo_section_regex.captures(&content) {
        if let Some(cargo_section) = captures.get(1) {
            let cargo_content = cargo_section.as_str();
            let cargo_section_span = cargo_section.range();

            // Parse the cargo section as TOML
            let mut updated_cargo_content = cargo_content.to_string();

            // Helper function to update dependencies in a section
            async fn update_deps_in_section(section_name: &str, content: &str, _options: &UpdateOptions) -> Result<Vec<DependencyUpdate>> {
                let mut section_updates = Vec::new();
                let deps_section_regex = Regex::new(&format!(r"(?s)\[{}\](.*?)(?:\n\s*\[|\z)", section_name)).unwrap();

                if let Some(deps_section) = deps_section_regex.captures(content) {
                    let deps_content = deps_section.get(1).unwrap().as_str();

                    // First, handle simple format: name = "version"
                    let simple_dep_regex = Regex::new(r#"(?m)^(\w+)\s*=\s*["']([^"']+)["']"#).unwrap();
                    for cap in simple_dep_regex.captures_iter(deps_content) {
                        let name = cap.get(1).unwrap().as_str();
                        let version = cap.get(2).unwrap().as_str();

                        // Get the latest version
                        if let Ok(Some(latest)) = get_latest_version(name).await {
                            if version != latest {
                                section_updates.push(DependencyUpdate {
                                    name: name.to_string(),
                                    from_version: version.to_string(),
                                    to_version: latest.clone(),
                                    dependency: Dependency {
                                        name: name.to_string(),
                                        version: version.to_string(),
                                        location: DependencyLocation::RustScriptCargo {
                                            section_range: (0, 0), // Will be updated later
                                        },
                                    },
                                });
                            }
                        }
                    }

                    // Second, handle table format: name = { version = "version", ... }
                    let table_dep_regex = Regex::new(r#"(?sm)^(\w+)\s*=\s*\{(.*?)version\s*=\s*["']([^"']+)["']"#).unwrap();
                    for cap in table_dep_regex.captures_iter(deps_content) {
                        let name = cap.get(1).unwrap().as_str();
                        let version = cap.get(3).unwrap().as_str();

                        // Get the latest version
                        if let Ok(Some(latest)) = get_latest_version(name).await {
                            if version != latest {
                                section_updates.push(DependencyUpdate {
                                    name: name.to_string(),
                                    from_version: version.to_string(),
                                    to_version: latest.clone(),
                                    dependency: Dependency {
                                        name: name.to_string(),
                                        version: version.to_string(),
                                        location: DependencyLocation::RustScriptCargo {
                                            section_range: (0, 0), // Will be updated later
                                        },
                                    },
                                });
                            }
                        }
                    }
                }

                Ok(section_updates)
            }

            // Update dependencies section
            let mut all_updates = update_deps_in_section("dependencies", &cargo_content, options).await?;

            // Update dev-dependencies section
            let dev_updates = update_deps_in_section("dev-dependencies", &cargo_content, options).await?;
            all_updates.extend(dev_updates);

            // Apply all the updates to the content
            for update in &all_updates {
                // Update simple version format
                let simple_regex = Regex::new(&format!(r#"({}\s*=\s*)["']{}["']"#,
                                                    regex::escape(&update.name),
                                                    regex::escape(&update.from_version))).unwrap();
                updated_cargo_content = simple_regex.replace_all(&updated_cargo_content,
                    &format!("${{1}}\"{}\"", update.to_version)).to_string();

                // Update table format
                let table_regex = Regex::new(&format!(r#"(?s)({}\s*=\s*\{{.*?version\s*=\s*)["']{}["']"#,
                                                   regex::escape(&update.name),
                                                   regex::escape(&update.from_version))).unwrap();
                updated_cargo_content = table_regex.replace_all(&updated_cargo_content,
                    &format!("${{1}}\"{}\"", update.to_version)).to_string();
            }

            updates.extend(all_updates);

            // If we made updates to the cargo section, update the content
            if !updates.is_empty() {
                updated_content.replace_range(cargo_section_span.start..cargo_section_span.end, &updated_cargo_content);
            }
        }
    }

    // 2. Handle single-line cargo-deps format
    let cargo_deps_regex = Regex::new(r"//\s*cargo-deps:\s*(.+)$").unwrap();
    if let Some(captures) = cargo_deps_regex.captures(&content) {
        if let Some(deps_match) = captures.get(1) {
            let deps_str = deps_match.as_str();
            let _deps_span = deps_match.range();

            // Parse the dependencies string
            let mut updated_deps = deps_str.to_string();

            // First, handle dependencies with versions
            let dep_regex = Regex::new(r#"(\w+)\s*=\s*["']([^"']+)["']"#).unwrap();
            for cap in dep_regex.captures_iter(deps_str) {
                let name = cap.get(1).unwrap().as_str();
                let version = cap.get(2).unwrap().as_str();

                // Get the latest version
                if let Ok(Some(latest)) = get_latest_version(name).await {
                    if version != latest {
                        let update = DependencyUpdate {
                            name: name.to_string(),
                            from_version: version.to_string(),
                            to_version: latest.clone(),
                            dependency: Dependency {
                                name: name.to_string(),
                                version: version.to_string(),
                                location: DependencyLocation::RustScriptDeps {
                                    line_range: (0, 0), // Will be calculated
                                },
                            },
                        };

                        // Handle different spacing patterns in the replacement
                        let patterns = [
                            format!("{}=\"{}\"", name, version),
                            format!("{} = \"{}\"", name, version),
                            format!("{}= \"{}\"", name, version),
                            format!("{}=\"{}\"", name, version),
                        ];

                        let new_spec = format!("{}=\"{}\"", name, latest);

                        // Try each pattern for replacement
                        let mut replaced = false;
                        for pattern in patterns.iter() {
                            if updated_deps.contains(pattern) {
                                updated_deps = updated_deps.replace(pattern, &new_spec);
                                replaced = true;
                                break;
                            }
                        }

                        // If none of our patterns matched exactly, use regex for replacement
                        if !replaced {
                            let replace_regex = Regex::new(&format!(r#"({}\s*=\s*)["']{}["']"#,
                                                                  regex::escape(name),
                                                                  regex::escape(version))).unwrap();
                            updated_deps = replace_regex.replace(&updated_deps,
                                                              &format!("${{1}}\"{}\"", latest)).to_string();
                        }

                        updates.push(update);
                    }
                }
            }

            // Then handle bare dependencies (no version specified)
            // Format: cargo-deps: dep1, dep2, dep3
            let bare_deps_regex = Regex::new(r"(?:^|,)\s*(\w+)(?:\s*,|$)").unwrap();
            for cap in bare_deps_regex.captures_iter(deps_str) {
                let name = cap.get(1).unwrap().as_str();

                // Skip if this dependency already has a version
                if dep_regex.captures_iter(deps_str)
                    .any(|cap| cap.get(1).unwrap().as_str() == name) {
                    continue;
                }

                // Get the latest version
                if let Ok(Some(latest)) = get_latest_version(name).await {
                    let update = DependencyUpdate {
                        name: name.to_string(),
                        from_version: "none".to_string(),
                        to_version: latest.clone(),
                        dependency: Dependency {
                            name: name.to_string(),
                            version: "none".to_string(),
                            location: DependencyLocation::RustScriptDeps {
                                line_range: (0, 0), // Will be calculated
                            },
                        },
                    };

                    // Replace the bare dependency with a versioned one
                    let bare_dep_pattern = format!(r"(?:^|,)\s*{}(?:\s*,|$)", regex::escape(name));
                    let bare_dep_regex = Regex::new(&bare_dep_pattern).unwrap();

                    // Keep the comma if it exists
                    let replacement = if deps_str.contains(&format!("{},", name)) {
                        format!("{}=\"{}\",", name, latest)
                    } else if deps_str.contains(&format!("{} ,", name)) {
                        format!("{}=\"{}\" ,", name, latest)
                    } else if deps_str.contains(&format!(", {}", name)) {
                        format!(", {}=\"{}\"", name, latest)
                    } else {
                        format!("{}=\"{}\"", name, latest)
                    };

                    updated_deps = bare_dep_regex.replace(&updated_deps, replacement.as_str()).to_string();
                    updates.push(update);
                }
            }

            // If we made updates to the deps, update the content
            if !updates.is_empty() {
                // Find the full line containing cargo-deps
                let line_regex = Regex::new(&format!(r"//\s*cargo-deps:\s*{}", regex::escape(deps_str))).unwrap();
                if let Some(line_match) = line_regex.find(&content) {
                    let line_span = line_match.range();
                    let new_line = format!("// cargo-deps: {}", updated_deps);
                    updated_content.replace_range(line_span.start..line_span.end, &new_line);
                }
            }
        }
    }

    // Write updates back to the file if needed
    if !updates.is_empty() {
        fs::write(&path, &updated_content).await?;
    }

    Ok(UpdateResult {
        path,
        updates,
        crate_type: CrateType::RustScript,
        error: None,
    })
}
