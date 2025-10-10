use anyhow::Result;
use jwalk::WalkDir;
use std::path::{Path, PathBuf};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use crate::models::Project;
use crate::Config;
use globset::{Glob, GlobSet, GlobSetBuilder};

/// Find all Rust projects starting from `root_path` with visual progress feedback
/// Returns Vec of Project with collected src and test files
/// Uses config.exclude_patterns to filter out directories matching glob patterns
pub fn find_projects_with_progress(root_path: &Path, config: &Config) -> Result<Vec<Project>> {
    // Build GlobSet from exclude patterns once (zero-allocation optimization)
    let mut builder = GlobSetBuilder::new();
    for pattern in &config.exclude_patterns {
        let glob = Glob::new(pattern)
            .map_err(|e| anyhow::anyhow!("Invalid glob pattern '{}': {}", pattern, e))?;
        builder.add(glob);
    }
    let globset = builder.build()
        .map_err(|e| anyhow::anyhow!("Failed to build glob matcher: {}", e))?;

    // Spinner for Cargo.toml discovery phase
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .map_err(|e| anyhow::anyhow!("Failed to set progress style: {e}"))?
    );
    pb.set_message("Scanning for Cargo.toml files...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let cargo_toml_paths = find_cargo_toml_files(root_path, &globset)?;

    pb.finish_with_message(format!("Found {} project(s)", cargo_toml_paths.len()));

    // Collect .rs files for each project
    let mut projects = Vec::new();
    for cargo_path in cargo_toml_paths {
        let mut project = Project::new(cargo_path.clone())?;
        collect_rust_files(&cargo_path, &mut project, &globset)?;
        projects.push(project);
    }

    Ok(projects)
}


/// Parallel directory traversal to find all Cargo.toml files
/// Uses jwalk with rayon for parallel scanning across all CPU cores
/// Filters directories using precompiled GlobSet for zero-allocation pattern matching
fn find_cargo_toml_files(root_path: &Path, globset: &GlobSet) -> Result<Vec<PathBuf>> {
    let mut cargo_toml_paths = Vec::new();

    // Clone globset for use in closure (Arc-based, cheap clone)
    let globset_clone = globset.clone();

    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .skip_hidden(true)
        .parallelism(jwalk::Parallelism::RayonNewPool(0))  // 0 = use all CPU cores
        .process_read_dir(move |_depth, _path, _state, entries| {
            entries.retain(|res| {
                if let Ok(entry) = res {
                    if entry.file_type().is_dir() {
                        let path = entry.path();
                        // Match against full path for patterns like **/vendor/**
                        return !globset_clone.is_match(&path);
                    }
                }
                true
            });
        })
        .into_iter()
        .filter_map(std::result::Result::ok)  // Skip permission denied errors
    {
        let path = entry.path();
        if path.file_name().is_some_and(|f| f == "Cargo.toml") {
            cargo_toml_paths.push(path);
        }
    }

    Ok(cargo_toml_paths)
}

/// Collect all .rs files from src/ and tests/ directories
fn collect_rust_files(cargo_toml: &Path, project: &mut Project, globset: &GlobSet) -> Result<()> {
    let project_dir = cargo_toml.parent()
        .ok_or_else(|| anyhow::anyhow!("Cargo.toml has no parent directory"))?;

    // Collect src files (if directory exists)
    let src_dir = project_dir.join("src");
    if src_dir.exists() && src_dir.is_dir() {
        project.src_files = collect_rs_files(&src_dir, globset)?;
    }

    // Collect test files (if directory exists)
    let tests_dir = project_dir.join("tests");
    if tests_dir.exists() && tests_dir.is_dir() {
        project.test_files = collect_rs_files(&tests_dir, globset)?;
    }

    Ok(())
}

/// Recursively collect all .rs files in a directory
/// Filters directories using precompiled GlobSet for zero-allocation pattern matching
fn collect_rs_files(dir: &Path, globset: &GlobSet) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Clone globset for use in closure (Arc-based, cheap clone)
    let globset_clone = globset.clone();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .skip_hidden(true)
        .process_read_dir(move |_depth, _path, _state, entries| {
            entries.retain(|res| {
                if let Ok(entry) = res {
                    if entry.file_type().is_dir() {
                        let path = entry.path();
                        // Match against full path for patterns like **/vendor/**
                        return !globset_clone.is_match(&path);
                    }
                }
                true
            });
        })
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }

    Ok(files)
}
