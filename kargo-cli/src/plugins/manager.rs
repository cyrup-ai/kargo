use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use log::info;
use std::process::Command;

use kargo_plugin_api::{CreateFn, PluginCommand};

use super::{trait_scanner, wasm_adapter::WasmPluginAdapter};
use crate::builtin::plugin::artifact::find_existing_lib;

pub struct PluginManager {
    search_paths: Vec<PathBuf>,
    plugins: HashMap<String, Box<dyn PluginCommand>>,
    _native_libs: Vec<Arc<Library>>, // keep libs alive
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    #[must_use] 
    pub fn new() -> Self {
        // 1) optional env override
        use std::env;
        let mut sp: Vec<PathBuf> = env::var_os("KARGO_PLUGIN_PATH")
            .map(|v| env::split_paths(&v).collect())
            .unwrap_or_default();

        // 2) Auto-discover workspace siblings
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let workspace_root = PathBuf::from(manifest_dir)
                .parent()
                .map(std::path::Path::to_path_buf);
            if let Some(root) = workspace_root {
                // Look for plugins in plugins/native directory
                let native_plugins_dir = root.join("plugins").join("native");
                if native_plugins_dir.is_dir() {
                    info!(
                        "Scanning native plugins in {}",
                        native_plugins_dir.display()
                    );
                    if let Ok(entries) = fs::read_dir(&native_plugins_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() && path.join("Cargo.toml").exists()
                                && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                info!("Discovered native plugin candidate: {name}");
                                sp.push(path);
                            }
                        }
                    }
                }

