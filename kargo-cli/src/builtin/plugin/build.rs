use anyhow::{Context, Result};
use std::path::Path;

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
