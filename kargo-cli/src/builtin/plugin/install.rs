use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, TargetKind};
use std::path::PathBuf;
use super::{artifact, build, git, metadata, parser};
use parser::SourceType;

pub async fn install_plugin(source: &str, branch: Option<&String>) -> Result<()> {
    let branch_name = branch.map_or("main", std::string::String::as_str);

    let source_type = parser::parse_source(source)?;

    enum RepoSource {
        Temporary(tempfile::TempDir),
        Local(PathBuf),
    }

    let repo_source = match &source_type {
        SourceType::GitHub { org, repo, plugin: _ } => {
            let temp = tempfile::tempdir().context("Failed to create temporary directory")?;
            let url = format!("https://github.com/{org}/{repo}");
            git::clone_repository(&url, temp.path(), branch.map(|s| s.as_str())).await?;
            RepoSource::Temporary(temp)
        }
        SourceType::LocalPath(path) => RepoSource::Local(path.clone()),
    };

    let repo_dir = match &repo_source {
        RepoSource::Temporary(temp) => temp.path(),
        RepoSource::Local(path) => path.as_path(),
    };

    let root_manifest = repo_dir.join("Cargo.toml");
    if !root_manifest.exists() {
        anyhow::bail!("No Cargo.toml found at {}", repo_dir.display());
    }

    let cargo_meta = MetadataCommand::new()
        .manifest_path(&root_manifest)
        .no_deps()
        .exec()
        .context("Failed to read workspace metadata")?;

    let mut workspace_members: Vec<_> = cargo_meta
        .packages
        .iter()
        .filter(|pkg| {
            pkg.targets
                .iter()
                .any(|t| t.kind.contains(&TargetKind::CDyLib))
        })
        .collect();

    // For LocalPath, check if user pointed to a specific package directory
    // (cargo metadata always resolves to workspace root, so we need to filter)
    if let SourceType::LocalPath(_) = &source_type {
        let target_manifest = root_manifest.canonicalize()?;
        let matching_pkg = workspace_members
            .iter()
            .find(|pkg| pkg.manifest_path.as_std_path() == target_manifest);
        
        if let Some(pkg) = matching_pkg {
            // User pointed to a specific package, not the workspace root
            workspace_members.retain(|p| p.id == pkg.id);
        }
    }

    if workspace_members.is_empty() {
        anyhow::bail!("No cdylib plugins found in {}", repo_dir.display());
    }

    let plugin_to_install = match &source_type {
        SourceType::GitHub { plugin: Some(plugin_name), .. } => {
            workspace_members
                .iter()
                .find(|pkg| pkg.name == *plugin_name)
                .ok_or_else(|| {
                    let available: Vec<&str> = workspace_members
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect();
                    anyhow::anyhow!(
                        "Plugin '{}' not found. Available plugins: {}",
                        plugin_name,
                        available.join(", ")
                    )
                })?
        }
        SourceType::LocalPath(_) | SourceType::GitHub { plugin: None, .. } => {
            if workspace_members.len() > 1 {
                let available: Vec<String> = workspace_members
                    .iter()
                    .map(|p| match &source_type {
                        SourceType::GitHub { org, repo, .. } => {
                            format!("{}/{}/{}", org, repo, p.name)
                        }
                        SourceType::LocalPath(_) => p.name.to_string(),
                    })
                    .collect();

                anyhow::bail!(
                    "Repository contains multiple plugins. Please specify which one:\n{}\n\nExample: kargo plugin install {}",
                    available
                        .iter()
                        .map(|name| format!("  - {}", name))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    available.first().ok_or_else(|| anyhow::anyhow!("No plugins available"))?
                );
            }
            workspace_members.first().ok_or_else(|| anyhow::anyhow!("No plugins found"))?
        }
    };

    let plugin_manifest_dir = plugin_to_install
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid manifest path"))?;

    build::build_plugin(plugin_manifest_dir.as_std_path()).await?;

    let plugin_metadata = metadata::extract_plugin_metadata(plugin_to_install.manifest_path.as_std_path())?;

    let lib_artifact = artifact::find_lib_artifact(plugin_manifest_dir.as_std_path(), &plugin_metadata.name)?;

    {
        let lib = unsafe { libloading::Library::new(&lib_artifact) }
            .context("Failed to load plugin library")?;

        let constructor: libloading::Symbol<kargo_plugin_api::CreateFn> = unsafe {
            lib.get(b"kargo_plugin_create")
                .context("Plugin does not export 'kargo_plugin_create' symbol")?
        };

        let _plugin = constructor();
        // Plugin is dropped here, validating it can be created successfully
    }

    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let plugin_install_dir = match &source_type {
        SourceType::GitHub { org, repo, .. } => config_dir
            .join("kargo")
            .join("plugins")
            .join(org)
            .join(repo)
            .join(branch_name)
            .join(&plugin_metadata.name)
            .join(&plugin_metadata.version),
        SourceType::LocalPath(_) => config_dir
            .join("kargo")
            .join("plugins")
            .join("local")
            .join(&plugin_metadata.name)
            .join(&plugin_metadata.version),
    };

    std::fs::create_dir_all(&plugin_install_dir)?;

    let lib_filename = lib_artifact
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid library filename"))?;
    let target_lib = plugin_install_dir.join(lib_filename);

    std::fs::copy(&lib_artifact, &target_lib)
        .context("Failed to copy plugin library to install directory")?;

    println!(
        "✓ Installed plugin: {} v{}",
        plugin_metadata.name, plugin_metadata.version
    );
    println!("  Location: {}", plugin_install_dir.display());

    Ok(())
}
