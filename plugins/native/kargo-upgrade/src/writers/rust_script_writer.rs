//! Writer for Rust script files

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::fs;

use crate::models::{DependencyLocation, DependencySource, DependencyUpdate, DependencyWriter};
use crate::types::PendingWrite;

// Regex patterns for replacements
static VERSION_PATTERN: Lazy<String> = Lazy::new(|| r#"({}\s*=\s*)["']{}["']"#.to_string());
static TABLE_VERSION_PATTERN: Lazy<String> =
    Lazy::new(|| r#"({}.*?version\s*=\s*)["']{}["']"#.to_string());
static BARE_DEP_PATTERN: Lazy<String> = Lazy::new(|| r"(^|,)\s*{}\s*(,|$)".to_string());

/// Writer for Rust script files
#[derive(Clone)]
pub struct RustScriptWriter;

impl DependencyWriter for RustScriptWriter {
    fn apply_updates(
        &self,
        source: &mut DependencySource,
        updates: &[DependencyUpdate],
    ) -> Result<()> {
        match source {
            DependencySource::RustScript { content, .. } => {
                let mut updated_content = content.clone();

                for update in updates {
                    match &update.dependency.location {
                        DependencyLocation::RustScriptCargo { section_range } => {
                            // Extract the cargo section
                            let section_content = &content[section_range.0..section_range.1];
                            let mut updated_section = section_content.to_string();

                            // Update the version in the section
                            self.update_version_in_cargo_section(
                                &mut updated_section,
                                &update.name,
                                &update.from_version,
                                &update.to_version,
                            )?;

                            // Replace the section in the content
                            updated_content
                                .replace_range(section_range.0..section_range.1, &updated_section);
                        }
                        DependencyLocation::RustScriptDeps { line_range } => {
                            // Extract the cargo-deps line
                            let line_content = &content[line_range.0..line_range.1];
                            let mut updated_line = line_content.to_string();

                            // Update or add the version
                            if update.from_version == "none" {
                                // This was a bare dependency, add the version
                                self.add_version_to_bare_dep(
                                    &mut updated_line,
                                    &update.name,
                                    &update.to_version,
                                )?;
                            } else {
                                // This had a version, update it
                                self.update_version_in_cargo_deps(
                                    &mut updated_line,
                                    &update.name,
                                    &update.from_version,
                                    &update.to_version,
                                )?;
                            }

                            // Replace the line in the content
                            updated_content
                                .replace_range(line_range.0..line_range.1, &updated_line);
                        }
                        _ => {} // Ignore other location types
                    }
                }

                // Update the source with the new content
                source.update_content(updated_content);

                Ok(())
            }
            _ => Err(anyhow!("Not a Rust script source")),
        }
    }

    fn write(&self, source: &DependencySource) -> Result<PendingWrite> {
        match source {
            DependencySource::RustScript { path, content, .. } => {
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
            _ => Err(anyhow!("Not a Rust script source")),
        }
    }
}

impl RustScriptWriter {
    /// Update a version in a ```cargo section
    fn update_version_in_cargo_section(
        &self,
        content: &mut String,
        name: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<()> {
        // Update simple format: name = "version"
        let pattern_str = VERSION_PATTERN.as_str();
        let simple_pattern = format!("{}", pattern_str)
            .replace("{}", &regex::escape(name))
            .replace("{}", &regex::escape(from_version));
        let simple_regex = Regex::new(&simple_pattern)?;
        *content = simple_regex
            .replace_all(content, &format!("${{1}}\"{}\"", to_version))
            .to_string();

        // Update table format: name = { version = "version", ... }
        let table_str = TABLE_VERSION_PATTERN.as_str();
        let table_pattern = format!("{}", table_str)
            .replace("{}", &regex::escape(name))
            .replace("{}", &regex::escape(from_version));
        let table_regex = Regex::new(&table_pattern)?;
        *content = table_regex
            .replace_all(content, &format!("${{1}}\"{}\"", to_version))
            .to_string();

        Ok(())
    }

    /// Update a version in a cargo-deps: line
    fn update_version_in_cargo_deps(
        &self,
        content: &mut String,
        name: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<()> {
        // Handle different spacing patterns
        let patterns = [
            format!("{}=\"{}\"", name, from_version),
            format!("{} = \"{}\"", name, from_version),
            format!("{}= \"{}\"", name, from_version),
            format!("{}=\"{}\"", name, from_version),
        ];

        let new_spec = format!("{}=\"{}\"", name, to_version);

        // Try each pattern for replacement
        let mut replaced = false;
        for pattern in patterns.iter() {
            if content.contains(pattern) {
                *content = content.replace(pattern, &new_spec);
                replaced = true;
                break;
            }
        }

        // If none of our patterns matched exactly, use regex for replacement
        if !replaced {
            let pattern_str = VERSION_PATTERN.as_str();
            let version_pattern = format!("{}", pattern_str)
                .replace("{}", &regex::escape(name))
                .replace("{}", &regex::escape(from_version));
            let version_regex = Regex::new(&version_pattern)?;
            *content = version_regex
                .replace_all(content, &format!("${{1}}\"{}\"", to_version))
                .to_string();
        }

        Ok(())
    }

    /// Add a version to a bare dependency
    fn add_version_to_bare_dep(
        &self,
        content: &mut String,
        name: &str,
        version: &str,
    ) -> Result<()> {
        let bare_str = BARE_DEP_PATTERN.as_str();
        let bare_pattern = format!("{}", bare_str).replace("{}", &regex::escape(name));
        let bare_regex = Regex::new(&bare_pattern)?;

        // Check how the dependency is specified in the content
        if content.contains(&format!("{},", name)) {
            // name,
            *content = bare_regex
                .replace(content, &format!("${{1}}{}=\"{}\",", name, version))
                .to_string();
        } else if content.contains(&format!("{} ,", name)) {
            // name ,
            *content = bare_regex
                .replace(content, &format!("${{1}}{}=\"{}\" ,", name, version))
                .to_string();
        } else if content.contains(&format!(", {}", name)) {
            // , name
            *content = bare_regex
                .replace(content, &format!("${{1}}, {}=\"{}\"", name, version))
                .to_string();
        } else {
            // Just name
            *content = bare_regex
                .replace(content, &format!("${{1}}{}=\"{}\"${{2}}", name, version))
                .to_string();
        }

        Ok(())
    }
}
