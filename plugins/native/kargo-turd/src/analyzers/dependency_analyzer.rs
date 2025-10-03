use std::collections::HashSet;
use std::fs;
use std::path::Path;
use toml::Value;
use crate::models::UnusedDependency;

// ============================================================================
// BASIC DEPENDENCY ANALYSIS (all files together)
// ============================================================================

/// Analyze unused dependencies by comparing Cargo.toml against source imports
///
/// # Arguments
/// * `cargo_toml` - Path to Cargo.toml file
/// * `all_file_contents` - Vec of (file_path, file_content) tuples
///
/// # Returns
/// Vec of unused dependencies with their Cargo.toml section
pub fn analyze_unused_dependencies(
    cargo_toml: &Path,
    all_file_contents: &[(String, String)],
) -> anyhow::Result<Vec<UnusedDependency>> {
    let content = fs::read_to_string(cargo_toml)?;
    let toml: Value = toml::from_str(&content)?;

    let mut deps = Vec::new();

    // Collect dependencies from all sections
    if let Some(dependencies) = toml.get("dependencies").and_then(|v| v.as_table()) {
        for (name, _) in dependencies {
            deps.push((name.clone(), "[dependencies]".to_string()));
        }
    }

    if let Some(dev_deps) = toml.get("dev-dependencies").and_then(|v| v.as_table()) {
        for (name, _) in dev_deps {
            deps.push((name.clone(), "[dev-dependencies]".to_string()));
        }
    }

    if let Some(build_deps) = toml.get("build-dependencies").and_then(|v| v.as_table()) {
        for (name, _) in build_deps {
            deps.push((name.clone(), "[build-dependencies]".to_string()));
        }
    }

    // Find used dependencies by scanning all source files
    let used_deps = collect_used_deps(all_file_contents);

    // Find unused: dependencies that aren't in the used set
    Ok(deps.into_iter()
        .filter(|(name, _)| !used_deps.contains(&normalize_crate_name(name)))
        .map(|(name, section)| UnusedDependency {
            name,
            cargo_toml: cargo_toml.to_string_lossy().to_string(),
            section,
        })
        .collect())
}

/// Normalize crate name from Cargo.toml format to Rust import format
///
/// Cargo.toml uses kebab-case: "tokio-util"
/// Rust uses snake_case: tokio_util
///
/// # Examples
/// ```
/// use kargo_turd::normalize_crate_name;
///
/// assert_eq!(normalize_crate_name("tokio-util"), "tokio_util");
/// assert_eq!(normalize_crate_name("serde_json"), "serde_json");  // No change
/// ```
pub fn normalize_crate_name(name: &str) -> String {
    name.replace("-", "_")
}

/// Collect all used dependency names from source files
fn collect_used_deps(files: &[(String, String)]) -> HashSet<String> {
    let mut used = HashSet::new();

    for (_path, content) in files {
        // Parse file as Rust source
        if let Ok(ast) = syn::parse_file(content) {
            // Find all use statements
            for item in ast.items {
                if let syn::Item::Use(use_item) = item {
                    extract_crate_names(&use_item.tree, &mut used);
                }
            }
        }
    }

    used
}

/// Recursively extract crate names from use tree
///
/// Handles all use statement patterns:
/// - `use foo;` → extracts "foo"
/// - `use foo::bar;` → extracts "foo"
/// - `use foo::{bar, baz};` → extracts "foo"
/// - `use foo::bar as qux;` → extracts "foo"
/// - `use foo::*;` → extracts "foo"
fn extract_crate_names(tree: &syn::UseTree, used: &mut HashSet<String>) {
    match tree {
        syn::UseTree::Path(path) => {
            // use foo::bar → extract "foo" (the root crate)
            used.insert(path.ident.to_string());
            extract_crate_names(&path.tree, used);
        }
        syn::UseTree::Name(name) => {
            // use foo → extract "foo"
            used.insert(name.ident.to_string());
        }
        syn::UseTree::Group(group) => {
            // use foo::{bar, baz} → recurse into group
            for item in &group.items {
                extract_crate_names(item, used);
            }
        }
        syn::UseTree::Glob(_) => {
            // use foo::* → nothing to extract (already got "foo" from Path)
        }
        syn::UseTree::Rename(rename) => {
            // use foo as bar → extract "foo" (the original name)
            used.insert(rename.ident.to_string());
        }
    }
}

// ============================================================================
// CONTEXT-AWARE DEPENDENCY ANALYSIS
// ============================================================================

/// Analyze unused dependencies with separate checking per dependency type
///
/// More accurate than basic analysis:
/// - [dependencies] must be used in src/ files
/// - [dev-dependencies] must be used in tests/ files
/// - [build-dependencies] must be used in build.rs (if exists)
///
/// # Arguments
/// * `cargo_toml` - Path to Cargo.toml
/// * `src_files` - Vec of (path, content) for src/ files
/// * `test_files` - Vec of (path, content) for tests/ files
/// * `has_build_rs` - Whether build.rs exists
pub fn analyze_unused_dependencies_with_context(
    cargo_toml: &Path,
    src_files: &[(String, String)],
    test_files: &[(String, String)],
    has_build_rs: bool,
) -> anyhow::Result<Vec<UnusedDependency>> {
    let content = fs::read_to_string(cargo_toml)?;
    let toml: Value = toml::from_str(&content)?;

    let mut unused = Vec::new();

    // Check [dependencies] - must be used in src/
    if let Some(dependencies) = toml.get("dependencies").and_then(|v| v.as_table()) {
        let used = collect_used_deps(src_files);

        for (name, _) in dependencies {
            if !used.contains(&normalize_crate_name(name)) {
                unused.push(UnusedDependency {
                    name: name.clone(),
                    cargo_toml: cargo_toml.to_string_lossy().to_string(),
                    section: "[dependencies]".to_string(),
                });
            }
        }
    }

    // Check [dev-dependencies] - must be used in tests/
    if let Some(dev_deps) = toml.get("dev-dependencies").and_then(|v| v.as_table()) {
        let used = collect_used_deps(test_files);

        for (name, _) in dev_deps {
            if !used.contains(&normalize_crate_name(name)) {
                unused.push(UnusedDependency {
                    name: name.clone(),
                    cargo_toml: cargo_toml.to_string_lossy().to_string(),
                    section: "[dev-dependencies]".to_string(),
                });
            }
        }
    }

    // Check [build-dependencies] - only if build.rs exists
    // Note: Would need to parse build.rs separately
    // For now, we skip this check (acceptable limitation)
    if has_build_rs && toml.get("build-dependencies").and_then(|v| v.as_table()).is_some() {
        // TODO: Parse build.rs and check usage
        // For v1, we'll skip build-dependencies
    }

    Ok(unused)
}
