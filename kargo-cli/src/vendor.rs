use crate::events::{Event, EventBus};
use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, Package};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;

/// Get the cargo home directory path
///
/// Checks $CARGO_HOME environment variable, falling back to ~/.cargo if not set.
/// Returns a String path to the cargo home directory.
fn get_cargo_home() -> String {
    std::env::var("CARGO_HOME").unwrap_or_else(|_| {
        dirs::home_dir()
            .map(|p| p.join(".cargo").to_string_lossy().to_string())
            .unwrap_or_else(|| ".cargo".to_string())
    })
}

/// Find the extracted source directory for a package in cargo's registry cache
async fn find_package_source(pkg: &Package) -> Result<PathBuf> {
    let cargo_home = get_cargo_home();

    let registry_src = PathBuf::from(cargo_home).join("registry").join("src");

    // Scan for the package in any registry index directory
    let mut entries = tokio::fs::read_dir(&registry_src)
        .await
        .with_context(|| format!("Failed to read registry src directory: {:?}", registry_src))?;

    while let Some(entry) = entries.next_entry().await? {
        let index_dir = entry.path();

        // Use tokio::fs::metadata for async metadata check
        let metadata = tokio::fs::metadata(&index_dir).await;
        if metadata.is_err() || !metadata?.is_dir() {
            continue;
        }

        let pkg_dir = index_dir.join(format!("{}-{}", pkg.name, pkg.version));

        // Use tokio::fs::try_exists for async existence check
        if tokio::fs::try_exists(&pkg_dir).await.unwrap_or(false) {
            return Ok(pkg_dir);
        }
    }

    anyhow::bail!(
        "Package source not found: {}-{}\nMake sure the package is downloaded (run `cargo fetch` first)",
        pkg.name,
        pkg.version
    );
}

/// Compute SHA256 hash of a file and return as lowercase hex string
async fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("Failed to open file for hashing: {:?}", path))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let n = file
            .read(&mut buffer)
            .await
            .with_context(|| format!("Failed to read file for hashing: {:?}", path))?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Find the .crate file (tarball) in cargo's cache for checksum computation
async fn find_crate_file(pkg: &Package) -> Result<PathBuf> {
    let cargo_home = get_cargo_home();

    let registry_cache = PathBuf::from(cargo_home).join("registry").join("cache");

    // Scan for the .crate file in any registry index directory
    let mut entries = tokio::fs::read_dir(&registry_cache)
        .await
        .with_context(|| {
            format!(
                "Failed to read registry cache directory: {:?}",
                registry_cache
            )
        })?;

    while let Some(entry) = entries.next_entry().await? {
        let index_dir = entry.path();

        // Use tokio::fs::metadata for async metadata check
        let metadata = tokio::fs::metadata(&index_dir).await;
        if metadata.is_err() || !metadata?.is_dir() {
            continue;
        }

        let crate_file = index_dir.join(format!("{}-{}.crate", pkg.name, pkg.version));

        // Use tokio::fs::try_exists for async existence check
        if tokio::fs::try_exists(&crate_file).await.unwrap_or(false) {
            return Ok(crate_file);
        }
    }

    anyhow::bail!(
        "Crate file not found: {}-{}.crate\nMake sure the package is downloaded (run `cargo fetch` first)",
        pkg.name,
        pkg.version
    );
}

/// Recursively copy a directory using tokio::fs
async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    tokio::fs::create_dir_all(dst)
        .await
        .with_context(|| format!("Failed to create directory: {:?}", dst))?;

    let mut entries = tokio::fs::read_dir(src)
        .await
        .with_context(|| format!("Failed to read directory: {:?}", src))?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await.with_context(|| {
                format!(
                    "Failed to copy file from {:?} to {:?}",
                    src_path, dst_path
                )
            })?;
        }
    }

    Ok(())
}

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
        tokio::fs::create_dir_all(&self.vendor_path).await?;

        for pkg in deps.values() {
            if pkg.source.as_ref().is_some_and(|s| s.repr.starts_with("registry+")) {
                self.vendor_package(pkg).await?;
            }
        }

        self.events.publish(Event::VendorFinished {
            path: workspace_path.to_owned(),
        });

        Ok(())
    }

    async fn vendor_package(&self, pkg: &Package) -> Result<()> {
        // 1. Find the source directory in cargo's registry cache
        let source_dir = find_package_source(pkg).await?;

        // 2. Determine vendor destination (correct format: {name}-{version})
        let pkg_name = format!("{}-{}", pkg.name, pkg.version);
        let dest_dir = self.vendor_path.join(&pkg_name);

        // 3. Copy source files to vendor directory
        copy_dir_recursive(&source_dir, &dest_dir)
            .await
            .with_context(|| format!("Failed to vendor package: {}", pkg_name))?;

        // 4. Find the .crate file for checksum computation
        let crate_file = find_crate_file(pkg).await?;

        // 5. Compute SHA256 of .crate file
        let checksum = compute_sha256(&crate_file)
            .await
            .with_context(|| format!("Failed to compute checksum for: {}", pkg_name))?;

        // 6. Generate .cargo-checksum.json
        let checksum_data = serde_json::json!({
            "files": {},
            "package": checksum
        });

        let checksum_path = dest_dir.join(".cargo-checksum.json");
        tokio::fs::write(
            &checksum_path,
            serde_json::to_string_pretty(&checksum_data)?,
        )
        .await
        .with_context(|| format!("Failed to write checksum file: {:?}", checksum_path))?;

        log::info!("Vendored: {} -> {:?}", pkg_name, dest_dir);

        Ok(())
    }
}
