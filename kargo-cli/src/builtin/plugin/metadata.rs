use anyhow::Result;
use std::path::Path;
use toml_edit::DocumentMut;

pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub has_lib: bool,
}

pub fn extract_plugin_metadata(cargo_toml_path: &Path) -> Result<PluginMetadata> {
    let content = std::fs::read_to_string(cargo_toml_path)?;
    let doc = content.parse::<DocumentMut>()?;

    let package = doc["package"].as_table()
        .ok_or_else(|| anyhow::anyhow!("No [package] section in Cargo.toml"))?;

    let name = package["name"].as_str()
        .ok_or_else(|| anyhow::anyhow!("No package name in Cargo.toml"))?
        .to_string();

    let version = package["version"].as_str()
        .ok_or_else(|| anyhow::anyhow!("No package version in Cargo.toml"))?
        .to_string();

    // Check for [lib] section
    let has_lib = doc.get("lib").is_some();

    Ok(PluginMetadata { name, version, has_lib })
}
