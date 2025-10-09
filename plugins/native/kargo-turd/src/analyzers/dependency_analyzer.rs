use std::collections::HashSet;
use std::fs;
use std::path::Path;
use toml::Value;
use crate::models::UnusedDependency;
use syn::{self, visit::Visit};

// ============================================================================
// BASIC DEPENDENCY ANALYSIS (all files together)
// ============================================================================

/// Analyze unused dependencies by comparing Cargo.toml against source imports
///
/// # Arguments
/// * `cargo_toml` - Path to Cargo.toml file
/// * `all_file_contents` - Vec of (`file_path`, `file_content`) tuples
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

    // Helper to build a minimal TOML snippet for a dep in a section
    fn build_dep_snippet(toml: &Value, section: &str, name: &str) -> String {
        let key = match section {
            "[dependencies]" => "dependencies",
            "[dev-dependencies]" => "dev-dependencies",
            "[build-dependencies]" => "build-dependencies",
            _ => return String::new(),
        };
        if let Some(table) = toml.get(key).and_then(|v| v.as_table())
            && let Some(val) = table.get(name)
        {
            // Construct a minimal doc with just this section and one entry
            let mut sec = toml::map::Map::new();
            sec.insert(name.to_string(), val.clone());
            let mut root = toml::map::Map::new();
            root.insert(key.to_string(), Value::Table(sec));
            match toml::to_string(&Value::Table(root)) {
                Ok(s) => s,
                Err(_) => format!("{section}\n{name} = ...\n"),
            }
        } else {
            String::new()
        }
    }

    fn to_unified_diff(snippet: &str) -> String {
        let mut out = String::new();
        for line in snippet.lines() {
            out.push_str("- ");
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    // Find unused: dependencies that aren't in the used set
    Ok(deps.into_iter()
        .filter(|(name, _)| !used_deps.contains(&normalize_crate_name(name)))
        .map(|(name, section)| {
            let raw = build_dep_snippet(&toml, &section, &name);
            let snippet = raw.trim_end().to_string();
            let toml_diff = to_unified_diff(&snippet);
            UnusedDependency {
                name,
                cargo_toml: cargo_toml.to_string_lossy().to_string(),
                section,
                toml_snippet: snippet,
                toml_diff,
            }
        })
        .collect())
}

/// Normalize crate name from Cargo.toml format to Rust import format
///
/// Cargo.toml uses kebab-case: "tokio-util"
/// Rust uses `snake_case`: `tokio_util`
///
/// # Examples
/// ```
/// use kargo_turd::normalize_crate_name;
///
/// assert_eq!(normalize_crate_name("tokio-util"), "tokio_util");
/// assert_eq!(normalize_crate_name("serde_json"), "serde_json");  // No change
/// ```
#[must_use] 
pub fn normalize_crate_name(name: &str) -> String {
    name.replace('-', "_")
}

/// Visitor to collect root crate usages from various syntax positions
struct CrateUseCollector<'a> {
    used: &'a mut HashSet<String>,
}

impl<'a> CrateUseCollector<'a> {
    fn add_path(&mut self, path: &syn::Path) {
        if let Some(seg) = path.segments.first() {
            self.used.insert(seg.ident.to_string());
        }
    }
}

impl<'a, 'ast> syn::visit::Visit<'ast> for CrateUseCollector<'a> {
    fn visit_expr_path(&mut self, i: &'ast syn::ExprPath) {
        self.add_path(&i.path);
        syn::visit::visit_expr_path(self, i);
    }
    fn visit_type_path(&mut self, i: &'ast syn::TypePath) {
        // TypePath carries a Path; record its root segment
        self.add_path(&i.path);
        syn::visit::visit_type_path(self, i);
    }
    fn visit_macro(&mut self, i: &'ast syn::Macro) {
        // syn 2.x Macro has a Path directly
        self.add_path(&i.path);
        syn::visit::visit_macro(self, i);
    }
    fn visit_attribute(&mut self, i: &'ast syn::Attribute) {
        // Capture crate names referenced in attributes like #[tokio::test]
        self.add_path(i.path());
        syn::visit::visit_attribute(self, i);
    }
    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        extract_crate_names(&i.tree, self.used);
        syn::visit::visit_item_use(self, i);
    }
}

