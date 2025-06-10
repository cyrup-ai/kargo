// The host module provides the host-side functionality for WASM plugins,
// including host functions that plugins can call and task management.

mod types;
mod functions;
mod tasks;

pub use types::{HostFunctionRequest, HostFunctionResponse};
pub use functions::register_host_functions;
pub use tasks::TaskManager;