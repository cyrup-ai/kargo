use anyhow::{Context, Result};
use kargo_plugin_api::{KargoPluginCommand, KargoPluginCreateFn};
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::wasm_adapter::WasmPluginAdapter;

/// A manifest file that describes one or more plugins.
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginManifest {
    /// Plugin manifests can describe multiple plugins in a single file
    pub plugins: Vec<PluginDescriptor>,
}

/// Describes a single plugin and how to load it.
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginDescriptor {
    /// Unique name for this plugin
    pub name: String,
    /// Either "native_rust" or "wasm_extism"
    #[serde(rename = "type")]
    pub plugin_type: String,
    /// Path to the .rlib/.so/.dylib (for native) or .wasm file
    pub path: PathBuf,
    /// Name of the exported function that creates the plugin instance (for native only)
    /// Typically "_kargo_plugin_create"
    #[serde(default = "default_entry_symbol")]
    pub entry_symbol: String,
}

/// Default entry symbol name for native plugins
fn default_entry_symbol() -> String {
    "_kargo_plugin_create".to_string()
}

/// Manages the discovery, loading, and access to plugins.
pub struct PluginManager {
    /// Directories to search for plugin manifests
    plugin_dirs: Vec<PathBuf>,
    /// Loaded plugin instances by name
    plugins: HashMap<String, Box<dyn KargoPluginCommand>>,
    /// Loaded native libraries (kept alive to prevent unloading)
    #[allow(dead_code)] // Needed to keep libraries in memory
    native_libraries: Vec<Arc<Library>>,
}

impl PluginManager {
    /// Creates a new PluginManager with the default plugin directories.
    pub fn new() -> Self {
        let mut plugin_dirs = Vec::new();
        
        // Check for user config directory
        if let Some(config_dir) = dirs::config_dir() {
            let user_plugin_dir = config_dir.join("kargo").join("plugins");
            plugin_dirs.push(user_plugin_dir);
        }
        
        // Check for project-local plugin directory
        let local_plugin_dir = PathBuf::from(".kargo/plugins");
        plugin_dirs.push(local_plugin_dir);

        // Add any embedded/bundled plugin directory if applicable
        // This would be a directory shipped with the kargo binary
        
        Self {
            plugin_dirs,
            plugins: HashMap::new(),
            native_libraries: Vec::new(),
        }
    }

    /// Add a custom directory to search for plugins
    pub fn add_plugin_dir(&mut self, dir: PathBuf) {
        self.plugin_dirs.push(dir);
    }

    /// Discover and load all plugins from configured plugin directories
    pub fn discover_and_load_plugins(&mut self) -> Result<()> {
        for dir in &self.plugin_dirs {
            // Skip if directory doesn't exist
            if !dir.exists() || !dir.is_dir() {
                continue;
            }

            // Find all plugin manifests in this directory
            let manifest_files = find_manifest_files(dir)?;
            for manifest_path in manifest_files {
                self.load_plugins_from_manifest(&manifest_path)
                    .with_context(|| format!("Failed to load plugins from manifest: {}", manifest_path.display()))?;
            }
        }

        Ok(())
    }

    /// Load specific plugins from a manifest file
    pub fn load_plugins_from_manifest(&mut self, manifest_path: &Path) -> Result<()> {
        let manifest_content = fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read manifest file: {}", manifest_path.display()))?;
        
        let manifest: PluginManifest = toml::from_str(&manifest_content)
            .with_context(|| format!("Failed to parse manifest file: {}", manifest_path.display()))?;
        
        // Use the manifest's directory as the base for relative paths
        let base_dir = manifest_path.parent().unwrap_or(Path::new("."));
        
        for plugin_desc in manifest.plugins {
            // Resolve relative paths against the manifest directory
            let plugin_path = if plugin_desc.path.is_relative() {
                base_dir.join(&plugin_desc.path)
            } else {
                plugin_desc.path
            };

            match plugin_desc.plugin_type.as_str() {
                "native_rust" => {
                    self.load_native_plugin(&plugin_desc.name, &plugin_path, &plugin_desc.entry_symbol)?;
                },
                "wasm_extism" => {
                    self.load_wasm_plugin(&plugin_desc.name, &plugin_path)?;
                },
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported plugin type '{}' for plugin '{}'", 
                        plugin_desc.plugin_type, 
                        plugin_desc.name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Load a native Rust plugin from a dynamic library
    pub fn load_native_plugin(&mut self, name: &str, path: &Path, entry_symbol: &str) -> Result<()> {
        // Load the dynamic library
        let library = Arc::new(unsafe { Library::new(path).with_context(|| format!("Failed to load library: {}", path.display()))? });
        
        // Get the plugin creation function
        let create_fn: Symbol<KargoPluginCreateFn> = unsafe {
            library.get(entry_symbol.as_bytes())
                .with_context(|| format!("Failed to find entry symbol '{}' in library: {}", entry_symbol, path.display()))?
        };
        
        // Call the function to create the plugin instance
        let plugin = create_fn();
        
        // Store the plugin and keep the library alive
        self.plugins.insert(name.to_string(), plugin);
        self.native_libraries.push(library);
        
        Ok(())
    }

    /// Load a WASM plugin using Extism
    pub fn load_wasm_plugin(&mut self, name: &str, path: &Path) -> Result<()> {
        // Create a wrapper for the WASM plugin
        let wasm_plugin = WasmPluginAdapter::new(path)
            .with_context(|| format!("Failed to create WASM plugin adapter for: {}", path.display()))?;
        
        // Store the plugin
        self.plugins.insert(name.to_string(), Box::new(wasm_plugin));
        
        Ok(())
    }

    /// Register a plugin manually (useful for built-in plugins)
    pub fn register_plugin(&mut self, name: &str, plugin: Box<dyn KargoPluginCommand>) {
        self.plugins.insert(name.to_string(), plugin);
    }

    /// Get a reference to a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn KargoPluginCommand>> {
        self.plugins.get(name)
    }

    /// Get all registered plugins
    pub fn get_all_plugins(&self) -> Vec<(&String, &Box<dyn KargoPluginCommand>)> {
        self.plugins.iter().collect()
    }
}

/// Find all plugin manifest files (plugin_manifest.toml) in a directory
fn find_manifest_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut manifest_files = Vec::new();
    
    if !dir.exists() || !dir.is_dir() {
        return Ok(manifest_files);
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recurse one level into subdirectories
            let sub_manifests = find_manifest_files(&path)?;
            manifest_files.extend(sub_manifests);
        } else if path.is_file() && path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "plugin_manifest.toml")
            .unwrap_or(false) {
            manifest_files.push(path);
        }
    }
    
    Ok(manifest_files)
}