use serde::{Deserialize, Serialize};

/// WASM plugin interface for kargo
///
/// This interface is designed to be implemented by:
/// - Rust (compiled to WASM)
/// - Python (via py_mini_racer or similar)
/// - Node/TypeScript (native WASM support)
/// - Go (via TinyGo)
/// - Any language that compiles to WASM
///
/// Plugins run in a sandboxed environment with:
/// - Memory isolation
/// - Controlled access to host functions
/// - No direct OS access
/// - Safe concurrent execution
pub trait WasmPlugin {
    /// Get command metadata as JSON
    fn get_command() -> String;

    /// Execute the plugin with JSON arguments
    fn execute(args_json: String) -> String;

    /// Get plugin metadata as JSON
    fn get_metadata() -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub name: String,
    pub about: String,
    pub args: Vec<ArgDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgDefinition {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub help: String,
    pub required: bool,
    pub takes_value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub language: String, // "rust", "python", "typescript", "go", etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Helper for Rust WASM plugins
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! kargo_wasm_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn get_command() -> *mut u8 {
            let cmd = <$plugin_type>::get_command();
            let bytes = cmd.into_bytes();
            let len = bytes.len();
            let ptr = bytes.as_ptr();
            std::mem::forget(bytes);
            ptr as *mut u8
        }

        #[no_mangle]
        pub extern "C" fn execute(args_ptr: *const u8, args_len: usize) -> *mut u8 {
            let args = unsafe {
                let slice = std::slice::from_raw_parts(args_ptr, args_len);
                String::from_utf8_unchecked(slice.to_vec())
            };
            let result = <$plugin_type>::execute(args);
            let bytes = result.into_bytes();
            let len = bytes.len();
            let ptr = bytes.as_ptr();
            std::mem::forget(bytes);
            ptr as *mut u8
        }

        #[no_mangle]
        pub extern "C" fn get_metadata() -> *mut u8 {
            let metadata = <$plugin_type>::get_metadata();
            let bytes = metadata.into_bytes();
            let len = bytes.len();
            let ptr = bytes.as_ptr();
            std::mem::forget(bytes);
            ptr as *mut u8
        }
    };
}
