use crate::events::{Event, EventBus};
use anyhow::Result;
use cargo_metadata::{MetadataCommand, Package};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct VendorManager {
    vendor_path: PathBuf,
    dedupe: bool,
    events: EventBus,
}

impl VendorManager {
    pub fn new(vendor_path: PathBuf, dedupe: bool, events: EventBus) -> Self {
        Self {
            vendor_path,
            dedupe,
            events,
        }
    }

    pub async fn vendor_dependencies(&self, workspace_path: &Path) -> Result<()> {
        self.events.publish(Event::VendorStarted {
            path: workspace_path.to_owned(),
        });

        // Get metadata for the workspace
        let metadata = MetadataCommand::new()
            .manifest_path(workspace_path.join("Cargo.toml"))
            .exec()?;

        // Collect all unique dependencies
        let mut deps = HashMap::new();
        for pkg in metadata.packages {
            if self.dedupe {
                // Only keep latest version of each package
                deps.entry(pkg.name.as_str().to_string())
                    .and_modify(|e: &mut Package| {
                        if pkg.version > e.version {
                            *e = pkg.clone();
                        }
                    })
                    .or_insert_with(|| pkg.clone());
            } else {
                deps.insert(pkg.id.repr.clone(), pkg);
            }
        }

        // Vendor the dependencies
        std::fs::create_dir_all(&self.vendor_path)?;

        for pkg in deps.values() {
            if let Some(source) = &pkg.source {
                if source.repr.starts_with("registry+") {
                    self.vendor_package(pkg).await?;
                }
            }
        }

        self.events.publish(Event::VendorFinished {
            path: workspace_path.to_owned(),
        });

        Ok(())
    }

    async fn vendor_package(&self, pkg: &Package) -> Result<()> {
        // TODO: Implement actual vendoring using cargo-vendor internals
        // For now, just create placeholder
        let pkg_path = self
            .vendor_path
            .join(pkg.name.as_str())
            .join(&pkg.version.to_string());
        std::fs::create_dir_all(&pkg_path)?;

        Ok(())
    }
}
