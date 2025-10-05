use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use regex::Regex;
use toml_edit::DocumentMut as Document;

use crate::models::{Dependency, DependencyLocation, DependencyParser, DependencySource};

// Regular expressions for parsing rust-script files
// Regex to find embedded cargo TOML sections
// Supports both: //! ```cargo format and standalone ```cargo format
static CARGO_SECTION_DOC_COMMENT_REGEX: Lazy<Regex> = Lazy::new(|| {
    match Regex::new(r"//!\s*```cargo\s*\n((?://!.*\n)*?)//!\s*```") {
        Ok(re) => re,
        Err(e) => {
            eprintln!("FATAL: Failed to compile hardcoded cargo doc comment section regex: {}", e);
            std::process::exit(1);
        }
    }
});

static CARGO_SECTION_STANDALONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    match Regex::new(r"```cargo\s*\n((?:.*\n)*?)```") {
        Ok(re) => re,
        Err(e) => {
            eprintln!("FATAL: Failed to compile hardcoded cargo standalone section regex: {}", e);
            std::process::exit(1);
        }
    }
});

// Regex to find cargo-deps inline format
static CARGO_DEPS_REGEX: Lazy<Regex> = Lazy::new(|| {
    match Regex::new(r"(?m)//\s*cargo-deps:\s*(.+)$") {
        Ok(re) => re,
        Err(e) => {
            eprintln!("FATAL: Failed to compile hardcoded cargo deps regex: {}", e);
            std::process::exit(1);
        }
    }
});

/// Parser for Rust script files
#[derive(Clone)]
pub struct RustScriptParser;

impl DependencyParser for RustScriptParser {
    fn parse(&self, source: &DependencySource) -> Result<Vec<Dependency>> {
        let content = source.content();
        let mut dependencies = Vec::new();

        // Parse embedded cargo manifest sections
        self.parse_cargo_sections(content, &mut dependencies, source)?;

        // Parse single-line cargo-deps format
        self.parse_cargo_deps_line(content, &mut dependencies, source)?;

        Ok(dependencies)
    }
}

impl RustScriptParser {
    fn parse_cargo_sections(
        &self,
        content: &str,
        dependencies: &mut Vec<Dependency>,
        _source: &DependencySource,
    ) -> Result<()> {
        // Parse doc comment format: //! ```cargo
        for captures in CARGO_SECTION_DOC_COMMENT_REGEX.captures_iter(content) {
            if let Some(cargo_content) = captures.get(1) {
                let section_start = cargo_content.start();
                let section_end = cargo_content.end();

                // Strip //! prefixes from each line to get clean TOML
                let toml_content: String = cargo_content.as_str()
                    .lines()
                    .map(|line| line.trim_start_matches("//!").trim_start())
                    .collect::<Vec<_>>()
                    .join("\n");

                self.parse_toml_dependencies(&toml_content, section_start, section_end, dependencies)?;
            }
        }

        // Parse standalone format: ```cargo (no comment prefix)
        for captures in CARGO_SECTION_STANDALONE_REGEX.captures_iter(content) {
            if let Some(cargo_content) = captures.get(1) {
                let section_start = cargo_content.start();
                let section_end = cargo_content.end();
                let toml_content = cargo_content.as_str();

                self.parse_toml_dependencies(toml_content, section_start, section_end, dependencies)?;
            }
        }

        Ok(())
    }

    /// Parse TOML content and extract dependencies
    fn parse_toml_dependencies(
        &self,
        toml_content: &str,
        section_start: usize,
        section_end: usize,
        dependencies: &mut Vec<Dependency>,
    ) -> Result<()> {
        // Parse as TOML document
        let document = toml_content
            .parse::<Document>()
            .map_err(|e| anyhow!("Failed to parse embedded Cargo.toml: {}", e))?;

        // Extract dependencies table
        if let Some(deps_item) = document.get("dependencies")
            && let Some(deps_table) = deps_item.as_table()
        {
            for (name, value) in deps_table.iter() {
                if let Some(version) = self.extract_version(value) {
                    dependencies.push(Dependency {
                        name: name.to_string(),
                        version,
                        location: DependencyLocation::RustScriptCargo {
                            section_range: (section_start, section_end),
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// Extract version from a TOML value (reused logic from CargoParser)
    fn extract_version(&self, value: &toml_edit::Item) -> Option<String> {
        match value {
            toml_edit::Item::Value(toml_edit::Value::String(s)) => {
                Some(s.value().to_string())
            }
            toml_edit::Item::Value(toml_edit::Value::InlineTable(table)) => {
                table.get("version").and_then(|v| {
                    if let toml_edit::Value::String(s) = v {
                        Some(s.value().to_string())
                    } else {
                        None
                    }
                })
            }
            toml_edit::Item::Table(table) => {
                table.get("version").and_then(|v| {
                    if let toml_edit::Item::Value(toml_edit::Value::String(s)) = v {
                        Some(s.value().to_string())
                    } else {
                        None
                    }
                })
            }
            _ => None,
        }
    }

    fn parse_cargo_deps_line(
        &self,
        content: &str,
        dependencies: &mut Vec<Dependency>,
        _source: &DependencySource,
    ) -> Result<()> {
        for captures in CARGO_DEPS_REGEX.captures_iter(content) {
            if let Some(deps_match) = captures.get(1) {
                let deps_str = deps_match.as_str();
                let line_start = captures
                    .get(0)
                    .ok_or_else(|| anyhow!("Failed to get match start"))?
                    .start();
                let line_end = captures
                    .get(0)
                    .ok_or_else(|| anyhow!("Failed to get match end"))?
                    .end();

                // Parse comma-separated dependencies
                // Format: anyhow="1.0", tokio="1.0", serde
                for dep_item in deps_str.split(',') {
                    let dep_item = dep_item.trim();
                    if dep_item.is_empty() {
                        continue;
                    }

                    // Check for name="version" or name = "version" format
                    if let Some(eq_pos) = dep_item.find('=') {
                        let name = dep_item[..eq_pos].trim();
                        let version_part = dep_item[eq_pos + 1..].trim();

                        // Extract version from quotes
                        let version = version_part
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();

                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version,
                            location: DependencyLocation::RustScriptDeps {
                                line_range: (line_start, line_end),
                            },
                        });
                    } else {
                        // Bare dependency name without version
                        dependencies.push(Dependency {
                            name: dep_item.to_string(),
                            version: "*".to_string(),
                            location: DependencyLocation::RustScriptDeps {
                                line_range: (line_start, line_end),
                            },
                        });
                    }
                }
            }
        }
        Ok(())
    }
}
