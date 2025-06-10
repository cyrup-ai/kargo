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
        let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap();
        re.is_match(name)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_name_only() {
        let spec = PackageSpec::parse("tokio").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, None);
    }

    #[test]
    fn test_parse_package_with_version() {
        let spec = PackageSpec::parse("tokio@1.28.0").unwrap();
        assert_eq!(spec.name, "tokio");
        assert_eq!(spec.version, Some("1.28.0".to_string()));
    }

    #[test]
    fn test_invalid_package_name() {
        assert!(PackageSpec::parse("").is_err());
        assert!(PackageSpec::parse("1invalid").is_err());
        assert!(PackageSpec::parse("invalid@1.0@extra").is_err());
    }

    #[test]
    fn test_version_spec() {
        let spec1 = PackageSpec::parse("tokio").unwrap();
        assert_eq!(spec1.version_spec(), "\"*\"");

        let spec2 = PackageSpec::parse("tokio@1.28.0").unwrap();
        assert_eq!(spec2.version_spec(), "\"1.28.0\"");
    }

    #[test]
    fn test_output_filenames() {
        let spec1 = PackageSpec::parse("tokio").unwrap();
        assert_eq!(spec1.json_filename(), "tokio.json");
        assert_eq!(spec1.markdown_filename(), "tokio.md");

        let spec2 = PackageSpec::parse("tokio@1.28.0").unwrap();
        assert_eq!(spec2.json_filename(), "tokio-1.28.0.json");
        assert_eq!(spec2.markdown_filename(), "tokio-1.28.0.md");
    }
}