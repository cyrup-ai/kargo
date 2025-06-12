use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use extism::*;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum HostFunctionRequest {
    ReadFile {
        path: PathBuf,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
    Log {
        msg: String,
        reply: oneshot::Sender<HostFunctionResponse>,
    },
}

#[derive(Debug)]
pub enum HostFunctionResponse {
    Text(String),
    Ok,
    Error(String),
}

// Host function for logging
host_fn!(log_fn(user_data: mpsc::Sender<HostFunctionRequest>; msg: String) {
    let tx = user_data.get()?;
    let tx = match tx.lock() {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to lock tx mutex: {}", e);
            return Ok(());
        }
    };
    let (sx, rx) = oneshot::channel();
    let _ = tx.blocking_send(HostFunctionRequest::Log{msg, reply:sx});
    let _ = rx.blocking_recv();
    Ok(())
});

// Host function for reading files
host_fn!(read_file_fn(user_data: mpsc::Sender<HostFunctionRequest>; path: String) -> String {
    let tx = user_data.get()?;
    let tx = match tx.lock() {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to lock tx mutex: {}", e);
            return Err(Error::msg(format!("Failed to lock tx mutex: {}", e)));
        }
    };
    let (sx, rx) = oneshot::channel();
    let _ = tx.blocking_send(HostFunctionRequest::ReadFile{
        path: PathBuf::from(path),
        reply: sx
    });
    match rx.blocking_recv() {
        Ok(HostFunctionResponse::Text(t)) => Ok(t),
        Ok(HostFunctionResponse::Error(e)) => Err(Error::msg(e)),
        _ => Err(Error::msg("read_file failed")),
    }
});

pub fn register_host_functions(
    tx: mpsc::Sender<HostFunctionRequest>,
    manifest: Manifest,
) -> Result<Plugin> {
    let tx_log = UserData::new(tx.clone());
    let tx_read = UserData::new(tx);

    PluginBuilder::new(manifest)
        .with_wasi(true)
        .with_function(
            "log",
            [ValType::I64], // string pointer
            [],             // no return
            tx_log,
            log_fn,
        )
        .with_function(
            "read_file",
            [ValType::I64], // path string pointer
            [ValType::I64], // returns string pointer
            tx_read,
            read_file_fn,
        )
        .build()
}

pub async fn handle_requests(
    _: Arc<std::sync::Mutex<Plugin>>,
    mut rx: mpsc::Receiver<HostFunctionRequest>,
) -> Result<()> {
    while let Some(req) = rx.recv().await {
        match req {
            HostFunctionRequest::Log { msg, reply } => {
                println!("[wasm] {msg}");
                let _ = reply.send(HostFunctionResponse::Ok);
            }
            HostFunctionRequest::ReadFile { path, reply } => {
                let res = tokio::fs::read_to_string(&path).await;
                let _ = reply.send(match res {
                    Ok(t) => HostFunctionResponse::Text(t),
                    Err(e) => HostFunctionResponse::Error(e.to_string()),
                });
            }
        }
    }
    Ok(())
}
