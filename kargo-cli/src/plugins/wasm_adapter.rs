use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use extism::{Manifest, Plugin, Wasm};
use tokio::sync::mpsc;

use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};

use super::host_functions::{HostFunctionRequest, handle_requests, register_host_functions};

pub struct WasmPluginAdapter {
    plugin: Arc<Mutex<Plugin>>,
    _sender: mpsc::Sender<HostFunctionRequest>,
}

impl WasmPluginAdapter {
    pub fn new(file: &Path) -> Result<Self> {
        let (tx, rx) = mpsc::channel(32);

        // Create manifest with the WASM file
        let wasm = Wasm::file(file);
        let manifest = Manifest::new([wasm]);

        // Build plugin with host functions
        let plugin = register_host_functions(tx.clone(), manifest)
            .with_context(|| format!("Failed to create Extism plugin from: {}", file.display()))?;

        let plugin = Arc::new(Mutex::new(plugin));
        let plugin_clone = Arc::clone(&plugin);
        tokio::spawn(handle_requests(plugin_clone, rx));
        Ok(Self {
            plugin,
            _sender: tx,
        })
    }

    fn json_call(&self, func: &str, input: &str) -> Result<String> {
        let mut plugin = self
            .plugin
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock plugin mutex: {}", e))?;
        let output = plugin
            .call::<&str, String>(func, input)
            .with_context(|| format!("Failed to call WASM function: {}", func))?;
        Ok(output)
    }
}

impl PluginCommand for WasmPluginAdapter {
    fn clap(&self) -> clap::Command {
        match self.json_call("_kargo_plugin_get_command_spec_json", "{}") {
            Ok(json) => {
                // Parse the JSON into command name and about
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(val) => {
                        let name = match val.get("name").and_then(|v| v.as_str()) {
                            Some(n) => n.to_string(),
                            None => {
                                eprintln!("Plugin command missing 'name' field");
                                return clap::Command::new("wasm-missing-name");
                            }
                        };
                        let about = val
                            .get("about")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let mut cmd = clap::Command::new(name);
                        if let Some(about) = about {
                            cmd = cmd.about(about);
                        }
                        cmd
                    }
                    Err(e) => {
                        eprintln!("Failed to parse command spec: {}", e);
                        clap::Command::new("wasm-bad-spec")
                    }
                }
            }
            Err(e) => {
                eprintln!("{e}");
                clap::Command::new("wasm-error")
            }
        }
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        let plugin = Arc::clone(&self.plugin);
        Box::pin(async move {
            let input = serde_json::to_string(&ctx.matched_args)?;
            let mut plugin = plugin
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock plugin mutex: {}", e))?;
            let output = plugin.call::<&str, String>("_kargo_plugin_execute", &input)?;
            println!("{}", output);
            Ok(())
        })
    }
}
