//! Module for updating dependencies to their latest versions

use crate::{
    crates_io::get_latest_version,
    models::{Dependency, DependencyUpdate, DependencyUpdater},
    types::{PendingDependencyUpdate, UpdateOptions},
};

/// Updates dependencies to their latest versions from crates.io
#[derive(Clone)]
pub struct CratesIoUpdater {
    options: UpdateOptions,
}

impl CratesIoUpdater {
    /// Create a new updater with the given options
    pub fn new(options: UpdateOptions) -> Self {
        Self { options }
    }
}

impl DependencyUpdater for CratesIoUpdater {
    fn update(&self, dependency: &Dependency) -> PendingDependencyUpdate {
        // Clone what we need for the async task
        let dependency = dependency.clone();
        let _options = self.options.clone(); // Unused for now but may be needed later

        // Create a future that will be performed asynchronously
        let update_future = async move {
            // Handle dependencies with no version (like bare cargo-deps entries)
            let from_version = if dependency.version.is_empty() {
                "none".to_string()
            } else {
                dependency.version.clone()
            };

            // Get the latest version from crates.io
            let to_version = get_latest_version(&dependency.name).await?;

            if let Some(to_version) = to_version {
                // Skip if already at latest version
                if !dependency.version.is_empty() && dependency.version == to_version {
                    return Ok(None);
                }

                // Return the update
                Ok(Some(DependencyUpdate {
                    name: dependency.name.clone(),
                    from_version,
                    to_version,
                    dependency: dependency.clone(),
                }))
            } else {
                Ok(None)
            }
        };

        // Return a domain-specific type that will resolve to the update result
        PendingDependencyUpdate::new(update_future)
    }
}
