use std::path::PathBuf;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
    ScanStarted {
        dirs: Vec<PathBuf>,
    },
    CargoTomlFound {
        path: PathBuf,
    },
    RustScriptFound {
        path: PathBuf,
    },
    WorkspaceFound {
        path: PathBuf,
    },
    DependencyUpdated {
        path: PathBuf,
        from: String,
        to: String,
    },
    CommandStarted {
        command: String,
    },
    CommandFinished {
        command: String,
        success: bool,
    },
    KargoOutputLine {
        line: String,
        is_error: bool,
    },
    KargoCommandStarted {
        subcommand: String,
        args: Vec<String>,
    },
    KargoCommandFinished {
        subcommand: String,
        success: bool,
        summary: String,
    },
    VendorStarted {
        path: PathBuf,
    },
    VendorFinished {
        path: PathBuf,
    },
    Error {
        message: String,
    },
    Info {
        message: String,
    },
    RollbackStarted {
        path: PathBuf,
    },
    RollbackFinished {
        path: PathBuf,
    },
}

#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: Event) {
        let _ = self.tx.send(event);
    }
}

// Add From implementation for broadcast::Receiver<Event>
impl From<broadcast::Receiver<Event>> for EventBus {
    fn from(rx: broadcast::Receiver<Event>) -> Self {
        // We need to create a new channel and forward events from rx to it
        let (tx, _) = broadcast::channel(100);
        let tx_clone = tx.clone();

        // Spawn a task to forward events
        tokio::spawn(async move {
            let mut rx = rx;
            while let Ok(event) = rx.recv().await {
                let _ = tx_clone.send(event);
            }
        });

        Self { tx }
    }
}
