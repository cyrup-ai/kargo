use anyhow::Result;
use cargo_metadata::{MetadataCommand, TargetKind};
use std::path::{Path, PathBuf};

/// Get platform-specific library prefix and extension
pub fn platform_lib_format() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("", "dll")
    } else if cfg!(target_os = "macos") {
        ("lib", "dylib")
    } else {
        ("lib", "so")
    }
}

/// Find workspace root by walking up directory tree using cargo_metadata
pub fn find_workspace_root(start_dir: &Path) -> Result<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Use cargo_metadata to properly detect workspace root
            if let Ok(metadata) = MetadataCommand::new()
                .manifest_path(&cargo_toml)
                .no_deps()
                .exec()
            {
                // Check if this manifest's workspace_root matches current directory
                // This correctly identifies workspace roots vs. workspace members
                if metadata.workspace_root.as_std_path() == current {
                    return Ok(current);
                }
            }
        }

        current = current
            .parent()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not find workspace root starting from {}",
                    start_dir.display()
                )
            })?
            .to_path_buf();
    }
}

/// Find the release directory (local or workspace)
pub fn find_release_directory(plugin_dir: &Path) -> Result<PathBuf> {
    // Try local target/release first
    let local_release = plugin_dir.join("target").join("release");
    if local_release.is_dir() {
        return Ok(local_release);
    }

    // Try workspace target/release
    let workspace_root = find_workspace_root(plugin_dir)?;
    let workspace_release = workspace_root.join("target").join("release");

    if workspace_release.is_dir() {
        Ok(workspace_release)
    } else {
        anyhow::bail!(
            "Release directory not found. Tried:\n  - {}\n  - {}",
            local_release.display(),
            workspace_release.display()
        )
    }
}

/// Find the compiled library artifact for a plugin
///
/// Searches for the cdylib in:
/// 1. Local target/release directory
/// 2. Workspace root target/release directory (if in workspace)
///
/// Returns the full path to the library file (e.g., libmy_plugin.dylib)
pub fn find_lib_artifact(plugin_dir: &Path, crate_name: &str) -> Result<PathBuf> {
    let release_dir = find_release_directory(plugin_dir)?;

    let (prefix, ext) = platform_lib_format();
    let lib_crate_name = crate_name.replace('-', "_");
    let lib_name = format!("{prefix}{lib_crate_name}.{ext}");
    let lib_path = release_dir.join(&lib_name);

    if lib_path.exists() {
        Ok(lib_path)
    } else {
        anyhow::bail!(
            "Library artifact not found: {}\nSearched in: {}",
            lib_path.display(),
            release_dir.display()
        )
    }
}

/// Find existing library artifact, returning None if not found
///
/// This is like find_lib_artifact but returns Option instead of Result
pub fn find_existing_lib(plugin_dir: &Path) -> Result<Option<PathBuf>> {
    // First, get the crate name from Cargo.toml
    let cargo_toml = plugin_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(None);
    }

    let metadata = MetadataCommand::new()
        .manifest_path(&cargo_toml)
        .no_deps()
        .exec()?;

    let package = metadata
        .packages
        .iter()
        .find(|p| p.manifest_path.as_std_path() == cargo_toml);

    let Some(pkg) = package else {
        return Ok(None);
    };

    // Get the library target name
    let lib_target = pkg.targets.iter().find(|t| {
        t.kind.contains(&TargetKind::CDyLib)
    });

    let Some(target) = lib_target else {
        return Ok(None);
    };

    let crate_name = &target.name;

    // Try to find the artifact
    match find_lib_artifact(plugin_dir, crate_name) {
        Ok(path) => Ok(Some(path)),
        Err(_) => Ok(None),  // Not found, return None instead of error
    }
}
