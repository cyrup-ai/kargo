use std::path::PathBuf;

/// Configuration for kargo-turd analysis
#[derive(Debug, Clone)]
pub struct Config {
    /// Watch mode: path to monitor for file changes (None = run once and exit)
    pub watch_path: Option<PathBuf>,

    /// Exclude files matching these glob patterns (e.g., "**/generated/**", "**/*_pb.rs")
    pub exclude_patterns: Vec<String>,

    /// Output directory for generated task files (default: ./task)
    pub output_dir: PathBuf,

    /// Tokio runtime handle for watch mode (required for watchexec)
    pub runtime_handle: Option<tokio::runtime::Handle>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watch_path: None,
            exclude_patterns: vec![
                "**/target/**".to_string(),  // Cargo build artifacts
                "**/task/**".to_string(),    // Don't analyze our own output
                "**/forks/**".to_string(),   // Don't analyze forked projects
                "**/vendor/**".to_string(),  // Don't analyze vendored dependencies
            ],
            output_dir: PathBuf::from("./task"),
            runtime_handle: None,
        }
    }
}
