use crate::config::Config;
use crate::error::Error;
use crate::package::PackageSpec;
use crate::toolchain::Toolchain;
use crate::utils;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Generator for Rust package documentation
pub struct DocGenerator {
    /// Command line options
    config: Config,
    /// Parsed package specification
    package_spec: PackageSpec,
    /// Temporary directory for the project
    temp_dir: Option<TempDir>,
    /// Path to the temporary project
    project_dir: PathBuf,
    /// Output directory
    output_dir: PathBuf,
}

impl DocGenerator {
    /// Create a new documentation generator
    pub fn new(config: Config) -> Result<Self, Error> {
        // Parse the package specification
        let package_spec = PackageSpec::parse(&config.package_spec)?;

        if package_spec.name.is_empty() {
            return Err(Error::InvalidPackageName(
                "Package name cannot be empty".to_string(),
            ));
        }

        // Get output directory
        let output_dir = config.output_dir.clone();

        // Set up the temporary directory
        let (temp_dir, project_dir) = Self::setup_temp_dir(&config)?;

        Ok(Self {
            config,
            package_spec,
            temp_dir,
            project_dir,
            output_dir,
        })
    }

    /// Run the documentation generation process
    pub fn run(&mut self) -> Result<PathBuf, Error> {
        let progress = self.setup_progress_bar();

        // Check requirements
        progress.set_message("Checking Rust toolchain requirements...");
        self.check_requirements()?;
        progress.inc(1);

        // Set up the project
        progress.set_message("Setting up temporary project...");
        self.setup_project()?;
        progress.inc(1);

        // Fetch dependencies
        progress.set_message("Fetching package dependencies...");
        self.fetch_dependencies()?;
        progress.inc(1);

        // Generate documentation
        progress.set_message("Generating JSON documentation...");
        self.generate_documentation()?;
        progress.inc(1);

        // Find and copy documentation
        progress.set_message("Processing documentation files...");
        let output_file = self.process_documentation()?;
        progress.inc(1);

        // Finish
        progress.finish_with_message("Documentation generation complete!");
        Ok(output_file)
    }

    /// Set up progress bar for visual feedback
    fn setup_progress_bar(&self) -> ProgressBar {
        let pb = ProgressBar::new(5);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("#>-"),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    /// Check all requirements
    fn check_requirements(&self) -> Result<(), Error> {
        // Check if rustup and cargo are installed
        Toolchain::check_rustup()?;
        Toolchain::check_cargo()?;

        // Check for nightly toolchain and rustdoc component
        if !self.config.skip_component_check {
            Toolchain::ensure_nightly_toolchain()?;
            Toolchain::ensure_rustdoc_component()?;
        }

        Ok(())
    }

    /// Set up the temporary directory
    fn setup_temp_dir(config: &Config) -> Result<(Option<TempDir>, PathBuf), Error> {
        match &config.temp_dir {
            Some(dir) => {
                // Use the specified directory
                utils::create_dir_all(dir)?;
                debug!("Using specified temporary directory: {}", dir.display());
                Ok((None, dir.clone()))
            }
            None => {
                // Create a new temporary directory
                let temp_dir = tempfile::Builder::new()
                    .prefix("rustdoc-md-")
                    .tempdir()
                    .map_err(|e| Error::TempProjectSetup(e.to_string()))?;
                let temp_path = temp_dir.path().to_path_buf();
                debug!("Created temporary directory: {}", temp_path.display());
                Ok((Some(temp_dir), temp_path))
            }
        }
    }

    /// Set up the project directory with Cargo.toml
    fn setup_project(&self) -> Result<(), Error> {
        debug!("Setting up project in {}", self.project_dir.display());

        // Create src directory
        let src_dir = self.project_dir.join("src");
        utils::create_dir_all(&src_dir)?;

        // Create a minimal main.rs
        let main_rs = src_dir.join("main.rs");
        utils::write_file(
            &main_rs,
            "fn main() { println!(\"Documentation generator\"); }\n",
        )?;

        // Create Cargo.toml
        let cargo_toml = self.project_dir.join("Cargo.toml");
        let mut cargo_content = String::from(
            r#"[package]
name = "doc-generator"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        );

        // Add the package dependency
        cargo_content.push_str(&format!(
            "{} = {}\n",
            self.package_spec.name,
            self.package_spec.version_spec()
        ));

        utils::write_file(&cargo_toml, &cargo_content)?;
        debug!("Created Cargo.toml file");

        Ok(())
    }

    /// Fetch dependencies to ensure the package exists
    fn fetch_dependencies(&self) -> Result<(), Error> {
        debug!("Fetching dependencies");

        let output = Toolchain::run_command(
            "cargo",
            &["fetch"],
            Some(&self.project_dir),
            self.config.verbose,
        )?;

        debug!(
            "Dependencies fetched successfully: {}",
            String::from_utf8_lossy(&output.stdout)
        );

        Ok(())
    }

    /// Generate the JSON documentation
    fn generate_documentation(&self) -> Result<(), Error> {
        debug!("Generating JSON documentation for {}", self.package_spec);

        // Prepare rustdoc arguments
        let mut args = vec![
            "+nightly",
            "-Zunstable-options",
            "rustdoc",
            "--output-format",
            "json",
            "--package",
            &self.package_spec.name,
        ];

        // Add option for private items if requested
        if self.config.document_private_items {
            args.push("--document-private-items");
        }

        // Note: Standard rustdoc JSON generation includes all public items by default
        // No additional flags needed for public API documentation

        // Run cargo with rustdoc
        let output =
            Toolchain::run_command("cargo", &args, Some(&self.project_dir), self.config.verbose)?;

        debug!(
            "Documentation generated successfully: {}",
            String::from_utf8_lossy(&output.stdout)
        );

        Ok(())
    }

    /// Find and copy the generated documentation
    fn process_documentation(&self) -> Result<PathBuf, Error> {
        debug!("Looking for generated documentation files");

        // Find the generated JSON file in target/doc
        let doc_dir = self.project_dir.join("target").join("doc");
        let pattern = format!("{}.json", self.package_spec.name);
        let files = utils::find_files(&doc_dir, &pattern)?;

        if files.is_empty() {
            return Err(Error::DocNotFound);
        }

        // Use the first matching file
        let source_file = &files[0];
        debug!("Found documentation file: {}", source_file.display());

        // Make sure output directory exists
        utils::create_dir_all(&self.output_dir)?;

        // Create the output filename
        let output_file = self.output_dir.join(self.package_spec.json_filename());
        utils::copy_file(source_file, &output_file)?;

        info!("Documentation saved to: {}", output_file.display());

        Ok(output_file)
    }
}

impl Drop for DocGenerator {
    fn drop(&mut self) {
        // Clean up temporary directory if needed
        if !self.config.keep_temp {
            if let Some(temp_dir) = self.temp_dir.take() {
                debug!("Cleaning up temporary directory");
                if let Err(e) = temp_dir.close() {
                    warn!("Failed to clean up temporary directory: {}", e);
                }
            }
        } else {
            debug!(
                "Keeping temporary directory: {}",
                self.project_dir.display()
            );
        }
    }
}
