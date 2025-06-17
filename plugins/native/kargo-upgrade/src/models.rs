//! Domain models for the dependency up2date

use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::types::PendingWrite;

/// Represents a parsed dependency with its metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    /// The name of the dependency
    pub name: String,
    /// The current version string
    pub version: String,
    /// The location of this dependency in the source
    pub location: DependencyLocation,
}

/// Specifies where a dependency is located within a source
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyLocation {
    /// In a Cargo.toml [dependencies] section
    CargoTomlDirect,
    /// In a Cargo.toml [dev-dependencies] section
    CargoTomlDev,
    /// In a Cargo.toml [build-dependencies] section
    CargoTomlBuild,
    /// In a rust-script ```cargo section
    RustScriptCargo {
        /// The section range in the file content
        section_range: (usize, usize),
    },
    /// In a rust-script // cargo-deps: line
    RustScriptDeps {
        /// The line range in the file content
        line_range: (usize, usize),
    },
}

/// Represents a source that can contain dependencies
#[derive(Debug, Clone)]
pub enum DependencySource {
    /// A standard Cargo.toml file
    CargoToml {
        /// Path to the Cargo.toml file
        path: PathBuf,
        /// Content of the file
        content: String,
        /// Whether this is a workspace Cargo.toml
        is_workspace: bool,
    },
    /// A Rust script with cargo dependency section
    RustScript {
        /// Path to the rust script file
        path: PathBuf,
        /// Content of the file
        content: String,
    },
}

impl DependencySource {
    /// Create a dependency source from a file path
    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path).await?;

        // Check if it's a Cargo.toml file
        if path.file_name().map_or(false, |name| name == "Cargo.toml") {
            // Check if it's a workspace Cargo.toml
            let is_workspace = content.contains("[workspace]");

            Ok(DependencySource::CargoToml {
                path,
                content,
                is_workspace,
            })
        } else {
            // Assume it's a Rust script
            Ok(DependencySource::RustScript { path, content })
        }
    }

    /// Get the path for this dependency source
    pub fn path(&self) -> &Path {
        match self {
            DependencySource::CargoToml { path, .. } => path,
            DependencySource::RustScript { path, .. } => path,
        }
    }

    /// Check if this is a workspace Cargo.toml
    pub fn is_workspace(&self) -> bool {
        match self {
            DependencySource::CargoToml { is_workspace, .. } => *is_workspace,
            _ => false,
        }
    }

    /// Get the content of this source
    pub fn content(&self) -> &str {
        match self {
            DependencySource::CargoToml { content, .. } => content,
            DependencySource::RustScript { content, .. } => content,
        }
    }

    /// Update the content of this source
    pub fn update_content(&mut self, new_content: String) {
        match self {
            DependencySource::CargoToml { content, .. } => *content = new_content,
            DependencySource::RustScript { content, .. } => *content = new_content,
        }
    }
}

/// Represents an update to a dependency
#[derive(Debug, Clone)]
pub struct DependencyUpdate {
    /// The name of the dependency
    pub name: String,
    /// The original version
    pub from_version: String,
    /// The updated version
    pub to_version: String,
    /// The original dependency
    pub dependency: Dependency,
}

/// Parser trait for extracting dependencies from different sources
pub trait DependencyParser {
    /// Parse dependencies from a source
    fn parse(&self, source: &DependencySource) -> Result<Vec<Dependency>>;
}

use crate::types::{BatchUpdateOperation, PendingDependencyUpdate};

/// Updater trait for updating dependencies to their latest versions
pub trait DependencyUpdater: Clone + Send + Sync + 'static {
    /// Update a dependency to its latest version
    /// Returns a PendingDependencyUpdate that can be awaited
    fn update(&self, dependency: &Dependency) -> PendingDependencyUpdate;

    /// Update a list of dependencies
    /// Returns a BatchUpdateOperation that provides a stream of updates
    fn update_all(&self, dependencies: &[Dependency]) -> BatchUpdateOperation {
        // Create a batch operation that will stream updates
        BatchUpdateOperation::new(dependencies.to_vec(), self)
    }
}

/// Writer trait for writing updates back to the source
pub trait DependencyWriter {
    /// Apply updates to a source
    fn apply_updates(
        &self,
        source: &mut DependencySource,
        updates: &[DependencyUpdate],
    ) -> Result<()>;

    /// Write the updated source back to disk
    fn write(&self, source: &DependencySource) -> Result<PendingWrite>;
}
