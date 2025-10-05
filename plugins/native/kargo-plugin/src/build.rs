use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn build_plugin(plugin_dir: &Path) -> Result<()> {
    let status = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--lib")
        .arg("--manifest-path")
        .arg(plugin_dir.join("Cargo.toml"))
        .status()?;

    if !status.success() {
        anyhow::bail!("cargo build failed for {}", plugin_dir.display());
    }

    Ok(())
}

pub fn find_lib_artifact(dir: &Path, crate_name: &str) -> Result<PathBuf> {
    // First try the local target directory
    let mut release = dir.join("target").join("release");

    // If not found, try the workspace target directory
    if !release.is_dir() {
        // Walk up to find workspace root (where Cargo.lock exists)
        let mut workspace_root = dir.to_path_buf();
        while !workspace_root.join("Cargo.lock").exists() {
            workspace_root = workspace_root
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Could not find workspace root"))?
                .to_path_buf();
        }
        release = workspace_root.join("target").join("release");
    }

    if !release.is_dir() {
        anyhow::bail!("Release directory not found at {}", release.display());
    }

    let (prefix, ext) = if cfg!(windows) {
        ("", "dll")
    } else if cfg!(target_os = "macos") {
        ("lib", "dylib")
    } else {
        ("lib", "so")
    };

    // Convert crate name to library name (replace hyphens with underscores)
    let lib_crate_name = crate_name.replace('-', "_");

    // Look for the specific library file
    let lib_name = format!("{prefix}{lib_crate_name}.{ext}");
    let lib_path = release.join(&lib_name);

    if lib_path.exists() {
        Ok(lib_path)
    } else {
        anyhow::bail!("Library artifact not found: {}", lib_path.display())
    }
}
