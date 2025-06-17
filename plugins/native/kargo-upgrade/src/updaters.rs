use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;
use tokio::fs;

use crate::crates_io::get_latest_version;
use crate::models::{Dependency, DependencyLocation, DependencyUpdate};
use crate::types::UpdateOptions;

// Pre-compile regex patterns
static CARGO_SECTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"```cargo\n([\s\S]*?)```").expect("Invalid cargo section regex"));
static CARGO_DEPS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"//\s*cargo-deps:\s*(.+)$").expect("Invalid cargo deps regex"));

/// Update dependencies in a cargo.toml file
pub async fn update_cargo_toml(
    path: &Path,
    updates: Vec<DependencyUpdate>,
    _options: &UpdateOptions,
) -> Result<()> {
    let content = fs::read_to_string(path).await?;
    let mut document = content.parse::<toml_edit::DocumentMut>()?;

    for update in updates {
        // Update based on dependency location
        match &update.dependency.location {
            DependencyLocation::CargoTomlDirect => {
                if let Some(deps) = document.get_mut("dependencies") {
                    if let Some(dep) = deps.get_mut(&update.name) {
                        update_dependency_version(dep, &update.to_version);
                    }
                }
            }
            DependencyLocation::CargoTomlDev => {
                if let Some(deps) = document.get_mut("dev-dependencies") {
                    if let Some(dep) = deps.get_mut(&update.name) {
                        update_dependency_version(dep, &update.to_version);
                    }
                }
            }
            DependencyLocation::CargoTomlBuild => {
                if let Some(deps) = document.get_mut("build-dependencies") {
                    if let Some(dep) = deps.get_mut(&update.name) {
                        update_dependency_version(dep, &update.to_version);
                    }
                }
            }
            _ => {
                // Skip non-cargo.toml updates
                continue;
            }
        }
    }

    // Write the updated content back
    fs::write(path, document.to_string()).await?;
    Ok(())
}

/// Update a dependency version in a TOML value
fn update_dependency_version(value: &mut toml_edit::Item, new_version: &str) {
    match value {
        toml_edit::Item::Value(val) => {
            // Simple format: name = "version"
            *val = toml_edit::Value::from(new_version);
        }
        toml_edit::Item::Table(table) => {
            // Table format: name = { version = "version", ... }
            if let Some(version_item) = table.get_mut("version") {
                *version_item = toml_edit::Item::Value(toml_edit::Value::from(new_version));
            }
        }
        _ => {
            // Unexpected format, skip
        }
    }
}

/// Update dependencies in Cargo manifest within a Rust file
pub async fn update_cargo_manifest_in_rust(
    path: &Path,
    updates: Vec<DependencyUpdate>,
    _options: &UpdateOptions,
) -> Result<()> {
    let content = fs::read_to_string(path).await?;
    let mut updated_content = content.clone();

    // Process updates by location type
    for update in updates {
        match &update.dependency.location {
            DependencyLocation::RustScriptCargo { .. } => {
                // Handle Cargo manifest updates
                if let Some(captures) = CARGO_SECTION_REGEX.captures(&content) {
                    if let Some(cargo_section) = captures.get(1) {
                        let original_cargo = cargo_section.as_str();
                        let updated_cargo = update_cargo_section(original_cargo, &update)?;

                        let full_section = captures
                            .get(0)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get full cargo section"))?;
                        let new_section = format!("```cargo\n{}```", updated_cargo);

                        updated_content =
                            updated_content.replace(full_section.as_str(), &new_section);
                    }
                }
            }
            _ => {
                // Skip non-manifest updates
                continue;
            }
        }
    }

    // Write the updated content back
    fs::write(path, updated_content).await?;
    Ok(())
}

fn update_cargo_section(cargo_content: &str, update: &DependencyUpdate) -> Result<String> {
    let mut doc = cargo_content
        .parse::<toml_edit::DocumentMut>()
        .context("Failed to parse cargo section as TOML")?;

    // Update in dependencies section
    if let Some(deps) = doc.get_mut("dependencies") {
        if let Some(dep) = deps.get_mut(&update.name) {
            update_dependency_version(dep, &update.to_version);
        }
    }

    // Update in dev-dependencies section
    if let Some(deps) = doc.get_mut("dev-dependencies") {
        if let Some(dep) = deps.get_mut(&update.name) {
            update_dependency_version(dep, &update.to_version);
        }
    }

    Ok(doc.to_string())
}

