//! Module for finding updatable files within a directory

use anyhow::Result;
use indicatif::ProgressBar;
use jwalk::{Parallelism, WalkDir};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Find all Cargo.toml files recursively in a directory
pub fn find_cargo_toml_files(root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let root_path = root.as_ref().to_string_lossy();
    let pb = ProgressBar::new_spinner();
    pb.set_message(format!("Scanning for Cargo.toml files in {}...", root_path));
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut cargo_toml_paths = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(true)
        .parallelism(Parallelism::RayonNewPool(0)) // Use available cores
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

/// Find all rust-script files recursively in a directory
pub fn find_rust_script_files(root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let root_path = root.as_ref().to_string_lossy();
    let pb = ProgressBar::new_spinner();
    pb.set_message(format!(
        "Scanning for Rust script files in {}...",
        root_path
    ));
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut rust_script_paths = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(true)
        .parallelism(Parallelism::RayonNewPool(0)) // Use available cores
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "rs") {
            // Skip files in target directories
            if !path.to_string_lossy().contains("/target/") {
                // Check if it's a rust-script with cargo dependencies
                if is_rust_script(&path)? {
                    rust_script_paths.push(path);
                }
            }
        }
    }

    pb.finish_with_message(format!(
        "Found {} Rust script files",
        rust_script_paths.len()
    ));
    Ok(rust_script_paths)
}

/// Check if a file is a rust-script with cargo dependencies
fn is_rust_script(path: impl AsRef<Path>) -> Result<bool> {
    let content = std::fs::read_to_string(path)?;

    // Look for cargo section in either format
    let has_cargo_section = content.contains("```cargo") || content.contains("cargo-deps:");

    Ok(has_cargo_section)
}
