use crate::error::Error;
use log::debug;
use regex::Regex;
use std::fmt;

/// Represents a parsed package specification
#[derive(Debug, Clone)]
pub struct PackageSpec {
    /// Name of the package
    pub name: String,
    /// Optional version constraint
    pub version: Option<String>,
}

impl PackageSpec {
    /// Parse a package specification string in the format "name[@version]"
    pub fn parse(spec: &str) -> Result<Self, Error> {
        let parts: Vec<&str> = spec.split('@').collect();

        match parts.len() {
            1 => {
                // Just a package name
                let name = parts[0].trim();
                if Self::is_valid_package_name(name) {
                    debug!("Parsed package name: {}", name);
                    Ok(Self {
                        name: name.to_string(),
                        version: None,
                    })
                } else {
                    Err(Error::InvalidPackageName(name.to_string()))
                }
            }
            2 => {
                // Package name with version
                let name = parts[0].trim();
                let version = parts[1].trim();

                if Self::is_valid_package_name(name) {
                    debug!("Parsed package name: {} with version: {}", name, version);
                    Ok(Self {
                        name: name.to_string(),
                        version: Some(version.to_string()),
                    })
                } else {
                    Err(Error::InvalidPackageName(name.to_string()))
                }
            }
            _ => Err(Error::PackageSpecParse(format!(
                "Invalid package specification: {}",
                spec
            ))),
        }
    }

    /// Check if a package name is valid according to Cargo rules
    fn is_valid_package_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Check using a regex for valid crate names
        // Rust crate names can contain alphanumeric characters, - and _
        // They cannot start with a digit, - or _
        lazy_static::lazy_static! {
            static ref CRATE_NAME_RE: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$")
                .expect("Invalid regex for crate name validation");
        }
        CRATE_NAME_RE.is_match(name)
    }

    /// Get the package version as a dependency specification string
    pub fn version_spec(&self) -> String {
        match &self.version {
            Some(version) => format!("\"{}\"", version),
            None => "\"*\"".to_string(),
        }
    }

    /// Get the output filename for the JSON documentation
    pub fn json_filename(&self) -> String {
        match &self.version {
            Some(version) => format!("{}-{}.json", self.name, version),
            None => format!("{}.json", self.name),
        }
    }

    /// Get the output filename for the Markdown documentation
    pub fn markdown_filename(&self) -> String {
        match &self.version {
            Some(version) => format!("{}-{}.md", self.name, version),
            None => format!("{}.md", self.name),
        }
    }

    /// Get a display name for the package (useful for status messages)
    pub fn display_name(&self) -> String {
        match &self.version {
            Some(version) => format!("{}@{}", self.name, version),
            None => self.name.clone(),
        }
    }
}

impl fmt::Display for PackageSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.version {
            Some(version) => write!(f, "{}@{}", self.name, version),
            None => write!(f, "{}", self.name),
        }
    }
}
