use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Result, Context};
use tokio::sync::{oneshot, mpsc};
use tracing::{debug, error, info};

use super::types::HostFunctionResponse;

/// Generator for unique task IDs
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Represents a task that can be run asynchronously
pub trait Task: Send + Sync {
    /// Run the task and return a result
    fn run(&self) -> Result<Vec<u8>>;
}

/// Status of an asynchronous task
enum TaskStatus {
    /// Task is running
    Running,
    /// Task completed successfully
    Completed(Vec<u8>),
    /// Task failed with an error
    Failed(String),
}/// Manages asynchronous tasks spawned by WASM plugins
pub struct TaskManager {
    /// Map of task ID to task status
    tasks: Arc<Mutex<HashMap<u64, TaskStatus>>>,
    /// Task registry for creating task instances from task names
    task_registry: HashMap<String, Box<dyn Fn(String) -> Result<Box<dyn Task>> + Send + Sync>>,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            task_registry: HashMap::new(),
        }
    }
    
    /// Register a task factory function for a specific task name
    pub fn register_task<F>(&mut self, name: &str, factory: F) 
    where 
        F: Fn(String) -> Result<Box<dyn Task>> + Send + Sync + 'static 
    {
        self.task_registry.insert(name.to_string(), Box::new(factory));
    }
    
    /// Spawn a new task with the given name and parameters
    pub fn spawn_task(&self, task_name: &str, params: &str) -> Result<u64> {
        // Get the task factory for this task name
        let factory = self.task_registry.get(task_name)
            .context(format!("Task type not registered: {}", task_name))?;
            
        // Create a task instance
        let task = factory(params.to_string())
            .context(format!("Failed to create task: {}", task_name))?;
            
        // Generate a new task ID
        let task_id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // Store initial task status
        {
            let mut tasks = self.tasks.lock().map_err(|e| anyhow::anyhow!("Failed to lock tasks mutex: {}", e))?;
            tasks.insert(task_id, TaskStatus::Running);
        }
        
        // Clone the tasks Arc for the task closure
        let tasks = Arc::clone(&self.tasks);
        
        // Spawn the task in a new Tokio task
        tokio::spawn(async move {
            let result = task.run();
            
            // Update task status based on result
            match tasks.lock() {
                Ok(mut tasks) => {
                    match result {
                        Ok(data) => {
                            tasks.insert(task_id, TaskStatus::Completed(data));
                        },
                        Err(err) => {
                            tasks.insert(task_id, TaskStatus::Failed(err.to_string()));
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to lock tasks mutex for update: {}", e);
                }
            }
        });
        
        Ok(task_id)
    }
    
    /// Poll for the result of a task
    pub fn poll_task(&self, task_id: u64) -> Result<Option<HostFunctionResponse>> {
        let tasks = self.tasks.lock().map_err(|e| anyhow::anyhow!("Failed to lock tasks mutex: {}", e))?;
        
        Ok(match tasks.get(&task_id) {
            Some(TaskStatus::Running) => Some(HostFunctionResponse::TaskPending),
            Some(TaskStatus::Completed(data)) => Some(HostFunctionResponse::Data(data.clone())),
            Some(TaskStatus::Failed(err)) => Some(HostFunctionResponse::Error(err.clone())),
            None => Some(HostFunctionResponse::Error(format!("Task not found: {}", task_id))),
        })
    }
}