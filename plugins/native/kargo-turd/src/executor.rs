use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;
use std::collections::{HashMap, HashSet};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use crate::analyzers::{OrphanDetector, find_comment_violations, find_method_naming_violations, find_variable_naming_violations, find_hardcoded_values, ast_analyzer};
use crate::file_queue::FileEntry;
use crate::models::{Violation, PanicPattern, TestInSrc, OrphanedMethod, OrphanedModule};
use crate::template_renderer::count_lines_of_code;

// ============================================================================
// ANALYSIS EXECUTOR - Parallel file processing with rayon
// ============================================================================

/// Orchestrates parallel file analysis
///
/// Uses Arc<Mutex<OrphanDetector>> for thread-safe accumulation of
/// function definitions and calls across all files.
pub struct AnalysisExecutor {
    /// Shared orphan detector (thread-safe)
    orphan_detector: Arc<Mutex<OrphanDetector>>,
}

impl AnalysisExecutor {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            orphan_detector: Arc::new(Mutex::new(OrphanDetector::new())),
        }
    }
}

impl Default for AnalysisExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of analyzing a single file
pub struct FileAnalysisResult {
    pub file_path: PathBuf,
    pub tier1_violations: Vec<Violation>,
    pub tier2_violations: Vec<Violation>,
    pub tier3_violations: Vec<Violation>,
    pub panic_patterns: Vec<PanicPattern>,
    pub tests_in_src: Vec<TestInSrc>,
    pub lines_of_code: u32,
}

impl AnalysisExecutor {
    /// Analyze all files in parallel
    ///
    /// Uses rayon's `par_iter()` to process files across all CPU cores.
    /// Each thread runs `analyze_file()` independently.
    ///
    /// # Thread Safety
    /// `OrphanDetector` is wrapped in Arc<Mutex<>> for thread-safe access.
    /// Each thread locks the mutex briefly to add its results.
    pub fn analyze_files(
        &self,
        file_queue: Vec<FileEntry>,
        project_name: &str,
    ) -> Result<Vec<FileAnalysisResult>> {
        // par_iter() creates parallel iterator
        // filter_map() runs on each thread
        let results: Vec<_> = file_queue.par_iter()
            .filter_map(|entry| {
                // If analyze_file() returns Err, filter_map skips it (None)
                // If Ok, filter_map includes it (Some)
                self.analyze_file(entry, project_name).ok()
            })
            .collect();

        Ok(results)
    }

    /// Analyze a single file (called in parallel)
    fn analyze_file(
        &self,
        entry: &FileEntry,
        _project_name: &str,
    ) -> Result<FileAnalysisResult> {
        let content = fs::read_to_string(&entry.path)?;
        let file_path = entry.path.to_string_lossy().to_string();
        
        // ===== Pattern Matching (TURD_3) =====
        let tier1_comments = find_comment_violations(&content, 1);
        let tier2_comments = find_comment_violations(&content, 2);
        let tier3_comments = find_comment_violations(&content, 3);

        let mut all_tier1 = tier1_comments;
        all_tier1.extend(find_method_naming_violations(&content, 1));
        all_tier1.extend(find_variable_naming_violations(&content, 1));
        all_tier1.extend(find_hardcoded_values(&content));

        let mut all_tier2 = tier2_comments;
        all_tier2.extend(find_method_naming_violations(&content, 2));
        let all_tier3 = tier3_comments;
        
        // ===== AST Analysis (TURD_4) =====
        let ast_result = ast_analyzer::analyze_file(&content, &file_path, entry.is_test)?;
        
        // ===== Orphan Detection (TURD_5) =====
        {
            let mut detector = self.orphan_detector.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock orphan detector: {e}"))?;
            detector.add_file_analysis(&file_path, &ast_result);
        }
        
        // ===== Line Counting (TURD_7) =====
        let lines_of_code = count_lines_of_code(&content);
        
        Ok(FileAnalysisResult {
            file_path: entry.path.clone(),
            tier1_violations: all_tier1,
            tier2_violations: all_tier2,
            tier3_violations: all_tier3,
            panic_patterns: ast_result.panic_patterns,
            tests_in_src: ast_result.tests_in_src,
            lines_of_code,
        })
    }

    /// Get orphaned methods after all files analyzed
    ///
    /// Returns `HashMap`<`file_path`, Vec<OrphanedMethod>>
    /// grouped by source file for task file generation
    pub fn get_orphaned_methods(&self) -> Result<HashMap<String, Vec<OrphanedMethod>>> {
        let detector = self.orphan_detector.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock orphan detector: {e}"))?;
        Ok(detector.find_orphaned_methods())
    }

    /// Get orphaned modules after all files analyzed
    ///
    /// Returns Vec<OrphanedModule> for modules that are
    /// declared but never imported
    pub fn get_orphaned_modules(&self) -> Result<Vec<OrphanedModule>> {
        let detector = self.orphan_detector.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock orphan detector: {e}"))?;
        Ok(detector.find_orphaned_modules())
    }

    /// Add module information from a file
    ///
    /// Should be called during or after file analysis for each file
    pub fn add_module_info(
        &self,
        decls: Vec<OrphanedModule>,
        uses: HashSet<String>,
    ) -> Result<()> {
        let mut detector = self.orphan_detector.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock orphan detector: {e}"))?;
        detector.add_module_info(decls, uses);
        Ok(())
    }

    /// Analyze files with progress bar
    ///
    /// Same as `analyze_files()` but shows progress to user.
    /// `ProgressBar` is thread-safe (Sync + Send) and can be used
    /// directly in rayon parallel iterators without Arc<Mutex<>>.
    pub fn analyze_files_with_progress(
        &self,
        file_queue: Vec<FileEntry>,
        project_name: &str,
    ) -> Result<Vec<FileAnalysisResult>> {
        // Create progress bar with known total
        let pb = ProgressBar::new(file_queue.len() as u64);

        // Configure progress bar style
        // Note: template() returns Result in indicatif 0.17
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files")?
        );

        // Parallel iteration with progress updates
        // ProgressBar.inc() is thread-safe - no additional locking needed
        let results: Vec<_> = file_queue.par_iter()
            .filter_map(|entry| {
                let result = self.analyze_file(entry, project_name).ok();

                // Thread-safe increment (ProgressBar handles internal locking)
                pb.inc(1);

                result
            })
            .collect();

        // Replace progress bar with completion message
        pb.finish_with_message("Analysis complete");

        Ok(results)
    }
}