                // Look for plugins in plugins/wasm directory
                let wasm_plugins_dir = root.join("plugins").join("wasm");
                if wasm_plugins_dir.is_dir()
                    && let Ok(entries) = fs::read_dir(&wasm_plugins_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() && path.join("Cargo.toml").exists() {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    info!("Discovered WASM plugin candidate: {name}");
                                    sp.push(path);
                                }
                            } else if path.extension().and_then(|e| e.to_str()) == Some("wasm")
                                && let Some(parent) = path.parent() {
                                if let Some(name) = parent.file_name().and_then(|n| n.to_str())
                                {
                                    info!(
                                        "Discovered standalone WASM module for plugin: {name}"
                                    );
                                }
                                sp.push(parent.to_path_buf());
                            }
                        }
                    }
            }
        } else {
            // Try compile-time workspace root
            if let Some(workspace_root) = option_env!("KARGO_WORKSPACE_ROOT") {
                // When installed, use the compile-time workspace root
                let workspace_path = PathBuf::from(workspace_root);

                // Look for plugins in plugins/native directory
                let native_plugins_dir = workspace_path.join("plugins").join("native");
                if native_plugins_dir.is_dir()
                    && let Ok(entries) = fs::read_dir(&native_plugins_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() && path.join("Cargo.toml").exists() {
                                sp.push(path);
                            }
                        }
                    }

                // Look for plugins in plugins/wasm directory
                let wasm_plugins_dir = workspace_path.join("plugins").join("wasm");
                if wasm_plugins_dir.is_dir()
                    && let Ok(entries) = fs::read_dir(&wasm_plugins_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() && path.join("Cargo.toml").exists() {
                                sp.push(path);
                            }
                        }
                    }
            }
        }

        // 3) Default search paths
        if let Some(cfg) = dirs::config_dir() {
            sp.push(cfg.join("kargo").join("plugins"));
        }
        sp.push(PathBuf::from(".kargo/plugins"));

        Self {
            search_paths: sp,
            plugins: HashMap::new(),
            _native_libs: vec![],
        }
    }

    pub fn discover_and_load_plugins(&mut self) -> Result<()> {
        let search_paths = self.search_paths.clone();
        for d in &search_paths {
            if !d.is_dir() {
                continue;
            }

            // Check if this directory itself is a plugin (for workspace siblings)
            if d.join("Cargo.toml").is_file() {
                match self.build_and_load_rust_project(d) {
                    Ok(()) => info!("Successfully loaded plugin from {}", d.display()),
                    Err(e) => info!("Failed to load plugin from {}: {}", d.display(), e),
                }
                continue;
            }

            // Otherwise scan for subdirectories (for .kargo/plugins style)
            for entry in fs::read_dir(d)? {
                let path = entry?.path();
                if path.is_dir() && path.join("Cargo.toml").is_file() {
                    self.build_and_load_rust_project(&path)
                        .with_context(|| format!("Rust plugin {}", path.display()))?;
                } else {
                    match path.extension().and_then(OsStr::to_str) {
                        Some("so" | "dylib" | "dll") => match self.load_native(&path) {
                            Ok(()) => info!("Successfully loaded native plugin: {}", path.display()),
                            Err(e) => {
                                info!("Failed to load native plugin {}: {}", path.display(), e);
                            }
                        },
                        Some("wasm") => match self.load_wasm(&path) {
                            Ok(()) => info!("Successfully loaded WASM plugin: {}", path.display()),
                            Err(e) => info!("Failed to load WASM plugin {}: {}", path.display(), e),
                        },
                        _ => {}
                    }
                }
            }
        }

        for name in self.plugins.keys() {
            info!("Loaded plugin: {name}");
        }

        Ok(())
    }

    #[must_use] 
    pub fn get(&self, name: &str) -> Option<&dyn PluginCommand> {
        self.plugins.get(name).map(|boxed| &**boxed)
    }

    pub fn plugins_iter(&self) -> impl Iterator<Item = (&String, &Box<dyn PluginCommand>)> {
        self.plugins.iter()
    }

    /* -------- raw Rust project -------- */
    fn build_and_load_rust_project(&mut self, dir: &Path) -> Result<()> {
        // First, verify the plugin implements the required traits
        self.verify_plugin_traits(dir)?;

        let needs_build = {
            let artifact = find_existing_lib(dir)?;
            match artifact {
                None => {
                    log::debug!("No artifact found for {}, needs build", dir.display());
                    true
                }
                Some(ref art) => {
                    // Get artifact modification time
                    let art_modified = match fs::metadata(art).and_then(|m| m.modified()) {
                        Ok(time) => Some(time),
                        Err(e) => {
                            log::warn!("Cannot read artifact metadata {}: {}", art.display(), e);
                            None
                        }
                    };

                    // Check Cargo.toml modification time
                    let cargo_toml = dir.join("Cargo.toml");
                    let cargo_modified = match fs::metadata(&cargo_toml).and_then(|m| m.modified()) {
                        Ok(time) => Some(time),
                        Err(e) => {
                            log::warn!("Cannot read Cargo.toml metadata {}: {}", cargo_toml.display(), e);
                            None
                        }
                    };

                    // Recursively check all source files
                    let src_dir = dir.join("src");
                    let src_modified = if src_dir.exists() {
                        jwalk::WalkDir::new(&src_dir)
                            .into_iter()
                            .filter_map(|entry_result| {
                                match entry_result {
                                    Ok(entry) if entry.file_type().is_file() => {
                                        // Get modification time for this file
                                        match entry.metadata() {
                                            Ok(metadata) => match metadata.modified() {
                                                Ok(time) => Some(time),
                                                Err(e) => {
                                                    log::warn!("Cannot read file metadata {}: {}", entry.path().display(), e);
                                                    None
                                                }
                                            },
                                            Err(e) => {
                                                log::warn!("Cannot read file metadata {}: {}", entry.path().display(), e);
                                                None
                                            }
                                        }
                                    }
                                    Ok(_) => None,  // Directory, skip
                                    Err(e) => {
                                        log::warn!("Error walking source directory {}: {}", src_dir.display(), e);
                                        None
                                    }
                                }
                            })
                            .max()  // Find the newest modification time
                    } else {
                        None
                    };

                    // Take the newest of Cargo.toml or any source file
                    let newest_source = [cargo_modified, src_modified]
                        .into_iter()
                        .flatten()
                        .max();

                    // Compare and log decision
                    match newest_source.zip(art_modified) {
                        Some((s, a)) => {
                            let needs = s > a;
                            if needs {
                                log::debug!("Source newer than artifact for {}, needs rebuild", dir.display());
                            } else {
                                log::debug!("Artifact up to date for {}, skipping build", dir.display());
                            }
                            needs
                        }
                        None => {
                            log::debug!("Cannot determine staleness for {}, rebuilding to be safe", dir.display());
                            true
                        }
                    }
                }
            }
        };

        if needs_build {
            let status = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .arg("--lib")
                .arg("--manifest-path")
                .arg(dir.join("Cargo.toml"))
                .status()?;
            if !status.success() {
                anyhow::bail!("cargo build failed for {}", dir.display());
            }
        }

        let lib = find_existing_lib(dir)?
            .ok_or_else(|| anyhow::anyhow!("built lib not found for {}", dir.display()))?;
        self.load_native(&lib)
    }

    /// Verify that the plugin implements the required traits using syn
    fn verify_plugin_traits(&self, dir: &Path) -> Result<()> {
        // Look for lib.rs or main.rs
        let src_dir = dir.join("src");
        let lib_rs = src_dir.join("lib.rs");
        let main_rs = src_dir.join("main.rs");

        let source_file = if lib_rs.exists() {
            lib_rs
        } else if main_rs.exists() {
            main_rs
        } else {
            anyhow::bail!("No lib.rs or main.rs found in {}", src_dir.display());
        };

        // Enforce validation - plugins MUST implement required traits
        trait_scanner::verify_native_plugin(&source_file)
            .with_context(|| format!(
                "Plugin at {} failed trait validation - must implement PluginCommand and export kargo_plugin_create",
                dir.display()
            ))
            .map(|_| ())
    }

    /* -------- existing native lib -------- */
    fn load_native(&mut self, file: &Path) -> Result<()> {
        let lib = unsafe { Library::new(file) }?;
        let arc = Arc::new(lib);
        let ctor: Symbol<CreateFn> = unsafe { arc.get(b"kargo_plugin_create") }?;
        let plugin = ctor();
        self.plugins
            .insert(plugin.clap().get_name().to_owned(), plugin);
        self._native_libs.push(arc);
        Ok(())
    }

    fn load_wasm(&mut self, file: &Path) -> Result<()> {
        let adapt = WasmPluginAdapter::new(file)?;
        self.plugins
            .insert(adapt.clap().get_name().to_owned(), Box::new(adapt));
        Ok(())
    }
}
