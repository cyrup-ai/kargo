use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Directories to scan (overridden by KRATER_SCAN)
    pub scan_dirs: Vec<PathBuf>,
    /// Commands to run after dependency consolidation
    pub post_commands: Vec<String>,
    /// Whether to enable rollback on failure
    pub rollback_on_failure: bool,
    /// Whether to vendor dependencies
    pub vendor: VendorConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VendorConfig {
    /// Enable vendoring
    pub enabled: bool,
    /// Path to store vendored sources
    pub path: PathBuf,
    /// Deduplicate versions
    pub dedupe: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_dirs: vec![std::env::var("HOME").map(PathBuf::from).unwrap_or_default()],
            post_commands: vec!["cargo fmt".to_string()],
            rollback_on_failure: true,
            vendor: VendorConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_paths = vec![
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".krater.yaml"))
                .ok(),
            ProjectDirs::from("rs", "", "krater").map(|p| p.config_dir().join("config.yaml")),
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config/krater.yaml"))
                .ok(),
        ];

        for path in config_paths.into_iter().flatten() {
            if path.exists() {
                return Ok(serde_yaml::from_str(&std::fs::read_to_string(path)?)?);
            }
        }

        Ok(Self::default())
    }
}
