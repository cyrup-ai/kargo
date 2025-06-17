//! Core data types for the dependency update functionality

use anyhow;
use futures::Stream;
use futures::StreamExt;
use std::path::PathBuf;
use tokio::sync::mpsc;

// Import Future types for SendFuture
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::models::{Dependency, DependencyUpdater};
// Re-export DependencyUpdate from models for public use
pub use crate::models::DependencyUpdate;

/// Represents a pending file write operation
pub struct PendingWrite {
    inner: SendFuture<anyhow::Result<()>>,
}

impl PendingWrite {
    /// Create a new pending write operation
    pub fn new(future: impl Future<Output = anyhow::Result<()>> + Send + 'static) -> Self {
        Self {
            inner: SendFuture(Box::pin(future)),
        }
    }
}

impl Future for PendingWrite {
    type Output = anyhow::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

// A wrapper around a future that ensures it is Send
pub struct SendFuture<T>(pub Pin<Box<dyn Future<Output = T> + Send + 'static>>);

/// Represents a pending dependency update that can be awaited
pub struct PendingDependencyUpdate {
    inner: SendFuture<anyhow::Result<Option<DependencyUpdate>>>,
}

impl PendingDependencyUpdate {
    /// Create a new pending dependency update with the given future
    pub fn new(
        future: impl Future<Output = anyhow::Result<Option<DependencyUpdate>>> + Send + 'static,
    ) -> Self {
        Self {
            inner: SendFuture(Box::pin(future)),
        }
    }
}

impl Future for PendingDependencyUpdate {
    type Output = anyhow::Result<Option<DependencyUpdate>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

/// Represents a stream of dependency updates
pub struct DependencyUpdateStream {
    inner: tokio_stream::wrappers::ReceiverStream<anyhow::Result<DependencyUpdate>>,
}

impl DependencyUpdateStream {
    /// Create a new dependency update stream from a receiver
    pub fn new(rx: mpsc::Receiver<anyhow::Result<DependencyUpdate>>) -> Self {
        Self {
            inner: tokio_stream::wrappers::ReceiverStream::new(rx),
        }
    }
}

impl Stream for DependencyUpdateStream {
    type Item = anyhow::Result<DependencyUpdate>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Represents a batch operation to update multiple dependencies
/// Returns a stream of dependency updates
pub struct BatchUpdateOperation {
    inner: mpsc::Receiver<anyhow::Result<DependencyUpdate>>,
}

impl BatchUpdateOperation {
    /// Create a new batch update operation that will stream updates
    pub fn new(dependencies: Vec<Dependency>, updater: &impl DependencyUpdater) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let deps = dependencies.clone();
        let updater = updater.clone();

        // Spawn a task to process updates and send them to the channel
        tokio::spawn(async move {
            for dep in deps {
                // Get update for each dependency
                let update_result = updater.update(&dep).await;

                match update_result {
                    Ok(Some(update)) => {
                        // If we got an update, send it through the channel
                        if tx.send(Ok(update)).await.is_err() {
                            // Channel closed, receiver dropped
                            break;
                        }
                    }
                    Ok(None) => {
                        // No update needed, continue to next dependency
                        continue;
                    }
                    Err(e) => {
                        // Error updating, send the error
                        if tx.send(Err(e)).await.is_err() {
                            // Channel closed, receiver dropped
                            break;
                        }
                    }
                }
            }
            // tx is dropped here, which will close the stream
        });

        Self { inner: rx }
    }

    /// Convert this batch operation into a stream
    pub fn into_stream(self) -> impl Stream<Item = anyhow::Result<DependencyUpdate>> {
        tokio_stream::wrappers::ReceiverStream::new(self.inner)
    }

    /// Collect all results into a vector
    pub async fn collect(self) -> anyhow::Result<Vec<DependencyUpdate>> {
        let mut results = Vec::new();
        let mut stream = tokio_stream::wrappers::ReceiverStream::new(self.inner);

        while let Some(result) = stream.next().await {
            match result {
                Ok(update) => results.push(update),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }
}

// Manual debug implementation since the inner future doesn't implement Debug
impl<T> std::fmt::Debug for SendFuture<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendFuture")
            .field("_inner", &"<opaque future>")
            .finish()
    }
}

impl<T> Future for SendFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

/// Represents a type of crate that can be updated
#[derive(Debug, Clone)]
pub enum CrateType {
    /// Standard crate with Cargo.toml
    Standard,
    /// Crate within a workspace
    Workspace,
    /// Rust script with embedded cargo
    RustScript,
    /// Unknown crate type
    Unknown,
}

/// Options for configuring how dependencies are updated
#[derive(Debug, Clone)]
pub struct UpdateOptions {
    /// Whether to update workspace dependencies
    pub update_workspace: bool,
    /// Whether to update to compatible versions only (respects semver)
    pub compatible_only: bool,
}

impl Default for UpdateOptions {
    fn default() -> Self {
        Self {
            update_workspace: true,
            compatible_only: true,
        }
    }
}

// DependencyUpdate type is imported from models.rs

/// Result of an update operation on a single file
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// Path to the file that was updated
    pub path: PathBuf,
    /// Dependencies that were updated
    pub updates: Vec<DependencyUpdate>,
    /// Type of crate
    pub crate_type: CrateType,
    /// Any errors that occurred during the update
    pub error: Option<String>,
}

/// A session for tracking dependency update operations
#[derive(Debug)]
pub struct UpdateSession {
    receiver: mpsc::Receiver<UpdateResult>,
}

impl UpdateSession {
    /// Create a new update session with the given channel receiver
    pub fn new(receiver: mpsc::Receiver<UpdateResult>) -> Self {
        Self { receiver }
    }

    /// Returns a collector that can collect all results into a vector
    pub fn collect_results(self) -> UpdateCollector {
        UpdateCollector {
            receiver: self.receiver,
        }
    }

    /// Returns a processor for handling results as they arrive with a callback function
    pub fn process_with<F>(self, callback: F) -> UpdateProcessor<F>
    where
        F: FnMut(UpdateResult) + Send + 'static,
    {
        UpdateProcessor {
            receiver: self.receiver,
            callback,
        }
    }

    /// Returns a watcher for getting the next result
    pub fn watch(&mut self) -> UpdateWatcher<'_> {
        UpdateWatcher {
            receiver: &mut self.receiver,
        }
    }

    /// Convert this session into a Stream
    ///
    /// Note: Requires tokio_stream, which is a standard dependency.
    pub fn into_stream(self) -> tokio_stream::wrappers::ReceiverStream<UpdateResult> {
        use tokio_stream::wrappers::ReceiverStream;
        ReceiverStream::new(self.receiver)
    }
}

/// Collects all update results
#[derive(Debug)]
pub struct UpdateCollector {
    receiver: mpsc::Receiver<UpdateResult>,
}

impl UpdateCollector {
    /// Collects all results into a vector
    pub fn get_all_results(mut self) -> SendFuture<Vec<UpdateResult>> {
        SendFuture(Box::pin(async move {
            let mut results = Vec::new();
            while let Some(result) = self.receiver.recv().await {
                results.push(result);
            }
            results
        }))
    }
}

/// Processes update results with a callback function
#[derive(Debug)]
pub struct UpdateProcessor<F>
where
    F: FnMut(UpdateResult) + Send + 'static,
{
    receiver: mpsc::Receiver<UpdateResult>,
    callback: F,
}

impl<F> UpdateProcessor<F>
where
    F: FnMut(UpdateResult) + Send + 'static,
{
    /// Start processing results with the callback
    pub fn start(mut self) -> SendFuture<()> {
        SendFuture(Box::pin(async move {
            while let Some(result) = self.receiver.recv().await {
                (self.callback)(result);
            }
        }))
    }
}

/// Watches for update results one at a time
#[derive(Debug)]
pub struct UpdateWatcher<'a> {
    receiver: &'a mut mpsc::Receiver<UpdateResult>,
}

impl<'a> UpdateWatcher<'a> {
    /// Gets the next result
    pub fn next(self) -> impl Future<Output = Option<UpdateResult>> + Send + 'a {
        async move { self.receiver.recv().await }
    }
}

/// Options for the version up2date
#[derive(Debug, Clone)]
pub struct VersionUpdaterOptions {
    /// Whether to update workspace dependencies
    pub update_workspace: bool,
    /// Whether to update compatible versions only (respects semver)
    pub compatible_only: bool,
}

impl Default for VersionUpdaterOptions {
    fn default() -> Self {
        Self {
            update_workspace: true,
            compatible_only: true,
        }
    }
}

/// Handles updating dependency versions in Cargo.toml files
#[derive(Debug, Clone)]
pub struct VersionUpdater {
    /// Options that control the update behavior
    pub options: VersionUpdaterOptions,
}

impl VersionUpdater {
    /// Creates a new VersionUpdater with the default options
    pub fn new() -> Self {
        Self {
            options: VersionUpdaterOptions::default(),
        }
    }

    /// Creates a new VersionUpdater with custom options
    pub fn with_options(options: VersionUpdaterOptions) -> Self {
        Self { options }
    }
}
