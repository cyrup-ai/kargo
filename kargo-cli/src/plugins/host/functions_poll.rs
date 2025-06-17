use anyhow::Result;
use extism::{CurrentPlugin, Function, HostFunction, Val};
use tokio::sync::{mpsc, oneshot};

use super::types::{HostFunctionRequest, HostFunctionResponse};

/// Register the poll_kargo_task host function
pub fn register_poll_kargo_task(host_fn_tx: mpsc::Sender<HostFunctionRequest>) -> Function {
    HostFunction::new(
        "poll_kargo_task",
        move |plugin: &mut CurrentPlugin, inputs: &[Val], outputs: &mut [Val]| -> Result<()> {
            // Get the task ID from inputs
            let task_id = inputs[0].value() as u64;
            
            // Create a oneshot channel for the response
            let (reply_tx, reply_rx) = oneshot::channel();
            
            // Send the request to the host function handler
            let _ = host_fn_tx.try_send(HostFunctionRequest::PollTask { 
                task_id,
                reply: reply_tx 
            });
            
            // Wait for the response
            match reply_rx.blocking_recv() {
                Ok(HostFunctionResponse::TaskPending) => {
                    outputs[0].set(0); // Not ready
                },
                Ok(HostFunctionResponse::Data(data)) => {
                    // Task completed successfully with data
                    let ptr = plugin.memory().allocate(&data)?;
                    outputs[0].set(1); // Ready
                    outputs[1].set(0); // Success
                    outputs[2].set(ptr.offset());
                    outputs[3].set(data.len() as i64);
                },
                Ok(HostFunctionResponse::Text(text)) => {
                    // Task completed successfully with text
                    let ptr = plugin.memory().allocate(text.as_bytes())?;
                    outputs[0].set(1); // Ready
                    outputs[1].set(0); // Success
                    outputs[2].set(ptr.offset());
                    outputs[3].set(text.len() as i64);
                },
                Ok(HostFunctionResponse::Error(msg)) => {
                    // Task completed with error
                    let ptr = plugin.memory().allocate(msg.as_bytes())?;
                    outputs[0].set(1); // Ready
                    outputs[1].set(1); // Error
                    outputs[2].set(ptr.offset());
                    outputs[3].set(msg.len() as i64);
                },
                _ => {
                    let msg = "Unknown error in poll_kargo_task";
                    let ptr = plugin.memory().allocate(msg.as_bytes())?;
                    outputs[0].set(1); // Ready
                    outputs[1].set(1); // Error
                    outputs[2].set(ptr.offset());
                    outputs[3].set(msg.len() as i64);
                },
            }
            
            Ok(())
        },
    )
}