/// Update dependencies in rust script files
pub async fn update_rust_script(
    path: &Path,
    updates: Vec<DependencyUpdate>,
    _options: &UpdateOptions,
) -> Result<()> {
    let content = fs::read_to_string(&path).await?;
    let mut updated_content = content.clone();

    // Process rust script format updates
    for update in &updates {
        match &update.dependency.location {
            DependencyLocation::RustScriptCargo { section_range } => {
                updated_content =
                    update_rust_script_cargo_section(&updated_content, section_range, update)?;
            }
            DependencyLocation::RustScriptDeps { line_range } => {
                updated_content =
                    update_rust_script_cargo_deps_line(&updated_content, line_range, update)?;
            }
            _ => continue,
        }
    }

    // Write the updated content back
    fs::write(path, updated_content).await?;
    Ok(())
}

fn update_rust_script_cargo_section(
    content: &str,
    section_range: &(usize, usize),
    update: &DependencyUpdate,
) -> Result<String> {
    // Extract the cargo section
    let section = &content[section_range.0..section_range.1];

    // Update the dependency in the section
    let updated_section = update_dependency_in_text(
        section,
        &update.name,
        &update.from_version,
        &update.to_version,
    )?;

    // Replace the section in the content
    let mut result = content.to_string();
    result.replace_range(section_range.0..section_range.1, &updated_section);
    Ok(result)
}

fn update_rust_script_cargo_deps_line(
    content: &str,
    line_range: &(usize, usize),
    update: &DependencyUpdate,
) -> Result<String> {
    // Extract the line
    let line = &content[line_range.0..line_range.1];

    // Update the dependency in the line
    let updated_line = update_dependency_in_deps_line(
        line,
        &update.name,
        &update.from_version,
        &update.to_version,
    )?;

    // Replace the line in the content
    let mut result = content.to_string();
    result.replace_range(line_range.0..line_range.1, &updated_line);
    Ok(result)
}

fn update_dependency_in_text(
    text: &str,
    name: &str,
    current_version: &str,
    new_version: &str,
) -> Result<String> {
    let mut result = text.to_string();

    // Try different patterns
    let patterns = vec![
        // Simple format: name = "version"
        format!(
            r#"{}\s*=\s*["']{}["']"#,
            regex::escape(name),
            regex::escape(current_version)
        ),
        // Table format with version
        format!(
            r#"version\s*=\s*["']{}["']"#,
            regex::escape(current_version)
        ),
    ];

    for pattern in patterns {
        let regex = Regex::new(&pattern)?;
        if regex.is_match(&result) {
            let replacement = if pattern.contains("version\\s*=") {
                format!("version = \"{}\"", new_version)
            } else {
                format!("{} = \"{}\"", name, new_version)
            };
            result = regex.replace(&result, replacement.as_str()).to_string();
            break;
        }
    }

    Ok(result)
}

