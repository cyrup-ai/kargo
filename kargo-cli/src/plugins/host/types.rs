use std::path::PathBuf;
use tokio::sync::oneshot;

/// Request types that can be sent to the host function handler
#[derive(Debug)]
pub enum HostFunctionRequest {
    /// Request to read a file asynchronously
    ReadFile {
        path: PathBuf,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    
    /// Request to write data to a file asynchronously
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    
    /// Request to spawn an async task
    SpawnTask {
        task_id: u64,
        task_name: String,
        params: String,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    
    /// Request to poll the result of a previously spawned task
    PollTask {
        task_id: u64,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    
    /// Request to log a message
    LogMessage {
        level: String,
        message: String,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    
    /// Request to get an environment variable
    GetEnvVar {
        name: String,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
}/// Response types that can be sent back to the plugin
#[derive(Debug)]
pub enum HostFunctionResponse {
    /// Success with no data
    Success,
    
    /// Success with binary data
    Data(Vec<u8>),
    
    /// Success with string data
    Text(String),
    
    /// Error message
    Error(String),
    
    /// Task is still running
    TaskPending,
}