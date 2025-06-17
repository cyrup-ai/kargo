use crate::error::Error;
use log::{debug, warn};
use std::fs;
use std::path::{Path, PathBuf};

/// Create a directory and all parent directories
pub fn create_dir_all(path: &Path) -> Result<(), Error> {
    debug!("Creating directory: {}", path.display());
    fs::create_dir_all(path).map_err(|e| Error::Io(e))
}

/// Find files matching a pattern in a directory
pub fn find_files(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, Error> {
    debug!(
        "Searching for files matching '{}' in {}",
        pattern,
        dir.display()
    );

    let mut result = Vec::new();
    if !dir.exists() {
        warn!("Directory does not exist: {}", dir.display());
        return Ok(result);
    }

    let entries = fs::read_dir(dir).map_err(|e| Error::Io(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| Error::Io(e))?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively search subdirectories
            let subdirectory_matches = find_files(&path, pattern)?;
            result.extend(subdirectory_matches);
        } else if let Some(filename) = path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str.contains(pattern) {
                    result.push(path.clone());
                }
            }
        }
    }

    debug!("Found {} matching files", result.len());
    Ok(result)
}

/// Copy a file with proper error handling
pub fn copy_file(src: &Path, dst: &Path) -> Result<(), Error> {
    debug!("Copying file from {} to {}", src.display(), dst.display());

    // Make sure parent directory exists
    if let Some(parent) = dst.parent() {
        create_dir_all(parent)?;
    }

    fs::copy(src, dst)
        .map(|_| ())
        .map_err(|e| Error::DocCopyFailed(format!("Failed to copy file: {}", e)))
}

/// Check if a file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Write content to a file with proper error handling
pub fn write_file(path: &Path, content: &str) -> Result<(), Error> {
    debug!("Writing to file: {}", path.display());

    // Make sure parent directory exists
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    fs::write(path, content).map_err(|e| Error::Io(e))
}

/// Read a file to string with proper error handling
pub fn read_file(path: &Path) -> Result<String, Error> {
    debug!("Reading file: {}", path.display());
    fs::read_to_string(path).map_err(|e| Error::Io(e))
}
