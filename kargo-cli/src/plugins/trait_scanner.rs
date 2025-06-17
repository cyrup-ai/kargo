use anyhow::{Context, Result};
use log::info;
use std::path::Path;
use syn::{Item, ItemFn, ItemImpl, ItemStatic, parse_file};

/// Scan a plugin's source to verify it implements the required trait
pub fn verify_native_plugin(source_path: &Path) -> Result<PluginInfo> {
    let content = std::fs::read_to_string(source_path)
        .with_context(|| format!("Failed to read plugin source: {}", source_path.display()))?;

    // Skip cargo-generate templates (they contain unparseable {{}} placeholders)
    if content.contains("{{") && content.contains("}}") {
        info!(
            "Skipping cargo-generate template file: {}",
            source_path.display()
        );
        anyhow::bail!("Skipping cargo-generate template (contains placeholders)");
    }

    let syntax_tree = parse_file(&content)
        .with_context(|| format!("Failed to parse plugin source: {}", source_path.display()))?;

    let mut plugin_info = PluginInfo::default();

    for item in syntax_tree.items {
        match item {
            Item::Impl(impl_item) => {
                if is_native_plugin_impl(&impl_item) {
                    plugin_info.implements_native_plugin = true;
                    plugin_info.impl_type = extract_self_type(&impl_item);
                    info!(
                        "Found NativePlugin implementation for type: {:?}",
                        plugin_info.impl_type
                    );
                } else if is_plugin_command_impl(&impl_item) {
                    plugin_info.implements_plugin_command = true;
                    info!("Found PluginCommand implementation (legacy)");
                }
            }
            Item::Static(static_item) => {
                if is_plugin_declaration(&static_item) {
                    plugin_info.has_declaration = true;
                    info!("Found plugin declaration");
                }
            }
            Item::Fn(fn_item) => {
                if is_plugin_create_function(&fn_item) {
                    plugin_info.has_create_function = true;
                    info!("Found kargo_plugin_create function");
                }
            }
            _ => {}
        }
    }

    // Check for either new trait or legacy interface
    if !plugin_info.implements_native_plugin && !plugin_info.implements_plugin_command {
        anyhow::bail!("Plugin does not implement NativePlugin trait or PluginCommand interface");
    }

    // Require the create function for dynamic loading
    if !plugin_info.has_create_function {
        anyhow::bail!("Plugin missing kargo_plugin_create function for dynamic loading");
    }

    Ok(plugin_info)
}

#[derive(Debug, Default)]
pub struct PluginInfo {
    pub implements_native_plugin: bool,
    pub implements_plugin_command: bool,
    pub has_declaration: bool,
    pub has_create_function: bool,
    pub impl_type: Option<String>,
}

fn is_native_plugin_impl(impl_item: &ItemImpl) -> bool {
    if let Some((_, path, _)) = &impl_item.trait_ {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "NativePlugin";
        }
    }
    false
}

fn is_plugin_command_impl(impl_item: &ItemImpl) -> bool {
    if let Some((_, path, _)) = &impl_item.trait_ {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "PluginCommand";
        }
    }
    false
}

fn is_plugin_declaration(static_item: &ItemStatic) -> bool {
    static_item.ident == "KARGO_PLUGIN_DECLARATION"
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
