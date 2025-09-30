//! Module for updating dependencies to their latest versions

use semver::Version;

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
        let options = self.options.clone();

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

            // Get the latest version from crates.io
            let to_version = get_latest_version(&dependency.name, allow_prerelease).await?;

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
