use serde::Serialize;

/// Represents a comment/naming/variable violation (tiers 1-3)
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub line_number: usize,
    pub search_term: String,    // Pattern matched (e.g., "TODO", "FIXME", "stub_")
    pub method_name: String,    // Method/function name if applicable, "" otherwise
    pub context: String,        // Code snippet: 2 lines before + violation line + 2 lines after
}

/// Represents a panic-prone pattern (.unwrap, .expect)
#[derive(Debug, Clone, Serialize)]
pub struct PanicPattern {
    pub line_number: usize,
    pub pattern: String,        // e.g., ".unwrap()", ".expect()"
    pub issue: String,          // Description: "Can panic in production" / "Should use expect in tests"
    pub context: String,        // Code snippet
}

/// Represents a test found in ./src directory (should be in ./tests)
#[derive(Debug, Clone, Serialize)]
pub struct TestInSrc {
    pub line_number: usize,
    pub test_attribute: String, // e.g., "#[test]", "#[tokio::test]", "#[cfg(test)]"
    pub file_path: String,      // Relative path to file
    pub context: String,        // Code snippet
}

/// Represents a module declared but never used
#[derive(Debug, Clone, Serialize)]
pub struct OrphanedModule {
    pub name: String,           // Module name
    pub declared_in: String,    // File where module is declared
    pub line_number: usize,
    pub visibility: String,     // "pub" or "private"
    pub context: String,        // Code snippet
}

/// Represents a function/method defined but never called
#[derive(Debug, Clone, Serialize)]
pub struct OrphanedMethod {
    pub name: String,           // Function name (without parens)
    pub file_path: String,      // File containing the function
    pub line_number: usize,
    pub visibility: String,     // "pub", "pub(crate)", or "private"
    pub context: String,        // Code snippet
}

/// Represents a dependency in Cargo.toml not imported anywhere
#[derive(Debug, Clone, Serialize)]
pub struct UnusedDependency {
    pub name: String,           // Crate name
    pub cargo_toml: String,     // Path to Cargo.toml
    pub section: String,        // "[dependencies]", "[dev-dependencies]", or "[build-dependencies]"
}

/// Metadata about a function definition (used for orphan detection)
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub visibility: String,
    pub context: String,
}

/// Computes an 8-character hash from a file path for unique task file naming
///
/// # Example
/// ```
/// use std::path::Path;
/// use kargo_turd::compute_file_hash;
///
/// let path = Path::new("src/service/handler.rs");
/// let hash = compute_file_hash(path);
/// assert_eq!(hash.len(), 8);
/// ```
pub fn compute_file_hash(path: &std::path::Path) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..8].to_string()
}
