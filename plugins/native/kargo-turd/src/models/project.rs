use std::path::PathBuf;

/// Represents a Rust project with a Cargo.toml file
#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub cargo_toml_path: PathBuf,
    pub src_files: Vec<PathBuf>,
    pub test_files: Vec<PathBuf>,
}

impl Project {
    /// Creates a new Project from a Cargo.toml path
    ///
    /// # Example
    /// ```
    /// use std::path::PathBuf;
    /// use kargo_turd::Project;
    ///
    /// let cargo_toml = PathBuf::from("./packages/my-crate/Cargo.toml");
    /// let project = Project::new(cargo_toml)?;
    /// assert_eq!(project.name, "my-crate");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(cargo_toml_path: PathBuf) -> anyhow::Result<Self> {
        let dir = cargo_toml_path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let name = dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            name,
            cargo_toml_path,
            src_files: Vec::new(),
            test_files: Vec::new(),
        })
    }
}
