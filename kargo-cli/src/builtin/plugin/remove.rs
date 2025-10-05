use anyhow::Result;
use super::parser::{self, SourceType};

pub fn remove_plugin(source: &str) -> Result<()> {
    let source_type = parser::parse_source(source)?;

    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let search_path = match source_type {
        SourceType::GitHub { org, repo } => {
            config_dir
                .join("kargo")
                .join("plugins")
                .join(org)
                .join(repo)
        }
        SourceType::LocalPath(_) => {
            anyhow::bail!("Cannot remove by local path - use plugin name instead");
        }
    };

    if !search_path.exists() {
        anyhow::bail!("Plugin not found: {source}");
    }

    // Remove entire org/repo directory (all branches/versions)
    std::fs::remove_dir_all(&search_path)?;

    println!("✓ Removed plugin: {source}");

    Ok(())
}
