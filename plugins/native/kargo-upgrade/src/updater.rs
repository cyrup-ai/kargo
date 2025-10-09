//! Module for updating dependencies to their latest versions

use semver::Version;

use crate::{
    crates_io::{get_latest_version, get_latest_version_blocking},
    models::{Dependency, DependencyUpdate, DependencyUpdater},
    types::{PendingDependencyUpdate, UpdateOptions},
};

/// Updates dependencies to their latest versions from crates.io
#[derive(Clone)]
pub struct CratesIoUpdater {
    options: UpdateOptions,
    handle: Option<tokio::runtime::Handle>,
}

impl CratesIoUpdater {
    /// Create a new updater with the given options and optional runtime handle
    pub fn new(options: UpdateOptions, handle: Option<tokio::runtime::Handle>) -> Self {
        Self { options, handle }
    }
}

impl DependencyUpdater for CratesIoUpdater {
    fn update(&self, dependency: &Dependency) -> PendingDependencyUpdate {
        // Clone what we need for the async task
        let dependency = dependency.clone();
        let options = self.options.clone();
        let handle = self.handle.clone();

        // Create a future that will be performed asynchronously
        let update_future = async move {
            // Handle dependencies with no version (like bare cargo-deps entries)
            let from_version = if dependency.version.is_empty() {
                "none".to_string()
            } else {
                dependency.version.clone()
            };

            // Determine if we allow pre-releases
            // When compatible_only is true, we want stable versions only
            let allow_prerelease = !options.compatible_only;

            // Get the latest version from crates.io using a safe execution context
            let to_version = if let Some(h) = handle {
                let name = dependency.name.clone();
                match h.spawn_blocking(move || get_latest_version_blocking(&name, allow_prerelease)).await {
                    Ok(res) => res?,
                    Err(e) => return Err(anyhow::anyhow!("JoinError in crates.io call: {}", e)),
                }
            } else {
                // Fallback: use async client (assumes caller has a reactor)
                get_latest_version(&dependency.name, allow_prerelease).await?
            };

            if let Some(to_version) = to_version {
                // Skip if already at latest version
                if !dependency.version.is_empty() && dependency.version == to_version {
                    return Ok(None);
                }

                // Parse versions for semver comparison
                if options.compatible_only && !dependency.version.is_empty() {
                    // Use semver to check compatibility
                    if let (Ok(from), Ok(to)) = (
                        Version::parse(&dependency.version),
                        Version::parse(&to_version),
                    ) {
                        // Skip major version bumps when compatible_only is true
                        if to.major > from.major {
                            return Ok(None);
                        }
                    }
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
