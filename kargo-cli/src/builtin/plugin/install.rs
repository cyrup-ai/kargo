use anyhow::Result;
use super::{build, git, metadata, parser};
use parser::SourceType;

pub async fn install_plugin(source: &str, branch: Option<&String>) -> Result<()> {
    let branch = branch.map_or("main", std::string::String::as_str);

    // 1. Parse source (URL vs org/repo vs local path)
    let source_type = parser::parse_source(source)?;

    // 2. Determine plugin directory
    let temp_dir = match &source_type {
        SourceType::GitHub { org, repo } => {
            // Clone to temporary directory
            let temp = tempfile::tempdir()?;
            let url = format!("https://github.com/{org}/{repo}");
            git::clone_repository(&url, temp.path()).await?;
            temp.keep()
        }
        SourceType::LocalPath(path) => {
            path.clone()
        }
    };

    // 3. Extract metadata from Cargo.toml
    let metadata = metadata::extract_plugin_metadata(&temp_dir.join("Cargo.toml"))?;

    // 4. Build the plugin
    build::build_plugin(&temp_dir)?;

    // 5. Create XDG directory structure
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let plugin_install_dir = match source_type {
        SourceType::GitHub { org, repo } => {
            config_dir
                .join("kargo")
                .join("plugins")
                .join(org)
                .join(repo)
                .join(branch)
                .join(&metadata.name)
                .join(&metadata.version)
        }
        SourceType::LocalPath(_) => {
            config_dir
                .join("kargo")
                .join("plugins")
                .join("local")
                .join(&metadata.name)
                .join(&metadata.version)
        }
    };

    std::fs::create_dir_all(&plugin_install_dir)?;

    // 6. Copy compiled library to install directory
    let lib_artifact = build::find_lib_artifact(&temp_dir, &metadata.name)?;
    let lib_filename = lib_artifact.file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid library filename"))?;
    let target_lib = plugin_install_dir.join(lib_filename);
    std::fs::copy(&lib_artifact, &target_lib)?;

    println!("✓ Installed plugin: {} v{}", metadata.name, metadata.version);
    println!("  Location: {}", plugin_install_dir.display());

    Ok(())
}
