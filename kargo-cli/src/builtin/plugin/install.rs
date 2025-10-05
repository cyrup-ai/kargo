use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, TargetKind};
use kargo_plugin_api::PluginCommand;
use super::{build, git, metadata, parser};
use parser::SourceType;

pub async fn install_plugin(source: &str, branch: Option<&String>) -> Result<()> {
    let branch_name = branch.map_or("main", std::string::String::as_str);

    let source_type = parser::parse_source(source)?;

    let repo_dir = match &source_type {
        SourceType::GitHub { org, repo, plugin: _ } => {
            let temp = tempfile::tempdir().context("Failed to create temporary directory")?;
            let url = format!("https://github.com/{org}/{repo}");
            git::clone_repository(&url, temp.path()).await?;
            temp.keep()
        }
        SourceType::LocalPath(path) => path.clone(),
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

    let workspace_members: Vec<_> = cargo_meta
        .packages
        .iter()
        .filter(|pkg| {
            pkg.targets
                .iter()
                .any(|t| t.kind.contains(&TargetKind::CDyLib))
        })
        .collect();

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

    let lib_artifact = build::find_lib_artifact(plugin_manifest_dir.as_std_path(), &plugin_metadata.name)?;

    {
        let lib = unsafe { libloading::Library::new(&lib_artifact) }
            .context("Failed to load plugin library")?;

        #[allow(improper_ctypes_definitions)]
        type PluginCreate = unsafe extern "C" fn() -> *mut dyn PluginCommand;

        let constructor: libloading::Symbol<PluginCreate> = unsafe {
            lib.get(b"kargo_plugin_create")
                .context("Plugin does not export 'kargo_plugin_create' symbol")?
        };

        let _plugin_ptr = unsafe { constructor() };
        if _plugin_ptr.is_null() {
            anyhow::bail!("Plugin constructor returned null pointer");
        }
        unsafe {
            let _ = Box::from_raw(_plugin_ptr);
        }
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
