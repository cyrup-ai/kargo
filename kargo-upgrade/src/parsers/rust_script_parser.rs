//! Parser for Rust script files with cargo dependencies

use anyhow::{anyhow, Result};
use regex::Regex;
use once_cell::sync::Lazy;

use crate::models::{Dependency, DependencyLocation, DependencyParser, DependencySource};

// Regular expressions for parsing rust-script files
static CARGO_SECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"```cargo\n([\s\S]*?)```").unwrap());
static CARGO_DEPS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"//\s*cargo-deps:\s*(.+)$").unwrap());
static DEPS_SECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)\[dependencies\](.*?)(?:\n\s*\[|\z)").unwrap());
static SIMPLE_DEP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?m)^(\w+)\s*=\s*["']([^"']+)["']"#).unwrap());
static TABLE_DEP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?ms)^(\w+)\s*=\s*\{(.*?)version\s*=\s*["']([^"']+)["']"#).unwrap());
static DEPS_WITH_VERSION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\w+)\s*=\s*["']([^"']+)["']"#).unwrap());
static CARGO_DEPS_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\w+)=["']([^"']+)["']"#).unwrap());
static DEBUG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([\w-]+)=?["']?([^,"']+)["']?"#).unwrap());
static BARE_DEPS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|,)\s*(\w+)(?:\s*,|$)").unwrap());

/// Parser for Rust script files
#[derive(Clone)]
pub struct RustScriptParser;

impl DependencyParser for RustScriptParser {
    fn parse(&self, source: &DependencySource) -> Result<Vec<Dependency>> {
        match source {
            DependencySource::RustScript { content, .. } => {
                let mut dependencies = Vec::new();
                
                // Parse ```cargo sections
                self.parse_cargo_sections(content, &mut dependencies)?;
                
                // Parse cargo-deps: lines
                self.parse_cargo_deps_lines(content, &mut dependencies)?;
                
                Ok(dependencies)
            },
            _ => Err(anyhow!("Not a Rust script source")),
        }
    }
}

impl RustScriptParser {
    /// Parse ```cargo sections and extract dependencies
    fn parse_cargo_sections(&self, content: &str, dependencies: &mut Vec<Dependency>) -> Result<()> {
        for captures in CARGO_SECTION_REGEX.captures_iter(content) {
            if let Some(cargo_section) = captures.get(1) {
                let cargo_content = cargo_section.as_str();
                let section_range = (cargo_section.start(), cargo_section.end());
                
                // Parse dependencies section
                if let Some(deps_section) = DEPS_SECTION_REGEX.captures(cargo_content) {
                    let deps_content = deps_section.get(1).unwrap().as_str();
                    
                    // Parse simple dependencies: name = "version"
                    for cap in SIMPLE_DEP_REGEX.captures_iter(deps_content) {
                        let name = cap.get(1).unwrap().as_str().to_string();
                        let version = cap.get(2).unwrap().as_str().to_string();
                        
                        dependencies.push(Dependency {
                            name,
                            version,
                            location: DependencyLocation::RustScriptCargo {
                                section_range,
                            },
                        });
                    }
                    
                    // Parse table format: name = { version = "version", ... }
                    for cap in TABLE_DEP_REGEX.captures_iter(deps_content) {
                        let name = cap.get(1).unwrap().as_str().to_string();
                        let version = cap.get(3).unwrap().as_str().to_string();
                        
                        dependencies.push(Dependency {
                            name,
                            version,
                            location: DependencyLocation::RustScriptCargo {
                                section_range,
                            },
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse cargo-deps: lines and extract dependencies
    fn parse_cargo_deps_lines(&self, content: &str, dependencies: &mut Vec<Dependency>) -> Result<()> {
        for captures in CARGO_DEPS_REGEX.captures_iter(content) {
            if let Some(deps_match) = captures.get(1) {
                let deps_str = deps_match.as_str();
                let line_range = (deps_match.start(), deps_match.end());
                
                // Handle all dependencies, trying to match with the most flexible regex
                let mut deps_with_version = Vec::new();
                
                // First try the standard dependencies format: name="version"
                for cap in DEPS_WITH_VERSION_REGEX.captures_iter(deps_str) {
                    let name = cap.get(1).unwrap().as_str().to_string();
                    
                    // Skip if we already added this dependency
                    if deps_with_version.contains(&name) {
                        continue;
                    }
                    
                    let version = cap.get(2).unwrap().as_str().to_string();
                    deps_with_version.push(name.clone());
                    
                    dependencies.push(Dependency {
                        name,
                        version,
                        location: DependencyLocation::RustScriptDeps {
                            line_range,
                        },
                    });
                }
                
                // Then try the cargo-deps format: name=version
                for cap in CARGO_DEPS_FORMAT_REGEX.captures_iter(deps_str) {
                    let name = cap.get(1).unwrap().as_str().to_string();
                    
                    // Skip if we already added this dependency
                    if deps_with_version.contains(&name) {
                        continue;
                    }
                    
                    let version = cap.get(2).unwrap().as_str().to_string();
                    deps_with_version.push(name.clone());
                    
                    dependencies.push(Dependency {
                        name,
                        version,
                        location: DependencyLocation::RustScriptDeps {
                            line_range,
                        },
                    });
                }
                
                // Finally, use the more flexible debug regex for any remaining formats
                for cap in DEBUG_REGEX.captures_iter(deps_str) {
                    let name = cap.get(1).unwrap().as_str().to_string();
                    
                    // Skip if we already added this dependency
                    if deps_with_version.contains(&name) {
                        continue;
                    }
                    
                    let version = cap.get(2).unwrap().as_str().to_string();
                    deps_with_version.push(name.clone());
                    
                    dependencies.push(Dependency {
                        name,
                        version,
                        location: DependencyLocation::RustScriptDeps {
                            line_range,
                        },
                    });
                }
                
                // Then handle bare dependencies (without version)
                for cap in BARE_DEPS_REGEX.captures_iter(deps_str) {
                    let name = cap.get(1).unwrap().as_str().to_string();
                    
                    // Skip if this dependency already has a version
                    if deps_with_version.contains(&name) {
                        continue;
                    }
                    
                    // For bare dependencies, we use an empty version string
                    // The updater will assign the latest version
                    dependencies.push(Dependency {
                        name,
                        version: String::new(),
                        location: DependencyLocation::RustScriptDeps {
                            line_range,
                        },
                    });
                }
            }
        }
        
        Ok(())
    }
}