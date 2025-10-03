use serde::Serialize;
use super::*;

/// Complete context object passed to minijinja templates
///
/// This structure contains all information needed to render a task file for a single source file.
/// Templates can access all fields directly via {{ field_name }} syntax.
///
/// See: [./prompt/VARIABLES.md](../../prompt/VARIABLES.md) for complete variable documentation
#[derive(Debug, Serialize)]
pub struct TemplateContext {
    // File identification
    pub project_relative_path: String,  // e.g., "src/service/handler.rs"
    pub absolute_path: String,           // e.g., "/Users/dev/project/src/service/handler.rs"
    pub project_name: String,            // From Cargo.toml package name
    pub file_hash: String,               // SHA256 hash for unique file naming (8 chars)
    pub timestamp: String,               // ISO 8601 timestamp
    pub lines_of_code: u32,              // Excluding blanks and comments
    pub version: String,                 // kargo-turd version

    // Control flags
    pub needs_decomposition: bool,       // true if lines_of_code > 300

    // Violation arrays (empty if none found)
    pub tier1_violations: Vec<Violation>,
    pub tier2_violations: Vec<Violation>,
    pub tier3_violations: Vec<Violation>,
    pub panic_patterns: Vec<PanicPattern>,
    pub tests_in_src: Vec<TestInSrc>,
    pub orphaned_modules: Vec<OrphanedModule>,
    pub orphaned_methods: Vec<OrphanedMethod>,
    pub unused_dependencies: Vec<UnusedDependency>,
}

impl TemplateContext {
    /// Returns the highest tier of violations found (1, 2, or 3)
    ///
    /// This determines which tier directory the task file will be placed in:
    /// - tier1/ - Most critical (almost certainly stubbed code)
    /// - tier2/ - Medium priority
    /// - tier3/ - Low priority (might be false positives)
    pub fn highest_tier(&self) -> u8 {
        if !self.tier1_violations.is_empty() { 1 }
        else if !self.tier2_violations.is_empty() { 2 }
        else { 3 }
    }
}
