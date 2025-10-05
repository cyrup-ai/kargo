use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub async fn build_plugin(plugin_dir: &Path) -> Result<()> {
    let manifest_path = plugin_dir.join("Cargo.toml");

    let status = tokio::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--lib")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .status()
        .await
        .context("Failed to spawn cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed for {}", plugin_dir.display());
    }

    Ok(())
}

pub fn find_lib_artifact(dir: &Path, crate_name: &str) -> Result<PathBuf> {
    let mut release = dir.join("target").join("release");

    if !release.is_dir() {
        let mut current = dir.to_path_buf();

        loop {
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists() {
                let content = std::fs::read_to_string(&cargo_toml)
                    .context("Failed to read Cargo.toml while searching for workspace root")?;

                if content.contains("[workspace]") {
                    release = current.join("target").join("release");
                    break;
                }
            }

            current = current
                .parent()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not find workspace root starting from {}",
                        dir.display()
                    )
                })?
                .to_path_buf();
        }
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

    let lib_crate_name = crate_name.replace('-', "_");
    let lib_name = format!("{prefix}{lib_crate_name}.{ext}");
    let lib_path = release.join(&lib_name);

    if lib_path.exists() {
        Ok(lib_path)
    } else {
        anyhow::bail!("Library artifact not found: {}", lib_path.display())
    }
}
