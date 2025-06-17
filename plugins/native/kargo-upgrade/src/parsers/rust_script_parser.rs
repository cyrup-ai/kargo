use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::models::{Dependency, DependencyLocation, DependencyParser, DependencySource};

// Regular expressions for parsing rust-script files
static CARGO_SECTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"```cargo\n([\s\S]*?)```").expect("Invalid cargo section regex"));
static CARGO_DEPS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"//\s*cargo-deps:\s*(.+)$").expect("Invalid cargo deps regex"));
static DEPS_SECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)\[dependencies\](.*?)(?:\n\s*\[|\z)").expect("Invalid deps section regex")
});
static SIMPLE_DEP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?m)^(\w+)\s*=\s*["']([^"']+)["']"#).expect("Invalid simple dep regex")
});
static TABLE_DEP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?ms)^(\w+)\s*=\s*\{(.*?)version\s*=\s*["']([^"']+)["']"#)
        .expect("Invalid table dep regex")
});
static DEPS_WITH_VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(\w+)\s*=\s*["']([^"']+)["']"#).expect("Invalid deps with version regex")
});
static CARGO_DEPS_FORMAT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(\w+)=["']([^"']+)["']"#).expect("Invalid cargo deps format regex"));
static DEBUG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"([\w-]+)=?["']?([^,"']+)["']?"#).expect("Invalid debug regex"));
static BARE_DEPS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|,)\s*(\w+)(?:\s*,|$)").expect("Invalid bare deps regex"));

/// Parser for Rust script files
#[derive(Clone)]
pub struct RustScriptParser;

impl DependencyParser for RustScriptParser {
    fn parse(&self, source: &DependencySource) -> Result<Vec<Dependency>> {
        let content = source.content();
        let mut dependencies = Vec::new();

        // Parse embedded cargo manifest sections
        self.parse_cargo_sections(&content, &mut dependencies, source)?;

        // Parse single-line cargo-deps format
        self.parse_cargo_deps_line(&content, &mut dependencies, source)?;

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
        for captures in CARGO_SECTION_REGEX.captures_iter(content) {
            if let Some(cargo_content) = captures.get(1) {
                let cargo_content_str = cargo_content.as_str();

                // Look for dependencies section
                if let Some(deps_section) = DEPS_SECTION_REGEX.captures(cargo_content_str) {
                    let deps_content = deps_section
                        .get(1)
                        .ok_or_else(|| anyhow::anyhow!("Failed to get deps content"))?
                        .as_str();

                    // Parse simple dependencies: name = "version"
                    for cap in SIMPLE_DEP_REGEX.captures_iter(deps_content) {
                        let name = cap
                            .get(1)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                            .as_str();
                        let version = cap
                            .get(2)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                            .as_str();

                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                            location: DependencyLocation::RustScriptCargo {
                                section_range: (cargo_content.start(), cargo_content.end()),
                            },
                        });
                    }

                    // Parse table-style dependencies: name = { version = "version", ... }
                    for cap in TABLE_DEP_REGEX.captures_iter(deps_content) {
                        let name = cap
                            .get(1)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                            .as_str();
                        let version = cap
                            .get(3)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                            .as_str();

                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                            location: DependencyLocation::RustScriptCargo {
                                section_range: (cargo_content.start(), cargo_content.end()),
                            },
                        });
                    }
                }
            }
        }
        Ok(())
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
                    .ok_or_else(|| anyhow::anyhow!("Failed to get match start"))?
                    .start();
                let line_end = captures
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("Failed to get match end"))?
                    .end();

                // Track which dependencies have version info
                let mut deps_with_version = Vec::new();

                // Try parsing: name = "version" format
                for cap in DEPS_WITH_VERSION_REGEX.captures_iter(deps_str) {
                    let name = cap
                        .get(1)
                        .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                        .as_str();

                    if !deps_with_version.iter().any(|d: &String| d == name) {
                        let version = cap
                            .get(2)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                            .as_str();
                        deps_with_version.push(name.to_string());

                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                            location: DependencyLocation::RustScriptDeps {
                                line_range: (line_start, line_end),
                            },
                        });
                    }
                }

                // Try parsing: name="version" format (no spaces)
                for cap in CARGO_DEPS_FORMAT_REGEX.captures_iter(deps_str) {
                    let name = cap
                        .get(1)
                        .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                        .as_str();

                    if !deps_with_version.iter().any(|d: &String| d == name) {
                        let version = cap
                            .get(2)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                            .as_str();
                        deps_with_version.push(name.to_string());

                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                            location: DependencyLocation::RustScriptDeps {
                                line_range: (line_start, line_end),
                            },
                        });
                    }
                }

                // More relaxed parsing for edge cases
                for cap in DEBUG_REGEX.captures_iter(deps_str) {
                    let name = cap
                        .get(1)
                        .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                        .as_str();

                    if !deps_with_version.iter().any(|d: &String| d == name) && cap.get(2).is_some()
                    {
                        let version = cap
                            .get(2)
                            .ok_or_else(|| anyhow::anyhow!("Failed to get dependency version"))?
                            .as_str();
                        deps_with_version.push(name.to_string());

                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                            location: DependencyLocation::RustScriptDeps {
                                line_range: (line_start, line_end),
                            },
                        });
                    }
                }

                // Parse bare dependency names (no version specified)
                for cap in BARE_DEPS_REGEX.captures_iter(deps_str) {
                    let name = cap
                        .get(1)
                        .ok_or_else(|| anyhow::anyhow!("Failed to get dependency name"))?
                        .as_str();

                    // Skip if we already found this with a version
                    if deps_with_version.iter().any(|d| d == name) {
                        continue;
                    }

                    dependencies.push(Dependency {
                        name: name.to_string(),
                        version: "*".to_string(),
                        location: DependencyLocation::RustScriptDeps {
                            line_range: (line_start, line_end),
                        },
                    });
                }
            }
        }
        Ok(())
    }
}
