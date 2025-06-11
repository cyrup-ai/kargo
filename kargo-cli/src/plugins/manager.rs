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
            info!("Scanning {}", d.display());
            for entry in fs::read_dir(d)? {
                let path = entry?.path();
                if path.is_dir() && path.join("Cargo.toml").is_file() {
                    self.build_and_load_rust_project(&path)
                        .with_context(|| format!("Rust plugin {}", path.display()))?;
                } else {
                    println!("DEBUG: Found file: {}", path.display());
                    match path.extension().and_then(OsStr::to_str) {
                        Some("so" | "dylib" | "dll") => {
                            println!("DEBUG: Loading native plugin: {}", path.display());
                            match self.load_native(&path) {
                                Ok(_) => println!("DEBUG: Successfully loaded: {}", path.display()),
                                Err(e) => println!("DEBUG: Failed to load: {} - {}", path.display(), e),
                            }
                        },
                        Some("wasm") => {
                            println!("DEBUG: Loading WASM plugin: {}", path.display());
                            match self.load_wasm(&path) {
                                Ok(_) => println!("DEBUG: Successfully loaded: {}", path.display()),
                                Err(e) => println!("DEBUG: Failed to load: {} - {}", path.display(), e),
                            }
                        },
                        _ => {}
                    }
                }
            }
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
                    src_max.zip(art_mod).map(|(s, o)| s > o).unwrap_or(true)
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
    let release = dir.join("target").join("release");
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

    for entry in fs::read_dir(release)? {
        let p = entry?.path();
        if p.extension().and_then(|s| s.to_str()) == Some(ext)
            && p.file_name()
                .and_then(|s| s.to_str())
                .map(|f| f.starts_with(prefix))
                .unwrap_or(false)
        {
            return Ok(Some(p));
        }
    }
    Ok(None)
}
