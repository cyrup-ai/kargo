use anyhow::{bail, Context, Result};
use log::info;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use syn::{parse_file, Item, ItemFn, ItemImpl, ItemMod};

/// Scan a plugin's source to verify it implements the required trait
pub fn verify_native_plugin(source_path: &Path) -> Result<PluginInfo> {
    let content = fs::read_to_string(source_path).with_context(|| {
        format!("Failed to read plugin source: {}", source_path.display())
    })?;

    // Skip cargo-generate templates by checking for specific template markers
    // Check for common cargo-generate builtin variables and custom template variables
    const CARGO_GENERATE_MARKERS: &[&str] = &[
        // Builtin placeholders (from cargo-generate documentation)
        "{{crate_name}}",
        "{{project-name}}",
        "{{project_name}}",
        "{{authors}}",
        "{{username}}",
        "{{crate_type}}",
        // Common custom variables (from kargo plugin templates)
        "{{plugin_name}}",
        "{{author_name}}",
        "{{author_email}}",
        // Liquid filter syntax (indicates template processing)
        "| pascal_case",
        "| snake_case",
        "| kebab-case",
    ];

    let has_template_markers = CARGO_GENERATE_MARKERS.iter().any(|m| content.contains(m));
    if has_template_markers {
        info!(
            "Skipping cargo-generate template file: {}",
            source_path.display()
        );
        bail!("Skipping cargo-generate template (contains placeholders)");
    }

    // Establish src directory (module root) based on the passed file
    let src_dir = source_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("Invalid source path: no parent for {}", source_path.display()))?;

    let mut scanner = ModuleScanner::new();
    scanner.scan_file(source_path, &src_dir)?;

    if !scanner.found_plugin_command || !scanner.found_create_fn {
        // Build a detailed error message for easier debugging
        let mut msg = String::new();
        msg.push_str(&format!(
            "Plugin at {} failed trait validation:\n",
            src_dir.display()
        ));
        msg.push_str(&format!(
            "  Scanned {} files:\n",
            scanner.scanned_files.len()
        ));
        for f in &scanner.scanned_files {
            msg.push_str(&format!("    - {}\n", f.display()));
        }
        msg.push_str(&format!(
            "  PluginCommand impl: {}\n",
            match &scanner.plugin_command_file {
                Some(p) => format!("✓ in {}", p.display()),
                None => "✗ not found".to_string(),
            }
        ));
        msg.push_str(&format!(
            "  kargo_plugin_create: {}\n",
            match &scanner.create_fn_file {
                Some(p) => format!("✓ in {}", p.display()),
                None => "✗ not found".to_string(),
            }
        ));
        bail!(msg);
    }

    let info = PluginInfo {
        implements_plugin_command: true,
        has_create_function: true,
        impl_type: scanner.impl_self_type,
    };
    log::debug!(
        "plugin trait scan OK: implements={}, create_fn={}, impl_type={:?}",
        info.implements_plugin_command,
        info.has_create_function,
        info.impl_type
    );
    Ok(info)
}

#[derive(Debug, Default)]
pub struct PluginInfo {
    pub implements_plugin_command: bool,
    pub has_create_function: bool,
    pub impl_type: Option<String>,
}

struct ModuleScanner {
    visited: HashSet<PathBuf>,
    pub found_plugin_command: bool,
    pub found_create_fn: bool,
    pub impl_self_type: Option<String>,
    pub scanned_files: Vec<PathBuf>,
    pub plugin_command_file: Option<PathBuf>,
    pub create_fn_file: Option<PathBuf>,
}

impl ModuleScanner {
    fn new() -> Self {
        Self {
            visited: HashSet::new(),
            found_plugin_command: false,
            found_create_fn: false,
            impl_self_type: None,
            scanned_files: Vec::new(),
            plugin_command_file: None,
            create_fn_file: None,
        }
    }

