use anyhow::{Context, Result, anyhow};
use cargo_toml::Manifest;
use indicatif::{ProgressBar, ProgressStyle};
use jwalk::WalkDir;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;

// Re-export for plugin use
pub mod plugin;

/// Project type classification
#[derive(Debug, Serialize, Deserialize)]
pub enum ProjectType {
    Binary,
    Library,
    Both,
    Unknown,
}

/// Project build status
#[derive(Debug, Serialize, Deserialize)]
pub enum ProjectStatus {
    Working,
    Broken,
    Unknown,
}

/// Comprehensive project metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub project_type: ProjectType,
    pub status: ProjectStatus,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub is_workspace: bool,
    pub workspace_members: Vec<String>,
    pub indicators: HashMap<String, String>,
}

/// Find all Cargo.toml files in a directory tree
///
/// # Errors
///
/// Returns an error if directory traversal fails
pub fn find_cargo_toml_files(root_path: &Path) -> Result<Vec<PathBuf>> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Scanning for Cargo.toml files...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut cargo_toml_paths = Vec::new();
    for entry in WalkDir::new(root_path)
        .follow_links(true)
        .parallelism(jwalk::Parallelism::RayonNewPool(0))
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.file_name().is_some_and(|f| f == "Cargo.toml")
            && !path.to_string_lossy().contains("/target/")
        {
            cargo_toml_paths.push(path);
        }
    }

    pb.finish_with_message(format!("Found {} Cargo.toml files", cargo_toml_paths.len()));
    Ok(cargo_toml_paths)
}

