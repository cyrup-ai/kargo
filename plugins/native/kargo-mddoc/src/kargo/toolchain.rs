use crate::error::Error;
use log::{debug, info, warn};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, SystemTime};

pub struct Toolchain;

impl Toolchain {
    /// Check if rustup is installed
    pub fn check_rustup() -> Result<(), Error> {
        debug!("Checking if rustup is installed");
        match Command::new("rustup").arg("--version").output() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::RustupNotFound),
        }
    }

    /// Check if cargo is installed
    pub fn check_cargo() -> Result<(), Error> {
        debug!("Checking if cargo is installed");
        match Command::new("cargo").arg("--version").output() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::CargoNotFound),
        }
    }

    /// Get the cache directory for toolchain updates
    fn get_cache_dir() -> Result<PathBuf, Error> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| Error::Other("Could not find home directory".to_string()))?;
        let cache_dir = home_dir.join(".cache").join("rustdoc-md");

        // Create directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| Error::Other(format!("Failed to create cache directory: {}", e)))?;
        }

        Ok(cache_dir)
    }

    /// Check if nightly toolchain needs update
    fn should_update_nightly() -> Result<bool, Error> {
        let cache_file = Self::get_cache_dir()?.join("nightly_update_timestamp");

        // If the file doesn't exist, we definitely need to update
        if !cache_file.exists() {
            return Ok(true);
        }

        // Read the timestamp file
        let timestamp_str = fs::read_to_string(&cache_file)
            .map_err(|e| Error::Other(format!("Failed to read cache file: {}", e)))?;

        let timestamp = timestamp_str
            .trim()
            .parse::<u64>()
            .map_err(|e| Error::Other(format!("Invalid timestamp format: {}", e)))?;

        // Convert to SystemTime
        let timestamp_time = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);

        // Check if 24 hours have passed
        match SystemTime::now().duration_since(timestamp_time) {
            Ok(duration) => Ok(duration.as_secs() > 24 * 60 * 60), // 24 hours in seconds
            Err(_) => {
                // System clock might have moved backwards; we'll update to be safe
                warn!("System time inconsistency detected, forcing toolchain update");
                Ok(true)
            }
        }
    }

    /// Update the nightly update timestamp
    fn update_nightly_timestamp() -> Result<(), Error> {
        let cache_file = Self::get_cache_dir()?.join("nightly_update_timestamp");

        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| Error::Other(format!("Failed to get system time: {}", e)))?
            .as_secs();

        // Write to cache file
        fs::write(&cache_file, timestamp.to_string())
            .map_err(|e| Error::Other(format!("Failed to write cache file: {}", e)))?;

        Ok(())
    }

    /// Check if nightly toolchain is installed, install or update if needed
    pub fn ensure_nightly_toolchain() -> Result<(), Error> {
        debug!("Checking for nightly toolchain");

        // Check if nightly is installed
        let output = Command::new("rustup")
            .args(["toolchain", "list"])
            .output()
            .map_err(|e| Error::RustupCheckFailed(e.to_string()))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let nightly_installed = output_str.contains("nightly");

        // Check if we need to update based on our cache
        let should_update = Self::should_update_nightly()?;

        if !nightly_installed {
            // Install nightly if not present
            info!("Installing nightly toolchain");
            Self::install_nightly_toolchain()?;
        } else if should_update {
            // Update nightly if it's been more than 24 hours
            info!("Updating nightly toolchain");
            Self::update_nightly_toolchain()?;
        } else {
            debug!("Nightly toolchain is already installed and up-to-date");
        }

        // Update our timestamp
        Self::update_nightly_timestamp()?;

        Ok(())
    }

    /// Install the nightly toolchain
    fn install_nightly_toolchain() -> Result<(), Error> {
        let install_output = Command::new("rustup")
            .args(["toolchain", "install", "nightly"])
            .output()
            .map_err(|e| Error::Toolchain(format!("Failed to install nightly: {}", e)))?;

        if !install_output.status.success() {
            return Err(Error::Toolchain(format!(
                "Failed to install nightly toolchain: {}",
                String::from_utf8_lossy(&install_output.stderr)
            )));
        }

        info!("Nightly toolchain installed successfully");
        Ok(())
    }

    /// Update the nightly toolchain
    fn update_nightly_toolchain() -> Result<(), Error> {
        let update_output = Command::new("rustup")
            .args(["update", "nightly"])
            .output()
            .map_err(|e| Error::Toolchain(format!("Failed to update nightly: {}", e)))?;

        if !update_output.status.success() {
            return Err(Error::Toolchain(format!(
                "Failed to update nightly toolchain: {}",
                String::from_utf8_lossy(&update_output.stderr)
            )));
        }

        info!("Nightly toolchain updated successfully");
        Ok(())
    }

    /// Check if rust-docs component is installed for nightly, install if not
    pub fn ensure_rustdoc_component() -> Result<(), Error> {
        debug!("Checking for rust-docs component in nightly toolchain");

        let output = Command::new("rustup")
            .args(["component", "list", "--toolchain", "nightly"])
            .output()
            .map_err(|e| Error::RustupCheckFailed(e.to_string()))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        if !output_str.contains("rust-docs (installed)") {
            info!("Installing rust-docs component for nightly toolchain");
            let install_output = Command::new("rustup")
                .args(["component", "add", "rust-docs", "--toolchain", "nightly"])
                .output()
                .map_err(|e| Error::Toolchain(format!("Failed to install rust-docs: {}", e)))?;

            if !install_output.status.success() {
                return Err(Error::Toolchain(format!(
                    "Failed to install rust-docs component: {}",
                    String::from_utf8_lossy(&install_output.stderr)
                )));
            }

            info!("Rust-docs component installed successfully");
        } else {
            debug!("Rust-docs component is already installed");
        }

        Ok(())
    }

    /// Run a command and return its output, with error handling
    pub fn run_command(
        command: &str,
        args: &[&str],
        current_dir: Option<&std::path::Path>,
        verbose: bool,
    ) -> Result<Output, Error> {
        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = current_dir {
            cmd.current_dir(dir);
        }

        debug!("Running command: {:?} {:?}", command, args);

        let output = cmd
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to execute {}: {}", command, e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            if verbose {
                eprintln!("Command failed: {} {:?}", command, args);
                eprintln!("Status: {}", output.status);
                eprintln!("Stdout: {}", stdout);
                eprintln!("Stderr: {}", stderr);
            }

            return Err(Error::CommandFailed(format!(
                "Command failed with status {}: {}",
                output.status, stderr
            )));
        }

        Ok(output)
    }
}
