// The plugins module provides functionality for loading, managing and executing plugins
// for the kargo CLI tool. This includes both native Rust library plugins and WASM plugins
// via the Extism framework.

mod host_functions;
pub mod manager;
mod trait_scanner;
mod wasm_adapter;
