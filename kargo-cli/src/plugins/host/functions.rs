use anyhow::Result;
use extism::{CurrentPlugin, Function, HostFunction, Val};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{mpsc, oneshot};

use super::types::{HostFunctionRequest, HostFunctionResponse};

/// Register all host functions and return them as a vector
pub fn register_host_functions(host_fn_tx: mpsc::Sender<HostFunctionRequest>) -> Vec<Function> {
    vec![
        register_log_message(host_fn_tx.clone()),
        register_read_file(host_fn_tx.clone()),
        register_write_file(host_fn_tx.clone()),
        register_spawn_kargo_task(host_fn_tx.clone()),
        register_poll_kargo_task(host_fn_tx.clone()),
        register_get_env_var(host_fn_tx),
    ]
}/// Register the log_message host function
fn register_log_message(host_fn_tx: mpsc::Sender<HostFunctionRequest>) -> Function {
    HostFunction::new(
        "log_message",
        move |plugin: &mut CurrentPlugin, inputs: &[Val], outputs: &mut [Val]| -> Result<()> {
            // Get the log level and message from inputs
            let level = plugin.memory().get_string(inputs[0].value())?;
            let message = plugin.memory().get_string(inputs[1].value())?;
            
            // Create a oneshot channel for the response
            let (reply_tx, reply_rx) = oneshot::channel();
            
            // Send the request to the host function handler
            let _ = host_fn_tx.try_send(HostFunctionRequest::LogMessage { 
                level, 
                message, 
                reply: reply_tx 
            });
            
            // Wait for the response
            match reply_rx.blocking_recv() {
                Ok(HostFunctionResponse::Success) => {
                    outputs[0].set(0); // Success
                },
                _ => {
                    outputs[0].set(1); // Error
                },
            }
            
            Ok(())
        },
    )
}/// Register the read_file host function
fn register_read_file(host_fn_tx: mpsc::Sender<HostFunctionRequest>) -> Function {
    HostFunction::new(
        "read_file",
        move |plugin: &mut CurrentPlugin, inputs: &[Val], outputs: &mut [Val]| -> Result<()> {
            // Get the file path from inputs
            let path_str = plugin.memory().get_string(inputs[0].value())?;
            let path = std::path::PathBuf::from(path_str);
            
            // Create a oneshot channel for the response
            let (reply_tx, reply_rx) = oneshot::channel();
            
            // Send the request to the host function handler
            let _ = host_fn_tx.try_send(HostFunctionRequest::ReadFile { 
                path, 
                reply: reply_tx 
            });
            
            // Wait for the response
            match reply_rx.blocking_recv() {
                Ok(HostFunctionResponse::Data(data)) => {
                    // Allocate memory in the plugin for the data
                    let ptr = plugin.memory().allocate(&data)?;
                    outputs[0].set(ptr.offset());
                    outputs[1].set(data.len() as i64);
                    outputs[2].set(0); // Success
                },
                Ok(HostFunctionResponse::Error(msg)) => {
                    let ptr = plugin.memory().allocate(msg.as_bytes())?;
                    outputs[0].set(ptr.offset());
                    outputs[1].set(msg.len() as i64);
                    outputs[2].set(1); // Error
                },
                _ => {
                    let msg = "Unknown error in read_file";
                    let ptr = plugin.memory().allocate(msg.as_bytes())?;
                    outputs[0].set(ptr.offset());
                    outputs[1].set(msg.len() as i64);
                    outputs[2].set(1); // Error
                },
            }
            
            Ok(())
        },
    )
}/// Register the write_file host function
fn register_write_file(host_fn_tx: mpsc::Sender<HostFunctionRequest>) -> Function {
    HostFunction::new(
        "write_file",
        move |plugin: &mut CurrentPlugin, inputs: &[Val], outputs: &mut [Val]| -> Result<()> {
            // Get the file path and data from inputs
            let path_str = plugin.memory().get_string(inputs[0].value())?;
            let path = std::path::PathBuf::from(path_str);
            
            let offset = inputs[1].value();
            let length = inputs[2].value() as usize;
            let data = plugin.memory().get_span(offset, length)?.to_vec();
            
            // Create a oneshot channel for the response
            let (reply_tx, reply_rx) = oneshot::channel();
            
            // Send the request to the host function handler
            let _ = host_fn_tx.try_send(HostFunctionRequest::WriteFile { 
                path,
                data,
                reply: reply_tx 
            });
            
            // Wait for the response
            match reply_rx.blocking_recv() {
                Ok(HostFunctionResponse::Success) => {
                    outputs[0].set(0); // Success
                },
                Ok(HostFunctionResponse::Error(msg)) => {
                    let ptr = plugin.memory().allocate(msg.as_bytes())?;
                    outputs[0].set(1); // Error
                    outputs[1].set(ptr.offset());
                    outputs[2].set(msg.len() as i64);
                },
                _ => {
                    let msg = "Unknown error in write_file";
                    let ptr = plugin.memory().allocate(msg.as_bytes())?;
                    outputs[0].set(1); // Error
                    outputs[1].set(ptr.offset());
                    outputs[2].set(msg.len() as i64);
                },
            }
            
            Ok(())
        },
    )
}/// Task ID counter for generating unique task IDs
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Register the spawn_kargo_task host function
fn register_spawn_kargo_task(host_fn_tx: mpsc::Sender<HostFunctionRequest>) -> Function {
    HostFunction::new(
        "spawn_kargo_task",
        move |plugin: &mut CurrentPlugin, inputs: &[Val], outputs: &mut [Val]| -> Result<()> {
            // Get the task name and parameters from inputs
            let task_name = plugin.memory().get_string(inputs[0].value())?;
            let params = plugin.memory().get_string(inputs[1].value())?;
            
            // Generate a unique task ID
            let task_id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
            
            // Create a oneshot channel for the response
            let (reply_tx, reply_rx) = oneshot::channel();
            
            // Send the request to the host function handler
            let _ = host_fn_tx.try_send(HostFunctionRequest::SpawnTask { 
                task_id,
                task_name,
                params,
                reply: reply_tx 
            });
            
            // Wait for the response
            match reply_rx.blocking_recv() {
                Ok(HostFunctionResponse::Success) => {
                    outputs[0].set(task_id as i64); // Return the task ID
                    outputs[1].set(0); // Success
                },
                Ok(HostFunctionResponse::Error(msg)) => {
                    let ptr = plugin.memory().allocate(msg.as_bytes())?;
                    outputs[0].set(0); // Invalid task ID
                    outputs[1].set(1); // Error
                    outputs[2].set(ptr.offset());
                    outputs[3].set(msg.len() as i64);
                },
                _ => {
                    let msg = "Unknown error in spawn_kargo_task";
                    let ptr = plugin.memory().allocate(msg.as_bytes())?;
                    outputs[0].set(0); // Invalid task ID
                    outputs[1].set(1); // Error
                    outputs[2].set(ptr.offset());
                    outputs[3].set(msg.len() as i64);
                },
            }
            
            Ok(())
        },
    )
}