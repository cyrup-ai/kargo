use kargo_plugin_wasm::{
    kargo_wasm_plugin, ArgDefinition, CommandDefinition, ExecutionResult, 
    PluginMetadata, WasmPlugin
};
use serde_json::Value;

pub struct {{plugin_name | pascal_case}}Plugin;

impl WasmPlugin for {{plugin_name | pascal_case}}Plugin {
    fn get_command() -> String {
        let cmd = CommandDefinition {
            name: "{{plugin_name}}".to_string(),
            about: "{{plugin_description}}".to_string(),
            args: vec![
                ArgDefinition {
                    name: "example".to_string(),
                    short: Some('e'),
                    long: Some("example".to_string()),
                    help: "An example argument".to_string(),
                    required: false,
                    takes_value: true,
                },
                // TODO: Add more arguments as needed
            ],
        };
        serde_json::to_string(&cmd).unwrap()
    }
    
    fn execute(args_json: String) -> String {
        let args: Value = match serde_json::from_str(&args_json) {
            Ok(v) => v,
            Err(e) => {
                return serde_json::to_string(&ExecutionResult {
                    success: false,
                    output: None,
                    error: Some(format!("Failed to parse arguments: {}", e)),
                }).unwrap();
            }
        };
        
        // TODO: Implement your plugin logic here
        let output = format!("Hello from {{plugin_name}} WASM plugin! Args: {}", args);
        
        let result = ExecutionResult {
            success: true,
            output: Some(output),
            error: None,
        };
        
        serde_json::to_string(&result).unwrap()
    }
    
    fn get_metadata() -> String {
        let metadata = PluginMetadata {
            name: "{{plugin_name}}".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "{{plugin_description}}".to_string(),
            author: "{{author_name}}".to_string(),
            language: "rust".to_string(),
        };
        serde_json::to_string(&metadata).unwrap()
    }
}

// Generate the WASM exports
kargo_wasm_plugin!({{plugin_name | pascal_case}}Plugin);