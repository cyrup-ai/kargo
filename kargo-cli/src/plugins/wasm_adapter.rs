use anyhow::{Context, Result};
use clap::Command;
use extism::{Function, Plugin, Val, Wasm};
use kargo_plugin_api::{KargoPluginCommand, KargoPluginExecuteFuture, PluginExecutionContext};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

use super::host_functions::{register_host_functions, HostFunctionRequest, HostFunctionResponse};

/// Adapts an Extism WASM plugin to implement the KargoPluginCommand trait.
pub struct WasmPluginAdapter {
    /// The Extism plugin instance, wrapped in an Arc for safe sharing.
    plugin: Arc<Plugin>,
    
    /// Sender for host function requests.
    host_fn_tx: mpsc::Sender<HostFunctionRequest>,
}

impl WasmPluginAdapter {
    /// Creates a new WasmPluginAdapter from a WASM file.
    pub fn new(wasm_path: &Path) -> Result<Self> {
        // Create a channel for host function communication
        let (host_fn_tx, host_fn_rx) = mpsc::channel(32);
        
        // Load the WASM module
        let wasm = Wasm::file(wasm_path)
            .with_context(|| format!("Failed to load WASM file: {}", wasm_path.display()))?;
        
        // Register host functions
        let functions = register_host_functions(host_fn_tx.clone());
        
        // Create the Extism plugin instance with registered host functions
        let plugin = Arc::new(Plugin::new(
            &[wasm], 
            functions, 
            false // Allow wasi
        ).with_context(|| format!("Failed to create Extism plugin instance from: {}", wasm_path.display()))?);
        
        // Spawn the host function handler task
        tokio::spawn({
            let plugin = Arc::clone(&plugin);
            async move {
                let _ = handle_host_function_requests(plugin, host_fn_rx).await;
            }
        });
        
        Ok(Self {
            plugin,
            host_fn_tx,
        })
    }
    
    /// Calls a function in the WASM plugin and returns the result as a JSON string.
    fn call_plugin_function(&self, function: &str, input: &str) -> Result<String> {
        let input_bytes = input.as_bytes();
        let result = self.plugin.call(function, input_bytes)
            .with_context(|| format!("Failed to call WASM plugin function: {}", function))?;
        
        Ok(String::from_utf8(result)
            .with_context(|| "WASM plugin returned invalid UTF-8 data")?)
    }
}

impl KargoPluginCommand for WasmPluginAdapter {
    fn clap_command(&self) -> Command {
        match self.call_plugin_function("_kargo_plugin_get_command_spec_json", "{}") {
            Ok(json) => {
                // Try to deserialize the JSON into a clap::Command
                match serde_json::from_str::<Command>(&json) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        // If deserialization fails, create a fallback command
                        let plugin_name = json.chars().take(20).collect::<String>();
                        eprintln!("Failed to parse command spec from WASM plugin [{}]: {}", plugin_name, e);
                        Command::new("wasm-plugin-error")
                            .about("Error loading WASM plugin")
                    }
                }
            },
            Err(e) => {
                // If function call fails, create a fallback command
                eprintln!("Failed to call get_command_spec function in WASM plugin: {}", e);
                Command::new("wasm-plugin-error")
                    .about("Error calling WASM plugin")
            }
        }
    }

    fn execute(&self, context: PluginExecutionContext) -> KargoPluginExecuteFuture {
        // Capture references for the async block
        let plugin = Arc::clone(&self.plugin);
        let host_fn_tx = self.host_fn_tx.clone();
        
        // Create a future that will be executed by the caller
        Box::pin(async move {
            // Serialize the context to JSON to pass to the WASM plugin
            let context_json = serde_json::to_string(&context.matched_args)
                .context("Failed to serialize plugin execution context")?;
            
            // Call the WASM plugin's execute function
            let result = {
                let result_bytes = plugin.call("_kargo_plugin_execute", context_json.as_bytes())
                    .context("Failed to call WASM plugin execute function")?;
                
                String::from_utf8(result_bytes)
                    .context("WASM plugin returned invalid UTF-8 data")?
            };
            
            // Handle any pending host function requests
            // (These might continue to come in during execution)
            
            // Parse the result - could be structured as a JSON response
            // with success/error information
            if result.contains("error") {
                // Example simple parsing, in reality would need proper JSON parsing
                Err(anyhow::anyhow!("WASM plugin execution failed: {}", result))
            } else {
                Ok(())
            }
        })
    }
}

/// Background task that handles host function requests from the WASM plugin.
/// 
/// This function runs in a separate Tokio task and processes requests from the
/// host_fn_rx channel, executing the appropriate host functions and sending
/// responses back to the WASM plugin.
async fn handle_host_function_requests(
    plugin: Arc<Plugin>,
    mut host_fn_rx: mpsc::Receiver<HostFunctionRequest>,
) -> Result<()> {
    while let Some(req) = host_fn_rx.recv().await {
        match req {
            HostFunctionRequest::ReadFile { path, reply } => {
                // Example of handling a host function request
                let result = tokio::fs::read(path).await;
                let _ = reply.send(match result {
                    Ok(data) => HostFunctionResponse::Data(data),
                    Err(e) => HostFunctionResponse::Error(e.to_string()),
                });
            },
            HostFunctionRequest::WriteFile { path, data, reply } => {
                let result = tokio::fs::write(path, data).await;
                let _ = reply.send(match result {
                    Ok(_) => HostFunctionResponse::Success,
                    Err(e) => HostFunctionResponse::Error(e.to_string()),
                });
            },
            HostFunctionRequest::SpawnTask { task_id, task_name, params, reply } => {
                // Handle task spawning based on task_name
                // This would dispatch to different async functions based on the task name
                let _ = reply.send(HostFunctionResponse::Success);
                
                // Store task state for later polling
                // (Would need a task registry to track task state)
            },
            // Add more request types as needed
        }
    }
    
    Ok(())
}