use jwalk::WalkDir;
use log::{info, warn};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::sync::broadcast;
use toml_edit::{DocumentMut, Item};

use crate::backup::BackupManager;
use crate::commands::CommandRunner;
use crate::config::Config;
use crate::events::{Event, EventBus};
use crate::vendor::VendorManager;

mod backup;
pub mod builtin;
pub mod cli;
mod commands;
pub mod config;
pub mod events;
pub mod plugins;
pub mod project;
pub mod rustscript;
pub mod vendor;

// Export types for convenience
pub use project::{ProjectAnalyzer, ProjectType};
pub use rustscript::RustScript;
// Re-export kargo-upgrade types for use throughout the CLI
pub use kargo_upgrade::types::{
    CrateType, DependencyUpdate, SendFuture, UpdateCollector, UpdateResult, UpdateSession,
    UpdateWatcher, VersionUpdater, VersionUpdaterOptions,
};

// Domain-specific type for representing an update job
pub struct DependencyUpdateJob<'a> {
    up2date: &'a DependencyUpdater,
    backup: Option<BackupManager>,
    events: EventBus,
}

impl DependencyUpdateJob<'_> {
    // Get a future that will execute the update job
    pub async fn execute(mut self) -> anyhow::Result<()> {
        let backup = &mut self.backup;
        let result = self.up2date.run_impl(backup).await;

        if let Err(e) = &result
            && let Some(backup) = backup
        {
            self.events.publish(Event::Error {
                message: e.to_string(),
            });
            backup.rollback()?;
        }

        result
    }
}

pub struct DependencyUpdater {
    config: Config,
    events: EventBus,
    scan_dirs: Vec<PathBuf>,
}

impl Default for DependencyUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyUpdater {
    #[must_use] 
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        let events = EventBus::new();

        let scan_dirs = std::env::var("KRATER_SCAN")
            .map(|dirs| dirs.split(':').map(PathBuf::from).collect())
            .unwrap_or_else(|_| {
                vec![
                    std::env::var("HOME").map_or_else(|_| PathBuf::from("."), PathBuf::from),
                ]
            });

        info!("Scanning directories: {scan_dirs:?}");

        Self {
            config,
            events,
            scan_dirs,
        }
    }

    #[must_use] 
    pub fn find_cargo_tomls(&self) -> Vec<PathBuf> {
        self.scan_dirs
            .par_iter()
            .flat_map(|dir| {
                WalkDir::new(dir)
                    .follow_links(true)
                    .parallelism(jwalk::Parallelism::RayonNewPool(0)) // Use available cores
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                    .filter(|e| e.file_name.to_string_lossy() == "Cargo.toml")
                    .map(|e| e.path())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    #[must_use] 
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    // Non-async interface that returns a domain-specific type
    #[must_use] 
    pub fn run(&self) -> DependencyUpdateJob<'_> {
        let backup = if self.config.rollback_on_failure {
            BackupManager::new(self.events.clone()).ok()
        } else {
            None
        };

        DependencyUpdateJob {
            up2date: self,
            backup,
            events: self.events.clone(),
        }
    }

    // Internal implementation moved to a separate type
    async fn run_impl<'a>(
        &'a self,
        backup: &'a mut Option<BackupManager>,
    ) -> anyhow::Result<()> {
        let cargo_tomls = self.find_cargo_tomls();
        info!("Found {} Cargo.toml files", cargo_tomls.len());

        if let Some(backup) = backup {
            for file_path in &cargo_tomls {
                backup.backup_file(file_path)?;
            }
        }

        if self.config.vendor.enabled {
            let vendor = VendorManager::new(
                self.config.vendor.path.clone(),
                self.config.vendor.dedupe,
                self.events.clone(),
            );

            let workspaces = vec![PathBuf::from("workspace/path")]; // Example paths
            for workspace in workspaces {
                vendor.vendor_dependencies(&workspace).await?;
            }
        }

        // Run post-commands
        if !self.config.post_commands.is_empty() {
            let runner = CommandRunner::new(self.events.clone());
            for dir in &self.scan_dirs {
                if let Err(e) = runner.run_commands(&self.config.post_commands, dir).await {
                    warn!("Post-command failed in {}: {}", dir.display(), e);
                }
            }
        }

        Ok(())
    }

    pub fn update_crate_deps(
        &self,
        crate_path: &Path,
        workspace_deps: &DocumentMut,
    ) -> anyhow::Result<()> {
        let content = fs::read_to_string(crate_path)?;
        let mut doc = content.parse::<DocumentMut>()?;

        if let Some(deps) = doc.get_mut("dependencies").and_then(|d| d.as_table_mut()) {
            // Collect all keys first
            let keys: Vec<String> = deps.iter().map(|(k, _)| k.to_string()).collect();

            // Then process each key
            for name in keys {
                if workspace_deps
                    .get("workspace.dependencies")
                    .and_then(|d| d.get(&name))
                    .is_some()
                {
                    info!(
                        "Updating {} in {} to use workspace version",
                        name,
                        crate_path.display()
                    );
                    deps[&name] = Item::from_str("{ workspace = true }")?;
                }
            }
        }

        fs::write(crate_path, doc.to_string())?;
        Ok(())
    }
}
