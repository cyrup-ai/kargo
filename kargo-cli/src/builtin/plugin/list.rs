use anyhow::Result;
use std::path::Path;
use super::{git, metadata};

pub async fn list_plugins(remote: Option<&String>) -> Result<()> {
    if let Some(remote_url) = remote {
        list_remote_plugins(remote_url).await
    } else {
        list_installed_plugins()
    }
}

fn list_installed_plugins() -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let plugins_dir = config_dir.join("kargo").join("plugins");

    if !plugins_dir.exists() {
        println!("No plugins installed");
        return Ok(());
    }

    println!("Installed plugins:");

    // Walk the directory structure: {org}/{repo}/{branch}/{name}/{version}
    for org_entry in std::fs::read_dir(&plugins_dir)? {
        let org_entry = org_entry?;
        let org_name = org_entry.file_name();

        for repo_entry in std::fs::read_dir(org_entry.path())? {
            let repo_entry = repo_entry?;
            let repo_name = repo_entry.file_name();

            for branch_entry in std::fs::read_dir(repo_entry.path())? {
                let branch_entry = branch_entry?;
                let branch_name = branch_entry.file_name();

                for name_entry in std::fs::read_dir(branch_entry.path())? {
                    let name_entry = name_entry?;
                    let plugin_name = name_entry.file_name();

                    for version_entry in std::fs::read_dir(name_entry.path())? {
                        let version_entry = version_entry?;
                        let version = version_entry.file_name();

                        println!(
                            "  {}/{} ({}) - {} v{}",
                            org_name.to_string_lossy(),
                            repo_name.to_string_lossy(),
                            branch_name.to_string_lossy(),
                            plugin_name.to_string_lossy(),
                            version.to_string_lossy()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

async fn list_remote_plugins(remote_url: &str) -> Result<()> {
    // Clone to temporary directory
    let temp = tempfile::tempdir()?;
    git::clone_repository(remote_url, temp.path()).await?;

    println!("Plugins available in {remote_url}:");

    // Scan for Cargo.toml files with [lib] sections
    scan_for_plugins(temp.path())?;

    Ok(())
}

fn scan_for_plugins(dir: &Path) -> Result<()> {
    const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "dist", "build"];

    for entry in jwalk::WalkDir::new(dir)
        .skip_hidden(true)
        .into_iter()
    {
        let entry = entry?;

        let file_type = entry.file_type();
        if file_type.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if SKIP_DIRS.contains(&name) {
                    continue;
                }
            }
        }

        if entry.file_name() == "Cargo.toml"
            && let Ok(metadata) = metadata::extract_plugin_metadata(&entry.path())
            && metadata.has_lib
        {
            println!("  {} v{}", metadata.name, metadata.version);
        }
    }

    Ok(())
}
