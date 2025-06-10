use anyhow::Result;
use extism::{CurrentPlugin, Function, HostFunction, Val};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{mpsc, oneshot};

// Task ID counter for generating unique task IDs
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Request types that can be sent to the host function handler
#[derive(Debug)]
pub enum HostFunctionRequest {
    ReadFile {
        path: PathBuf,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    SpawnTask {
        task_id: u64,
        task_name: String,
        params: String,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    PollTask {
        task_id: u64,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    LogMessage {
        level: String,
        message: String,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    GetEnvVar {
        name: String,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
}