fn update_dependency_in_deps_line(
    line: &str,
    name: &str,
    _current_version: &str,
    new_version: &str,
) -> Result<String> {
    // Handle various formats in cargo-deps line
    let patterns = vec![
        (
            format!(r#"{}=["']([^"']+)["']"#, regex::escape(name)),
            format!("{}=\"{}\"", name, new_version),
        ),
        (
            format!(r#"{}\s*=\s*["']([^"']+)["']"#, regex::escape(name)),
            format!("{} = \"{}\"", name, new_version),
        ),
    ];

    let mut result = line.to_string();
    for (pattern, replacement) in patterns {
        let regex = Regex::new(&pattern)?;
        if regex.is_match(&result) {
            result = regex.replace(&result, replacement.as_str()).to_string();
            break;
        }
    }

    Ok(result)
}

/// Update dependencies in a markdown file
pub async fn update_markdown(
    path: &Path,
    _updates: Vec<DependencyUpdate>,
    options: &UpdateOptions,
) -> Result<()> {
    let content = fs::read_to_string(&path).await?;
    let mut updated_content = content.clone();

    // 1. Handle embedded cargo manifest format: ```cargo ... ```
    if let Some(captures) = CARGO_SECTION_REGEX.captures(&content) {
        if let Some(cargo_section) = captures.get(1) {
            let cargo_content = cargo_section.as_str();
            let cargo_section_span = cargo_section.range();

            // Parse the cargo section as TOML
            let mut updated_cargo_content = cargo_content.to_string();

            // Helper function to update dependencies in a section
            async fn update_deps_in_section(
                section_name: &str,
                content: &str,
                _options: &UpdateOptions,
            ) -> Result<Vec<DependencyUpdate>> {
                let mut section_updates = Vec::new();
                let deps_section_regex =
                    Regex::new(&format!(r"(?s)\[{}\](.*?)(?:\n\s*\[|\z)", section_name))?;

                if let Some(deps_section) = deps_section_regex.captures(content) {
                    let deps_content = deps_section
                        .get(1)
                        .ok_or_else(|| anyhow::anyhow!("Failed to get deps content"))?
                        .as_str();

                    // First, handle simple format: name = "version"
                    let simple_dep_regex = Regex::new(r#"(?m)^(\w+)\s*=\s*["']([^"']+)["']"#)?;
                    for cap in simple_dep_regex.captures_iter(deps_content) {
                        let name = cap
                            .get(1)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                            .as_str();
                        let version = cap
                            .get(2)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                            .as_str();

                        // Get the latest version
                        if let Ok(Some(latest)) = get_latest_version(name).await {
                            if version != latest {
                                let dummy_dep = Dependency {
                                    name: name.to_string(),
                                    version: version.to_string(),
                                    location: DependencyLocation::CargoTomlDirect,
                                };

                                section_updates.push(DependencyUpdate {
                                    name: name.to_string(),
                                    from_version: version.to_string(),
                                    to_version: latest,
                                    dependency: dummy_dep,
                                });
                            }
                        }
                    }

                    // Second, handle table format: name = { version = "version", ... }
                    let table_dep_regex =
                        Regex::new(r#"(?ms)^(\w+)\s*=\s*\{(.*?)version\s*=\s*["']([^"']+)["']"#)?;
                    for cap in table_dep_regex.captures_iter(deps_content) {
                        let name = cap
                            .get(1)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                            .as_str();
                        let version = cap
                            .get(3)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                            .as_str();

                        // Get the latest version
                        if let Ok(Some(latest)) = get_latest_version(name).await {
                            if version != latest {
                                let dummy_dep = Dependency {
                                    name: name.to_string(),
                                    version: version.to_string(),
                                    location: DependencyLocation::CargoTomlDirect,
                                };

                                section_updates.push(DependencyUpdate {
                                    name: name.to_string(),
                                    from_version: version.to_string(),
                                    to_version: latest,
                                    dependency: dummy_dep,
                                });
                            }
                        }
                    }
                }

                Ok(section_updates)
            }

            // Look for and update dependencies in different sections
            let mut all_updates = Vec::new();

            // Check dependencies section
            if let Ok(deps_updates) =
                update_deps_in_section("dependencies", cargo_content, options).await
            {
                all_updates.extend(deps_updates);
            }

            // Check dev-dependencies section
            if let Ok(dev_deps_updates) =
                update_deps_in_section("dev-dependencies", cargo_content, options).await
            {
                all_updates.extend(dev_deps_updates);
            }

            // Check build-dependencies section
            if let Ok(build_deps_updates) =
                update_deps_in_section("build-dependencies", cargo_content, options).await
            {
                all_updates.extend(build_deps_updates);
            }

            // Apply all updates to the cargo content
            for update in all_updates {
                // Update simple format
                let simple_regex = Regex::new(&format!(
                    r#"({}\s*=\s*["']){}(["'])"#,
                    regex::escape(&update.name),
                    regex::escape(&update.from_version)
                ))?;
                updated_cargo_content = simple_regex
                    .replace_all(
                        &updated_cargo_content,
                        format!("${{1}}{}${{2}}", update.to_version).as_str(),
                    )
                    .to_string();

                // Update table format
                let table_regex = Regex::new(&format!(
                    r#"(version\s*=\s*["']){}(["'])"#,
                    regex::escape(&update.from_version)
                ))?;
                updated_cargo_content = table_regex
                    .replace_all(
                        &updated_cargo_content,
                        format!("${{1}}{}${{2}}", update.to_version).as_str(),
                    )
                    .to_string();
            }

            // Replace the cargo section in the content
            let _original_section = &content[cargo_section_span.clone()];
            updated_content.replace_range(cargo_section_span, &updated_cargo_content);
        }
    }

    // 2. Handle single-line cargo-deps format
    if let Some(captures) = CARGO_DEPS_REGEX.captures(&content) {
        if let Some(deps_match) = captures.get(1) {
            let deps_str = deps_match.as_str();
            let _deps_range = deps_match.range();

            // Parse dependencies from the line
            let mut updated_deps = deps_str.to_string();

            // First, handle dependencies with versions
            let dep_regex = Regex::new(r#"(\w+)\s*=\s*["']([^"']+)["']"#)?;
            for cap in dep_regex.captures_iter(deps_str) {
                let name = cap
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                    .as_str();
                let version = cap
                    .get(2)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                    .as_str();

                // Get the latest version
                if let Ok(Some(latest)) = get_latest_version(name).await {
                    if version != latest {
                        // Update in the deps string
                        let replace_regex = Regex::new(&format!(
                            r#"({}\s*=\s*["']){}(["'])"#,
                            regex::escape(name),
                            regex::escape(version)
                        ))?;
                        updated_deps = replace_regex
                            .replace(&updated_deps, format!("${{1}}{}${{2}}", latest).as_str())
                            .to_string();
                    }
                }
            }

            // Also handle format without quotes: name=version
            let bare_regex = Regex::new(r#"(\w+)=([^\s,]+)"#)?;
            for cap in bare_regex.captures_iter(deps_str) {
                let name = cap
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                    .as_str();

                // Skip if this was already handled with quotes
                if dep_regex.is_match(&format!("{} = ", name)) {
                    continue;
                }

                let version = cap
                    .get(2)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                    .as_str();

                // Get the latest version
                if let Ok(Some(latest)) = get_latest_version(name).await {
                    if version != latest {
                        // Update in the deps string
                        let replace_regex = Regex::new(&format!(
                            r#"{}={}"#,
                            regex::escape(name),
                            regex::escape(version)
                        ))?;
                        updated_deps = replace_regex
                            .replace(&updated_deps, format!("{}={}", name, latest).as_str())
                            .to_string();
                    }
                }
            }

            // Handle bare dependency names (no version specified)
            // Format: cargo-deps: dep1, dep2, dep3
            let bare_deps_regex = Regex::new(r"(?:^|,)\s*(\w+)(?:\s*,|$)")?;
            for cap in bare_deps_regex.captures_iter(deps_str) {
                let name = cap
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                    .as_str();

                // Skip if this dependency already has a version specified
                if dep_regex
                    .captures_iter(deps_str)
                    .any(|c| c.get(1).map(|m| m.as_str()) == Some(name))
                {
                    continue;
                }

                // Get the latest version
                if let Ok(Some(latest)) = get_latest_version(name).await {
                    // Replace bare dependency with versioned one
                    let bare_dep_pattern = format!(r"(?:^|,)\s*{}(?:\s*,|$)", regex::escape(name));
                    let bare_dep_regex = Regex::new(&bare_dep_pattern)?;

                    // Determine the replacement based on context
                    if deps_str.contains('=') {
                        // Other deps have versions, match that format
                        updated_deps = bare_dep_regex
                            .replace(
                                &updated_deps,
                                format!(r#", {} = "{}", "#, name, latest).as_str(),
                            )
                            .to_string();
                    } else {
                        // This might be the only dep or all are bare
                        updated_deps = bare_dep_regex
                            .replace(&updated_deps, format!(r#"{}="{}""#, name, latest).as_str())
                            .to_string();
                    }
                }
            }

            // Replace the deps string in the content
            // We need to replace within the full match to preserve the comment prefix
            let full_match = captures
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Failed to get full match"))?;
            let _full_str = full_match.as_str();

            // Find the full line containing cargo-deps
            let line_regex = Regex::new(r"(?m)^.*cargo-deps:.*$")?;
            if let Some(line_match) = line_regex.find(&content) {
                let line_str = line_match.as_str();
                let updated_line = line_str.replace(deps_str, &updated_deps);
                updated_content = updated_content.replace(line_str, &updated_line);
            }
        }
    }

    // Write the updated content back
    fs::write(path, updated_content).await?;
    Ok(())
}