/// Extract project information from Cargo.toml files in parallel
///
/// # Errors
///
/// Returns an error if progress bar template is invalid, or if the Arc/Mutex
/// operations fail during parallel extraction
pub fn extract_project_info(cargo_toml_paths: &[PathBuf]) -> Result<Vec<ProjectInfo>> {
    println!("Extracting project information...");

    let pb = ProgressBar::new(cargo_toml_paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    let projects = Arc::new(Mutex::new(Vec::new()));

    cargo_toml_paths.par_iter().for_each(|path| {
        let mut project_info = extract_single_project_info(path);
        if let Ok(info) = &mut project_info {
            info.project_type = determine_project_type(path);
        }

        if let Ok(info) = project_info {
            match projects.lock() {
                Ok(mut proj) => proj.push(info),
                Err(e) => eprintln!("Failed to lock projects mutex: {e}"),
            }
        } else {
            println!("Warning: Failed to extract info from {}", path.display());
        }

        pb.inc(1);
    });

    pb.finish_with_message("Project information extracted");

    Arc::try_unwrap(projects).map_or_else(
        |_| Err(anyhow!("Failed to unwrap Arc - still has multiple references")),
        |mutex| match mutex.into_inner() {
            Ok(data) => Ok(data),
            Err(e) => Err(anyhow!("Failed to extract data from mutex: {e}")),
        },
    )
}

fn extract_single_project_info(path: &Path) -> Result<ProjectInfo> {
    let manifest = Manifest::from_path(path)
        .with_context(|| format!("Failed to parse Cargo.toml at {}", path.display()))?;

    let package = manifest
        .package
        .as_ref()
        .with_context(|| format!("No package section in {}", path.display()))?;

    let dependencies = manifest
        .dependencies
        .keys()
        .map(std::string::ToString::to_string)
        .collect();

    let workspace_members = manifest
        .workspace
        .as_ref()
        .map_or_else(Vec::new, |workspace| workspace.members.clone());

    let version = match &package.version {
        cargo_toml::Inheritable::Set(v) => v.clone(),
        cargo_toml::Inheritable::Inherited => "0.0.0".to_string(),
    };

    let description = package.description.as_ref().and_then(|desc| match desc {
        cargo_toml::Inheritable::Set(v) => Some(v.clone()),
        cargo_toml::Inheritable::Inherited => None,
    });

    Ok(ProjectInfo {
        path: path
            .parent()
            .map_or_else(|| ".".to_string(), |p| p.to_string_lossy().to_string()),
        name: package.name.clone(),
        version,
        description,
        project_type: ProjectType::Unknown,
        status: ProjectStatus::Unknown,
        dependencies,
        tags: Vec::new(),
        is_workspace: manifest.workspace.is_some(),
        workspace_members,
        indicators: HashMap::new(),
    })
}

fn determine_project_type(cargo_toml_path: &Path) -> ProjectType {
    let Some(parent_dir) = cargo_toml_path.parent() else {
        return ProjectType::Unknown;
    };

    let has_main = parent_dir.join("src/main.rs").exists();
    let has_lib = parent_dir.join("src/lib.rs").exists();

    match (has_main, has_lib) {
        (true, true) => ProjectType::Both,
        (true, false) => ProjectType::Binary,
        (false, true) => ProjectType::Library,
        (false, false) => ProjectType::Unknown,
    }
}

/// Check build status of projects by running cargo check
///
/// # Errors
///
/// Returns an error if the Arc/Mutex operations fail during parallel status checking
pub async fn check_project_status(projects: Vec<ProjectInfo>) -> Result<Vec<ProjectInfo>> {
    println!("Checking project status...");

    let semaphore = Arc::new(Semaphore::new(4));
    let updated_projects = Arc::new(Mutex::new(Vec::new()));

    let mut tasks = Vec::new();
    for project in projects {
        let semaphore = Arc::clone(&semaphore);
        let updated_projects = Arc::clone(&updated_projects);

        let task = tokio::spawn(async move {
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(e) => {
                    eprintln!("Failed to acquire semaphore: {e}");
                    return;
                }
            };
            println!("Checking project: {}", project.name);

            let mut updated_project = project;
            updated_project.status = check_single_project_status(&updated_project.path).await;

            match updated_projects.lock() {
                Ok(mut proj) => proj.push(updated_project),
                Err(e) => eprintln!("Failed to lock updated_projects mutex: {e}"),
            }
        });

        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    println!("Project status check completed");
    Arc::try_unwrap(updated_projects).map_or_else(
        |_| Err(anyhow!("Failed to unwrap Arc - still has multiple references")),
        |mutex| match mutex.into_inner() {
            Ok(data) => Ok(data),
            Err(e) => Err(anyhow!("Failed to extract data from mutex: {e}")),
        },
    )
}

async fn check_single_project_status(project_path: &str) -> ProjectStatus {
    let output = tokio::process::Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(project_path)
        .output()
        .await;

    match output {
        Ok(output) => {
            if output.status.success() {
                ProjectStatus::Working
            } else {
                ProjectStatus::Broken
            }
        }
        Err(_) => ProjectStatus::Unknown,
    }
}

/// Analyze project relationships (workspaces, dependencies)
#[must_use]
pub fn analyze_relationships(mut projects: Vec<ProjectInfo>) -> Vec<ProjectInfo> {
    println!("Analyzing project relationships...");

    let project_map: HashMap<String, (usize, String)> = projects
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.clone(), (i, p.path.clone())))
        .collect();

    let mut dep_usage: HashMap<String, Vec<String>> = HashMap::new();
    for project in &projects {
        for dep in &project.dependencies {
            dep_usage
                .entry(dep.clone())
                .or_default()
                .push(project.name.clone());
        }
    }

    let shared_deps: Vec<String> = dep_usage
        .iter()
        .filter(|(_, users)| users.len() >= 2)
        .map(|(dep, _)| dep.clone())
        .collect();

    for i in 0..projects.len() {
        if projects[i].is_workspace {
            let member_count = projects[i].workspace_members.len();
            projects[i]
                .indicators
                .insert("workspace_members".to_string(), member_count.to_string());

            let root_path = projects[i].path.clone();
            for member_name in &projects[i].workspace_members.clone() {
                if let Some(&(member_idx, _)) = project_map.get(member_name) {
                    projects[member_idx]
                        .indicators
                        .insert("workspace_root".to_string(), root_path.clone());
                }
            }
        }

        if let Some(dependents) = dep_usage.get(&projects[i].name)
            && !dependents.is_empty()
        {
            projects[i]
                .indicators
                .insert("dependents".to_string(), dependents.join(","));
        }

        let project_shared_deps: Vec<String> = projects[i]
            .dependencies
            .iter()
            .filter(|dep| shared_deps.contains(dep))
            .cloned()
            .collect();

        if !project_shared_deps.is_empty() {
            projects[i]
                .indicators
                .insert("shared_deps".to_string(), project_shared_deps.join(","));
        }
    }

    projects
}

/// Generate YAML inventory file
///
/// # Errors
///
/// Returns an error if YAML serialization fails or if writing to the output file fails
pub fn generate_index_yaml(projects: &[ProjectInfo], output_path: &Path) -> Result<()> {
    println!("Generating inventory...");

    let yaml = serde_yaml_ok::to_string(projects)?;
    std::fs::write(output_path, yaml)?;

    println!("✅ Inventory generated at {}", output_path.display());
    println!("   Total projects: {}", projects.len());
    Ok(())
}