/// Collect all used dependency names from source files (enhanced)
fn collect_used_deps(files: &[(String, String)]) -> HashSet<String> {
    let mut used = HashSet::new();

    for (_path, content) in files {
        if let Ok(ast) = syn::parse_file(content) {
            let mut visitor = CrateUseCollector { used: &mut used };
            visitor.visit_file(&ast);
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
/// * `build_files` - Vec of (path, content) for build.rs file (empty vec if no build.rs)
pub fn analyze_unused_dependencies_with_context(
    cargo_toml: &Path,
    src_files: &[(String, String)],
    test_files: &[(String, String)],
    build_files: &[(String, String)],
) -> anyhow::Result<Vec<UnusedDependency>> {
    let content = fs::read_to_string(cargo_toml)?;
    let toml: Value = toml::from_str(&content)?;

    let mut unused = Vec::new();

    // Local helper to build a minimal TOML snippet for a dep in a section
    fn build_dep_snippet_ctx(toml: &Value, section: &str, name: &str) -> String {
        let key = match section {
            "[dependencies]" => "dependencies",
            "[dev-dependencies]" => "dev-dependencies",
            "[build-dependencies]" => "build-dependencies",
            _ => return String::new(),
        };
        if let Some(table) = toml.get(key).and_then(|v| v.as_table())
            && let Some(val) = table.get(name)
        {
            let mut sec = toml::map::Map::new();
            sec.insert(name.to_string(), val.clone());
            let mut root = toml::map::Map::new();
            root.insert(key.to_string(), Value::Table(sec));
            return toml::to_string(&Value::Table(root)).unwrap_or_else(|_| format!("{section}\n{name} = ...\n"));
        }
        String::new()
    }

    // Check [dependencies] - must be used in src/
    if let Some(dependencies) = toml.get("dependencies").and_then(|v| v.as_table()) {
        let used = collect_used_deps(src_files);

        for (name, _) in dependencies {
            if !used.contains(&normalize_crate_name(name)) {
                let section = "[dependencies]".to_string();
                let snippet = build_dep_snippet_ctx(&toml, &section, name);
                let toml_diff = snippet.lines().map(|l| format!("- {}\n", l)).collect::<String>();
                unused.push(UnusedDependency {
                    name: name.clone(),
                    cargo_toml: cargo_toml.to_string_lossy().to_string(),
                    section,
                    toml_snippet: snippet,
                    toml_diff,
                });
            }
        }
    }

    // Check [dev-dependencies] - must be used in tests/
    if let Some(dev_deps) = toml.get("dev-dependencies").and_then(|v| v.as_table()) {
        let used = collect_used_deps(test_files);

        for (name, _) in dev_deps {
            if !used.contains(&normalize_crate_name(name)) {
                let section = "[dev-dependencies]".to_string();
                let snippet = build_dep_snippet_ctx(&toml, &section, name);
                let toml_diff = snippet.lines().map(|l| format!("- {}\n", l)).collect::<String>();
                unused.push(UnusedDependency {
                    name: name.clone(),
                    cargo_toml: cargo_toml.to_string_lossy().to_string(),
                    section,
                    toml_snippet: snippet,
                    toml_diff,
                });
            }
        }
    }

    // Check [build-dependencies] - must be used in build.rs
    if !build_files.is_empty()
        && let Some(build_deps) = toml.get("build-dependencies").and_then(|v| v.as_table())
    {
        let used = collect_used_deps(build_files);

        for (name, _) in build_deps {
            if !used.contains(&normalize_crate_name(name)) {
                let section = "[build-dependencies]".to_string();
                let snippet = build_dep_snippet_ctx(&toml, &section, name);
                let toml_diff = snippet.lines().map(|l| format!("- {}\n", l)).collect::<String>();
                unused.push(UnusedDependency {
                    name: name.clone(),
                    cargo_toml: cargo_toml.to_string_lossy().to_string(),
                    section,
                    toml_snippet: snippet,
                    toml_diff,
                });
            }
        }
    }

    Ok(unused)
}