    fn scan_file(&mut self, file_path: &Path, module_dir: &Path) -> Result<()> {
        // Canonicalize to have stable keys for visited
        let can_file = fs::canonicalize(file_path).with_context(|| {
            format!(
                "Failed to canonicalize file path '{}'",
                file_path.display()
            )
        })?;

        if self.visited.contains(&can_file) {
            return Ok(());
        }
        self.visited.insert(can_file.clone());
        self.scanned_files.push(can_file.clone());

        let content = fs::read_to_string(&can_file).with_context(|| {
            format!("Failed to read module file: {}", can_file.display())
        })?;
        let syntax_tree = parse_file(&content).with_context(|| {
            format!("Failed to parse module file: {}", can_file.display())
        })?;

        for item in syntax_tree.items {
            match item {
                Item::Impl(impl_item) => {
                    if is_plugin_command_impl(&impl_item) {
                        self.found_plugin_command = true;
                        self.impl_self_type = extract_self_type(&impl_item);
                        if self.plugin_command_file.is_none() {
                            self.plugin_command_file = Some(can_file.clone());
                        }
                    }
                }
                Item::Fn(fn_item) => {
                    if is_plugin_create_function(&fn_item) {
                        self.found_create_fn = true;
                        if self.create_fn_file.is_none() {
                            self.create_fn_file = Some(can_file.clone());
                        }
                    }
                }
                Item::Mod(mod_item) => {
                    self.scan_module(&mod_item, module_dir, &can_file)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn scan_module(&mut self, mod_item: &ItemMod, parent_dir: &Path, current_file: &Path) -> Result<()> {
        // Inline module: scan its items directly
        if let Some((_, items)) = &mod_item.content {
            self.scan_inline_items(items, parent_dir)?;
            return Ok(());
        }

        // External module: resolve and scan file
        let mod_name = mod_item.ident.to_string();
        let (mod_file, sub_dir) = self
            .resolve_module_path(parent_dir, current_file, &mod_name)
            .with_context(|| {
                format!(
                    "Failed to resolve module '{}' relative to {}",
                    mod_name,
                    current_file.display()
                )
            })?;
        self.scan_file(&mod_file, &sub_dir)
    }

    fn scan_inline_items(&mut self, items: &[Item], module_dir: &Path) -> Result<()> {
        for item in items {
            match item {
                Item::Impl(impl_item) => {
                    if is_plugin_command_impl(impl_item) {
                        self.found_plugin_command = true;
                        self.impl_self_type = extract_self_type(impl_item);
                    }
                }
                Item::Fn(fn_item) => {
                    if is_plugin_create_function(fn_item) {
                        self.found_create_fn = true;
                    }
                }
                Item::Mod(mod_item) => {
                    // Nested inline module
                    if let Some((_, nested)) = &mod_item.content {
                        self.scan_inline_items(nested, module_dir)?;
                    } else {
                        // External nested module: resolve relative to module_dir
                        let mod_name = mod_item.ident.to_string();
                        let (mod_file, sub_dir) = self.resolve_module_path(module_dir, module_dir, &mod_name)?;
                        self.scan_file(&mod_file, &sub_dir)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Resolve an external module path using Rust's module rules.
    /// Returns (file_to_scan, directory_for_its_submodules).
    fn resolve_module_path(&self, parent_dir: &Path, current_file: &Path, mod_name: &str) -> Result<(PathBuf, PathBuf)> {
        // Determine the base directory where submodules of the current file live.
        // If current_file is .../foo.rs, submodules live in .../foo/
        // If current_file is .../foo/mod.rs, submodules live in .../foo/
        let current_dir = current_file
            .parent()
            .ok_or_else(|| anyhow::anyhow!("No parent directory for {}", current_file.display()))?;

        let stem_dir = if current_file.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            // .../foo/mod.rs -> .../foo
            current_dir.to_path_buf()
        } else {
            // .../foo.rs -> .../foo
            match current_file.file_stem() {
                Some(stem) => current_dir.join(stem),
                None => current_dir.to_path_buf(),
            }
        };

        // Try stem_dir/mod_name.rs
        let file_rs = stem_dir.join(format!("{}.rs", mod_name));
        if file_rs.exists() {
            return Ok((file_rs, stem_dir.join(mod_name)));
        }

        // Try stem_dir/mod_name/mod.rs
        let mod_rs = stem_dir.join(mod_name).join("mod.rs");
        if mod_rs.exists() {
            return Ok((mod_rs, stem_dir.join(mod_name)));
        }

        // As a fallback, try parent_dir level (handles root lib.rs case)
        let file_rs_alt = parent_dir.join(format!("{}.rs", mod_name));
        if file_rs_alt.exists() {
            return Ok((file_rs_alt, parent_dir.join(mod_name)));
        }
        let mod_rs_alt = parent_dir.join(mod_name).join("mod.rs");
        if mod_rs_alt.exists() {
            return Ok((mod_rs_alt, parent_dir.join(mod_name)));
        }

        bail!(
            "Module '{}' not found under '{}' or '{}'",
            mod_name,
            stem_dir.display(),
            parent_dir.display()
        );
    }
}

fn is_plugin_command_impl(impl_item: &ItemImpl) -> bool {
    if let Some((_, path, _)) = &impl_item.trait_ {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "PluginCommand";
        }
    }
    false
}

fn is_plugin_create_function(fn_item: &ItemFn) -> bool {
    fn_item.sig.ident == "kargo_plugin_create"
}

fn extract_self_type(impl_item: &ItemImpl) -> Option<String> {
    match &*impl_item.self_ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string()),
        _ => None,
    }
}

// End of file
