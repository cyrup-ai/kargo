use std::path::PathBuf;
use std::fs;

/// File entry with metadata for analysis
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,      // File size in bytes
    pub is_test: bool,  // Is this a test file?
}

/// Build priority queue from src and test files
///
/// Sorts files by size (largest first) for optimal parallel processing
///
/// # Arguments
/// * `src_files` - Vec of paths from src/ directory
/// * `test_files` - Vec of paths from tests/ directory
///
/// # Returns
/// Vec<FileEntry> sorted by size descending
#[must_use] 
pub fn build_priority_queue(
    src_files: &[PathBuf],
    test_files: &[PathBuf],
) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    
    // Add src files
    for path in src_files {
        if let Ok(metadata) = fs::metadata(path) {
            entries.push(FileEntry {
                path: path.clone(),
                size: metadata.len(),
                is_test: false,
            });
        }
        // If metadata fails (permission denied), skip file silently
    }
    
    // Add test files
    for path in test_files {
        if let Ok(metadata) = fs::metadata(path) {
            entries.push(FileEntry {
                path: path.clone(),
                size: metadata.len(),
                is_test: true,
            });
        }
    }
    
    // Sort by size descending (largest files first)
    entries.sort_by(|a, b| b.size.cmp(&a.size));
    
    entries
}
