use minijinja::{Environment, path_loader, context};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use chrono::Utc;
use crate::models::*;

// ============================================================================
// TEMPLATE RENDERING
// ============================================================================

/// Render task file from template context
///
/// Loads master.j2.md and all included templates from template_dir
///
/// # Arguments
/// * `ctx` - TemplateContext with all violation data
/// * `template_dir` - Path to ./prompt/ directory
///
/// # Returns
/// Rendered markdown as String
pub fn render_task_file(ctx: TemplateContext, template_dir: &Path) -> Result<String> {
    // Create minijinja environment
    let mut env = Environment::new();
    
    // Set up path loader to find templates in template_dir
    // This allows {% include "tier1.j2.md" %} to work
    env.set_loader(path_loader(template_dir));
    
    // Load main template
    let tmpl = env.get_template("master.j2.md")?;
    
    // Render with context
    // context! macro creates a Context object with named values
    let rendered = tmpl.render(context! {
        project_relative_path => ctx.project_relative_path,
        absolute_path => ctx.absolute_path,
        project_name => ctx.project_name,
        file_hash => ctx.file_hash,
        timestamp => ctx.timestamp,
        lines_of_code => ctx.lines_of_code,
        version => ctx.version,
        needs_decomposition => ctx.needs_decomposition,
        
        // Violation arrays
        tier1_violations => ctx.tier1_violations,
        tier2_violations => ctx.tier2_violations,
        tier3_violations => ctx.tier3_violations,
        panic_patterns => ctx.panic_patterns,
        tests_in_src => ctx.tests_in_src,
        orphaned_modules => ctx.orphaned_modules,
        orphaned_methods => ctx.orphaned_methods,
        unused_dependencies => ctx.unused_dependencies,
    })?;
    
    Ok(rendered)
}

// ============================================================================
// CONTEXT BUILDER
// ============================================================================

/// All violation data collected from analyzing a single file
pub struct ViolationData {
    pub tier1_violations: Vec<Violation>,
    pub tier2_violations: Vec<Violation>,
    pub tier3_violations: Vec<Violation>,
    pub panic_patterns: Vec<PanicPattern>,
    pub tests_in_src: Vec<TestInSrc>,
    pub orphaned_modules: Vec<OrphanedModule>,
    pub orphaned_methods: Vec<OrphanedMethod>,
    pub unused_dependencies: Vec<UnusedDependency>,
}

/// Helper for building TemplateContext from analysis results
pub struct ContextBuilder {
    pub project_name: String,
    pub file_path: PathBuf,
    pub absolute_path: PathBuf,
}

impl ContextBuilder {
    /// Build TemplateContext with all violation data
    ///
    /// Computes:
    /// - file_hash from file path
    /// - timestamp (current time in RFC3339 format)
    /// - needs_decomposition (true if > 300 LOC)
    /// - project_relative_path (relative to project root)
    pub fn build(
        &self,
        violations: ViolationData,
        lines_of_code: u32,
    ) -> TemplateContext {
        // Compute file hash (8-char SHA256)
        let file_hash = compute_file_hash(&self.file_path);
        
        // Get current timestamp in ISO 8601 format
        let timestamp = Utc::now().to_rfc3339();
        
        // Check if file needs decomposition (>300 LOC)
        let needs_decomposition = lines_of_code > 300;
        
        // Compute project-relative path
        // e.g., /Users/me/project/src/main.rs → src/main.rs
        let project_relative_path = self.file_path
            .strip_prefix(&self.absolute_path)
            .unwrap_or(&self.file_path)
            .to_string_lossy()
            .to_string();
        
        TemplateContext {
            project_relative_path,
            absolute_path: self.absolute_path.to_string_lossy().to_string(),
            project_name: self.project_name.clone(),
            file_hash,
            timestamp,
            lines_of_code,
            version: env!("CARGO_PKG_VERSION").to_string(),
            needs_decomposition,
            tier1_violations: violations.tier1_violations,
            tier2_violations: violations.tier2_violations,
            tier3_violations: violations.tier3_violations,
            panic_patterns: violations.panic_patterns,
            tests_in_src: violations.tests_in_src,
            orphaned_modules: violations.orphaned_modules,
            orphaned_methods: violations.orphaned_methods,
            unused_dependencies: violations.unused_dependencies,
        }
    }
}

// ============================================================================
// FILE WRITING
// ============================================================================

/// Write rendered task file to tier-specific directory
///
/// Creates directory structure:
/// ./task/_<project_name>/tier<N>/<file_name>_<hash>.md
///
/// # Arguments
/// * `rendered` - Rendered markdown string
/// * `output_dir` - Base output directory (usually ./task)
/// * `project_name` - Project name for directory
/// * `file_name` - Source file name (without extension)
/// * `file_hash` - 8-char file hash
/// * `tier` - Tier number (1, 2, or 3)
///
/// # Returns
/// Path to written file
pub fn write_task_file(
    rendered: &str,
    output_dir: &Path,
    project_name: &str,
    file_name: &str,
    file_hash: &str,
    tier: u8,
) -> Result<PathBuf> {
    // Build directory path: ./task/_<project>/tier<N>/
    let tier_dir = output_dir
        .join(format!("_{}", project_name))
        .join(format!("tier{}", tier));
    
    // Create directories if they don't exist
    fs::create_dir_all(&tier_dir)?;
    
    // Build filename: <file_name_no_ext>_<hash>.md
    let output_file = tier_dir.join(format!("{}_{}.md", file_name, file_hash));
    
    // Write rendered content
    fs::write(&output_file, rendered)?;
    
    Ok(output_file)
}

// ============================================================================
// LINE COUNTING
// ============================================================================

/// Count non-blank, non-comment lines of code
///
/// Excludes:
/// - Empty lines (whitespace only)
/// - Lines that are only comments (// or ///)
///
/// Includes:
/// - Code lines with trailing comments: `let x = 5; // comment` counts as 1
///
/// # Example
/// ```
/// use kargo_turd::count_lines_of_code;
///
/// let code = r#"
/// use std::fs;  // 1
///               // 0 (blank)
/// // comment    // 0 (comment only)
/// fn main() {   // 1
///     let x = 5; // comment  // 1
/// }             // 1
/// "#;
/// assert_eq!(count_lines_of_code(code), 4);
/// ```
pub fn count_lines_of_code(content: &str) -> u32 {
    content.lines()
        .filter(|line| {
            let trimmed = line.trim();
            
            // Skip blank lines
            if trimmed.is_empty() {
                return false;
            }
            
            // Skip comment-only lines
            if trimmed.starts_with("//") {
                return false;
            }
            
            // Count everything else (including code with trailing comments)
            true
        })
        .count() as u32
}
