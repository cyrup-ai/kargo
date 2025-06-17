use std::path::PathBuf;

/// Configuration for the documentation generator
#[derive(Debug, Clone)]
pub struct Config {
    /// Package name with optional version (e.g., 'tokio' or 'tokio@1.28.0')
    pub package_spec: String,

    /// Output directory for documentation
    pub output_dir: PathBuf,

    /// Use specific temporary directory
    pub temp_dir: Option<PathBuf>,

    /// Keep temporary directory after completion
    pub keep_temp: bool,

    /// Skip checking/installing rustup components
    pub skip_component_check: bool,

    /// Enable verbose output
    pub verbose: bool,

    /// Include private items in documentation
    pub document_private_items: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            package_spec: String::new(),
            output_dir: PathBuf::from("./rust_docs"),
            temp_dir: None,
            keep_temp: false,
            skip_component_check: false,
            verbose: false,
            document_private_items: false,
        }
    }
}
