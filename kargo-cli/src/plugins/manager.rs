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

use super::wasm_adapter::WasmPluginAdapter;

pub struct PluginManager {
    search_paths: Vec<PathBuf>,
    plugins: HashMap<String, Box<dyn PluginCommand>>,
    _native_libs: Vec<Arc<Library>>, // keep libs alive
}

impl PluginManager {
    pub fn new() -> Self {
        // 1) optional env override
        use std::env;
        let mut sp = env::var_os("KARGO_PLUGIN_PATH")
            .map(|v| env::split_paths(&v).collect())
            .unwrap_or_else(Vec::new);

        // 2) In development, auto-discover workspace siblings
        if sp.is_empty() && cfg!(debug_assertions) {
            if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
                let workspace_root = PathBuf::from(manifest_dir).parent().map(|p| p.to_path_buf());
                if let Some(root) = workspace_root {
                    info!("Development mode: discovering workspace plugins in {}", root.display());
                    // Add all directories with Cargo.toml as potential plugin paths
                    if let Ok(entries) = fs::read_dir(&root) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() && path.join("Cargo.toml").exists() {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    // Skip self
                                    if name == "kargo-cli" {
                                        continue;
                                    }
                                    info!("Found potential plugin: {}", name);
                                    sp.push(path);
                                }
                            }
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
                info!("Loading plugin project: {}", d.display());
                match self.build_and_load_rust_project(&d) {
                    Ok(_) => info!("Successfully loaded plugin from {}", d.display()),
                    Err(e) => info!("Failed to load plugin from {}: {}", d.display(), e),
                }
                continue;
            }
            
            // Otherwise scan for subdirectories (for .kargo/plugins style)
            info!("Scanning {}", d.display());
            for entry in fs::read_dir(d)? {
                let path = entry?.path();
                if path.is_dir() && path.join("Cargo.toml").is_file() {
                    self.build_and_load_rust_project(&path)
                        .with_context(|| format!("Rust plugin {}", path.display()))?;
                } else {
                    match path.extension().and_then(OsStr::to_str) {
                        Some("so" | "dylib" | "dll") => {
                            match self.load_native(&path) {
                                Ok(_) => info!("Successfully loaded native plugin: {}", path.display()),
                                Err(e) => info!("Failed to load native plugin {}: {}", path.display(), e),
                            }
                        },
                        Some("wasm") => {
                            match self.load_wasm(&path) {
                                Ok(_) => info!("Successfully loaded WASM plugin: {}", path.display()),
                                Err(e) => info!("Failed to load WASM plugin {}: {}", path.display(), e),
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        
        info!("Total plugins loaded: {}", self.plugins.len());
        for (name, _) in &self.plugins {
            info!("  - {}", name);
        }
        
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn PluginCommand>> {
        self.plugins.get(name)
    }

    pub fn plugins_iter(&self) -> impl Iterator<Item = (&String, &Box<dyn PluginCommand>)> {
        self.plugins.iter()
    }

    /* -------- raw Rust project -------- */
    fn build_and_load_rust_project(&mut self, dir: &Path) -> Result<()> {
        info!("Compiling plugin at {}", dir.display());

        let needs_build = {
            let artifact = find_existing_lib(dir)?;
            match artifact {
                None => true,
                Some(ref art) => {
                    let src_max = fs::read_dir(dir)?
                        .filter_map(|e| e.ok())
                        .map(|e| e.metadata().and_then(|m| m.modified()))
                        .flatten()
                        .max();
                    let art_mod = fs::metadata(art).and_then(|m| m.modified()).ok();
                    match src_max.zip(art_mod) {
                        Some((s, o)) => s > o,
                        None => true,
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

/* ---------- helper: locate compiled library ---------- */
fn find_existing_lib(dir: &Path) -> Result<Option<PathBuf>> {
    // First try the local target directory
    let mut release = dir.join("target").join("release");
    
    // If not found, try the workspace target directory
    if !release.is_dir() {
        // Walk up to find workspace root (where Cargo.lock exists)
        let mut workspace_root = dir.to_path_buf();
        while !workspace_root.join("Cargo.lock").exists() && workspace_root.parent().is_some() {
            workspace_root = workspace_root
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Workspace root has no parent directory"))?
                .to_path_buf();
        }
        release = workspace_root.join("target").join("release");
    }
    
    if !release.is_dir() {
        return Ok(None);
    }

    let (prefix, ext) = if cfg!(windows) {
        ("", "dll")
    } else if cfg!(target_os = "macos") {
        ("lib", "dylib")
    } else {
        ("lib", "so")
    };

    // Get the crate name from Cargo.toml
    let cargo_toml = dir.join("Cargo.toml");
    let crate_name = if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        // Simple extraction of lib.name or package.name
        if let Some(lib_name) = content.lines()
            .skip_while(|l| !l.starts_with("[lib]"))
            .skip(1)
            .find(|l| l.trim_start().starts_with("name"))
            .and_then(|l| l.split('=').nth(1))
            .map(|s| s.trim().trim_matches('"'))
        {
            lib_name.to_string()
        } else if let Some(pkg_name) = content.lines()
            .find(|l| l.trim_start().starts_with("name") && !l.contains('['))
            .and_then(|l| l.split('=').nth(1))
            .map(|s| s.trim().trim_matches('"'))
        {
            pkg_name.replace('-', "_")
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    // Look for the specific library file
    let lib_name = format!("{}{}.{}", prefix, crate_name, ext);
    let lib_path = release.join(&lib_name);
    
    if lib_path.exists() {
        Ok(Some(lib_path))
    } else {
        Ok(None)
    }
}
