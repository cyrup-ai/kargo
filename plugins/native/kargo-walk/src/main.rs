use anyhow::{Context, Result, anyhow};
use cargo_toml::Manifest;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use jwalk::WalkDir;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;

#[derive(Debug, Serialize, Deserialize)]
enum ProjectType {
    Binary,
    Library,
    Both,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
enum ProjectStatus {
    Working,
    Broken,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectInfo {
    path: String,
    name: String,
    version: String,
    description: Option<String>,
    project_type: ProjectType,
    status: ProjectStatus,
    dependencies: Vec<String>,
    tags: Vec<String>,
    is_workspace: bool,
    workspace_members: Vec<String>,
    indicators: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Forge Inventory Tool - Scanning projects in /home/ubuntu/forge");

    // Step 1: Find all Cargo.toml files
    let cargo_toml_paths = find_cargo_toml_files("/home/ubuntu/forge")?;
    println!("Found {} Cargo.toml files", cargo_toml_paths.len());

    // Take only the first 10 projects for testing
    let limited_paths = cargo_toml_paths.into_iter().take(10).collect();
    println!("Limited to 10 projects for testing");

    // Step 2: Extract project information in parallel
    let mp = MultiProgress::new();
    let projects = extract_project_info(limited_paths, &mp)?;

    // Step 3: Check project status concurrently
    let projects = check_project_status(projects).await?;

    // Step 4: Analyze project relationships (simplified for now)
    let projects_with_relationships = analyze_relationships(projects);

    // Step 5: Generate index.yaml
    generate_index_yaml(&projects_with_relationships)?;

    println!("✅ Completed inventory process. Results saved to index.yaml");
    Ok(())
}

fn find_cargo_toml_files(root_path: &str) -> Result<Vec<PathBuf>> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("Scanning for Cargo.toml files...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut cargo_toml_paths = Vec::new();
    for entry in WalkDir::new(root_path)
        .follow_links(true)
        .parallelism(jwalk::Parallelism::RayonNewPool(0)) // Use available cores
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.file_name().map_or(false, |f| f == "Cargo.toml") {
            // Skip nested Cargo.toml files in target directories
            if !path.to_string_lossy().contains("/target/") {
                cargo_toml_paths.push(path);
            }
        }
    }

    pb.finish_with_message(format!("Found {} Cargo.toml files", cargo_toml_paths.len()));
    Ok(cargo_toml_paths)
}

fn extract_project_info(
    cargo_toml_paths: Vec<PathBuf>,
    mp: &MultiProgress,
) -> Result<Vec<ProjectInfo>> {
    println!("Extracting project information...");

    let pb = mp.add(ProgressBar::new(cargo_toml_paths.len() as u64));
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
                Err(e) => eprintln!("Failed to lock projects mutex: {}", e),
            }
        } else {
            println!("Warning: Failed to extract info from {:?}", path);
        }

        pb.inc(1);
    });

    pb.finish_with_message("Project information extracted");

    match Arc::try_unwrap(projects) {
        Ok(mutex) => match mutex.into_inner() {
            Ok(data) => Ok(data),
            Err(e) => Err(anyhow!("Failed to extract data from mutex: {}", e)),
        },
        Err(_) => Err(anyhow!(
            "Failed to unwrap Arc - still has multiple references"
        )),
    }
}

fn extract_single_project_info(path: &Path) -> Result<ProjectInfo> {
    let manifest = Manifest::from_path(path)
        .with_context(|| format!("Failed to parse Cargo.toml at {:?}", path))?;

    let package = manifest
        .package
        .as_ref()
        .with_context(|| format!("No package section in {:?}", path))?;

    let dependencies = manifest
        .dependencies
        .keys()
        .map(|k| k.to_string())
        .collect();

    // Handle workspace members
    let workspace_members = if let Some(workspace) = &manifest.workspace {
        workspace.members.clone()
    } else {
        Vec::new()
    };

    // Extract version from package.version (Inheritable<String>)
    let version = match &package.version {
        cargo_toml::Inheritable::Set(v) => v.clone(),
        cargo_toml::Inheritable::Inherited => "0.0.0".to_string(),
    };

    // Extract description from package.description (Option<Inheritable<String>>)
    let description = package.description.as_ref().and_then(|desc| match desc {
        cargo_toml::Inheritable::Set(v) => Some(v.clone()),
        cargo_toml::Inheritable::Inherited => None,
    });

    Ok(ProjectInfo {
        path: match path.parent() {
            Some(p) => p.to_string_lossy().to_string(),
            None => ".".to_string(),
        },
        name: package.name.clone(),
        version,
        description,
        project_type: ProjectType::Unknown, // Will be set later
        status: ProjectStatus::Unknown,     // Will be set later
        dependencies,
        tags: Vec::new(), // Can be enhanced later
        is_workspace: manifest.workspace.is_some(),
        workspace_members,
        indicators: HashMap::new(),
    })
}

fn determine_project_type(cargo_toml_path: &Path) -> ProjectType {
    let parent_dir = match cargo_toml_path.parent() {
        Some(dir) => dir,
        None => return ProjectType::Unknown,
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

async fn check_project_status(projects: Vec<ProjectInfo>) -> Result<Vec<ProjectInfo>> {
    println!("Checking project status...");

    // Limit concurrent cargo check operations
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
                    eprintln!("Failed to acquire semaphore: {}", e);
                    return;
                }
            };
            println!("Checking project: {}", project.name);

            let mut updated_project = project;
            updated_project.status = check_single_project_status(&updated_project.path).await;

            match updated_projects.lock() {
                Ok(mut proj) => proj.push(updated_project),
                Err(e) => eprintln!("Failed to lock updated_projects mutex: {}", e),
            }
        });

        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    println!("Project status check completed");
    match Arc::try_unwrap(updated_projects) {
        Ok(mutex) => match mutex.into_inner() {
            Ok(data) => Ok(data),
            Err(e) => Err(anyhow!("Failed to extract data from mutex: {}", e)),
        },
        Err(_) => Err(anyhow!(
            "Failed to unwrap Arc - still has multiple references"
        )),
    }
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

fn analyze_relationships(projects: Vec<ProjectInfo>) -> Vec<ProjectInfo> {
    println!("Analyzing project relationships...");
    // For this simple implementation, we'll just return the projects without modification
    // In a more complex implementation, this could analyze dependencies and workspace relationships
    projects
}

fn generate_index_yaml(projects: &[ProjectInfo]) -> Result<()> {
    println!("Generating index.yaml...");

    let yaml = serde_yaml_ok::to_string(projects)?;
    std::fs::write("index.yaml", yaml)?;

    println!("✅ index.yaml generated with {} projects", projects.len());
    Ok(())
}
