use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, TargetKind};
use std::path::Path;

pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub has_lib: bool,
}

pub fn extract_plugin_metadata(cargo_toml_path: &Path) -> Result<PluginMetadata> {
    let metadata = MetadataCommand::new()
        .manifest_path(cargo_toml_path)
        .no_deps()
        .exec()
        .context("Failed to execute cargo metadata")?;

    let package = metadata
        .packages
        .iter()
        .find(|p| p.manifest_path.as_std_path() == cargo_toml_path)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Package not found in metadata for manifest: {}",
                cargo_toml_path.display()
            )
        })?;

    let has_cdylib = package
        .targets
        .iter()
        .any(|t| t.kind.contains(&TargetKind::CDyLib));

    Ok(PluginMetadata {
        name: package.name.to_string(),
        version: package.version.to_string(),
        has_lib: has_cdylib,
    })
}
