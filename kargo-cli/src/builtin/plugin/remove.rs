use anyhow::Result;
use super::parser::{self, SourceType};

pub fn remove_plugin(source: &str) -> Result<()> {
    let source_type = parser::parse_source(source)?;

    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    // Handle different removal scenarios based on specificity
    match source_type {
        // CASE 1: Specific plugin removal across all branches
        // Example: cyrup-ai/kargo/plugin-a
        SourceType::GitHub { org, repo, plugin: Some(plugin_name) } => {
            let base_path = config_dir
                .join("kargo")
                .join("plugins")
                .join(&org)
                .join(&repo);

            if !base_path.exists() {
                anyhow::bail!(
                    "Repository '{}/{}' not found.\nUse 'kargo plugin list' to see installed plugins.",
                    org, repo
                );
            }

            // Remove plugin from all branches
            let mut removed_from_branches = Vec::new();
            
            for branch_entry in std::fs::read_dir(&base_path)? {
                let branch_entry = branch_entry?;
                let branch_name = branch_entry.file_name();
                let plugin_path = branch_entry.path().join(&plugin_name);
                
                if plugin_path.exists() {
                    std::fs::remove_dir_all(&plugin_path)?;
                    removed_from_branches.push(branch_name.to_string_lossy().to_string());
                }
            }

            if removed_from_branches.is_empty() {
                anyhow::bail!(
                    "Plugin '{}/{}/{}' not found in any branch.\nUse 'kargo plugin list' to see installed plugins.",
                    org, repo, plugin_name
                );
            }

            println!(
                "✓ Removed plugin: {}/{}/{} from {} branch(es): {}",
                org, repo, plugin_name,
                removed_from_branches.len(),
                removed_from_branches.join(", ")
            );
        }

        // CASE 2: Local plugin removal  
        // Example: local/my-plugin
        SourceType::GitHub { org, repo, plugin: None } if org == "local" => {
            let local_path = config_dir
                .join("kargo")
                .join("plugins")
                .join("local")
                .join(&repo);

            if !local_path.exists() {
                anyhow::bail!(
                    "Local plugin '{}' not found.\nUse 'kargo plugin list' to see installed plugins.",
                    repo
                );
            }

            std::fs::remove_dir_all(&local_path)?;
            println!("✓ Removed local plugin: {} (all versions)", repo);
        }

        // CASE 3: Entire repository removal (all plugins, all branches)
        // Example: cyrup-ai/kargo
        SourceType::GitHub { org, repo, plugin: None } => {
            let repo_path = config_dir
                .join("kargo")
                .join("plugins")
                .join(&org)
                .join(&repo);

            if !repo_path.exists() {
                anyhow::bail!(
                    "Repository '{}/{}' not found.\nUse 'kargo plugin list' to see installed plugins.",
                    org, repo
                );
            }

            // Show what will be removed
            println!("Removing: all plugins from {}/{} (all branches, all versions)", org, repo);
            std::fs::remove_dir_all(&repo_path)?;
            println!("✓ Removed: {}/{}", org, repo);
        }

        // CASE 4: Local path not supported (use local/name instead)
        SourceType::LocalPath(_) => {
            anyhow::bail!(
                "Cannot remove by local path.\n\
                 Use: kargo plugin remove local/<plugin-name>\n\
                 Example: kargo plugin remove local/my-plugin"
            );
        }
    }

    Ok(())
}
