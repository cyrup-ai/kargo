pub mod config;
pub mod models;
pub mod project_scanner;
pub mod analyzers;
pub mod template_renderer;
pub mod plugin;
pub mod file_queue;
pub mod executor;
pub mod watch;

pub use config::*;
pub use models::*;
pub use project_scanner::*;
pub use analyzers::*;
pub use template_renderer::*;
pub use file_queue::*;
pub use executor::*;
pub use watch::*;

// ============================================================================
// PUBLIC API - Synchronous analysis for watch mode
// ============================================================================

/// Run full analysis pipeline (SYNC version for watch mode)
///
/// This is the SYNC version of the analysis pipeline, suitable for
/// calling from watchexec's action handler (which is a sync closure).
///
/// Called by:
/// - plugin.rs in async context via `tokio::task::spawn_blocking` (correct pattern)
/// - watch.rs action handler (sync context)
///
/// Architecture: `spawn_blocking` + rayon for CPU-bound work
/// - Plugin interface requires async (`PluginCommand` trait)
/// - Analysis is CPU-intensive (AST parsing, pattern matching)
/// - Rayon provides optimal parallelism via dedicated thread pool
/// - `spawn_blocking` bridges async→sync without blocking async executor
///
/// This follows Tokio's recommended pattern for CPU-bound work in async contexts.
/// See: <https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html>
pub fn run_analysis_sync(config: &Config) -> anyhow::Result<()> {
    use std::env;
    use log::info;

    println!("🔍 Scanning for Rust projects...");
    info!("Starting kargo-turd analysis");

    // Step 1: Find all Rust projects
    let current_dir = env::current_dir()?;
    info!("Working directory: {current_dir:?}");

    let projects = find_projects_with_progress(&current_dir, config)?;

    if projects.is_empty() {
        println!("No Rust projects found.");
        info!("No projects found in {current_dir:?}");
        return Ok(());
    }

    println!("Found {} project(s)\n", projects.len());
    info!("Discovered {} project(s)", projects.len());

    // Step 2: Analyze each project
    let mut success_count = 0;
    let mut failure_count = 0;

    for project in projects {
        match analyze_project_sync(&project, config) {
            Ok(()) => {
                success_count += 1;
            }
            Err(e) => {
                log::warn!("Failed to analyze project {}: {}", project.name, e);
                eprintln!("⚠️  Failed to analyze project {}: {}", project.name, e);
                failure_count += 1;
            }
        }
    }

    println!("\n✅ Analysis complete!");
    info!("Analysis finished: {success_count} succeeded, {failure_count} failed");

    if failure_count > 0 {
        println!("⚠️  {failure_count} project(s) failed to analyze");
    }

    Ok(())
}

/// Generate a single task file for Cargo.toml aggregating all unused dependencies
fn generate_cargo_toml_task_sync(
    project_name: &str,
    cargo_toml_path: &std::path::Path,
    unused_deps: &[UnusedDependency],
    config: &Config,
) -> anyhow::Result<()> {
    use std::path::Path;
    use std::env;
    use std::fs;

    // Load Cargo.toml contents to compute a simple line count (non-blank lines)
    let content = fs::read_to_string(cargo_toml_path).unwrap_or_default();
    let loc = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count() as u32;

    let context_builder = ContextBuilder {
        project_name: project_name.to_string(),
        file_path: cargo_toml_path.to_path_buf(),
        absolute_path: env::current_dir()?,
    };

    let violations = ViolationData {
        tier1_violations: vec![],
        tier2_violations: vec![],
        tier3_violations: vec![],
        panic_patterns: vec![],
        tests_in_src: vec![],
        orphaned_modules: vec![],
        orphaned_methods: vec![],
        unused_dependencies: unused_deps.to_vec(),
    };

    let context = context_builder.build(violations, loc);
    let tier = context.highest_tier();
    let file_hash = context.file_hash.clone();

    let template_dir = Path::new("./prompt");
    let rendered = render_task_file(context, template_dir)?;

    let file_name = "Cargo.toml".to_string();

    write_task_file(
        &rendered,
        &config.output_dir,
        project_name,
        &file_name,
        &file_hash,
        tier,
    )?;

    Ok(())
}

/// Analyze a single project (SYNC version)
fn analyze_project_sync(project: &Project, config: &Config) -> anyhow::Result<()> {
    use log::info;

    println!("📦 Analyzing project: {}", project.name);
    info!("Project: {} at {:?}", project.name, project.cargo_toml_path);

    // Build priority queue
    let file_queue = build_priority_queue(&project.src_files, &project.test_files);
    println!("  {} files to analyze", file_queue.len());

    if file_queue.is_empty() {
        println!("  ⚠️  No source files found");
        return Ok(());
    }

    // Run parallel analysis (already sync - uses rayon)
    let executor = AnalysisExecutor::new();
    let results = executor.analyze_files_with_progress(file_queue, &project.name)?;

    // Get orphaned methods
    let orphaned_methods = executor.get_orphaned_methods()?;
    let _orphaned_modules = executor.get_orphaned_modules()?;

    info!(
        "Found {} orphaned methods across {} files",
        orphaned_methods.values().map(std::vec::Vec::len).sum::<usize>(),
        orphaned_methods.len()
    );

    // Analyze dependencies
    let unused_deps = analyze_dependencies_sync(project)?;

    if !unused_deps.is_empty() {
        info!("Found {} unused dependencies", unused_deps.len());
    }

    // Generate task files
    let task_files_generated = generate_task_files_sync(
        &results,
        &project.name,
        &project.cargo_toml_path,
        &orphaned_methods,
        &unused_deps,
        config,
    )?;

    println!("  ✅ Generated {task_files_generated} task file(s)\n");

    Ok(())
}

/// Analyze dependencies for unused packages (SYNC version)
fn analyze_dependencies_sync(project: &Project) -> anyhow::Result<Vec<UnusedDependency>> {
    use std::fs;

    let src_contents: Vec<(String, String)> = project
        .src_files
        .iter()
        .filter_map(|p| {
            fs::read_to_string(p).ok().map(|content| {
                (p.to_string_lossy().to_string(), content)
            })
        })
        .collect();

    let test_contents: Vec<(String, String)> = project
        .test_files
        .iter()
        .filter_map(|p| {
            fs::read_to_string(p).ok().map(|content| {
                (p.to_string_lossy().to_string(), content)
            })
        })
        .collect();

    let project_dir = project.cargo_toml_path.parent()
        .ok_or_else(|| anyhow::anyhow!("Cargo.toml has no parent directory"))?;

    // Read build.rs content if it exists
    let build_rs_path = project_dir.join("build.rs");
    let build_files: Vec<(String, String)> = if build_rs_path.exists() {
        match fs::read_to_string(&build_rs_path) {
            Ok(content) => vec![(build_rs_path.to_string_lossy().to_string(), content)],
            Err(_) => vec![],
        }
    } else {
        vec![]
    };

    analyze_unused_dependencies_with_context(
        &project.cargo_toml_path,
        &src_contents,
        &test_contents,
        &build_files,
    )
}

/// Generate task files for all files with violations (SYNC version)
fn generate_task_files_sync(
    results: &[FileAnalysisResult],
    project_name: &str,
    cargo_toml_path: &std::path::Path,
    orphaned_methods: &std::collections::HashMap<String, Vec<OrphanedMethod>>,
    unused_deps: &[UnusedDependency],
    config: &Config,
) -> anyhow::Result<usize> {
    let mut count = 0;

    // First, emit per-source-file task files WITHOUT unused dependencies
    for result in results {
        if should_generate_task_file_sync(result, orphaned_methods) {
            generate_single_task_file_sync(
                result,
                project_name,
                orphaned_methods,
                &[], // do not attach unused deps to .rs task files
                config,
            )?;
            count += 1;
        }
    }

    // Then, if there are unused dependencies, emit ONE dedicated Cargo.toml task
    if !unused_deps.is_empty() {
        generate_cargo_toml_task_sync(
            project_name,
            cargo_toml_path,
            unused_deps,
            config,
        )?;
        count += 1;
    }

    Ok(count)
}

/// Check if file has violations worth reporting (SYNC version)
fn should_generate_task_file_sync(
    result: &FileAnalysisResult,
    orphaned_methods: &std::collections::HashMap<String, Vec<OrphanedMethod>>,
) -> bool {
    let file_path_str = result.file_path.to_string_lossy().to_string();

    !result.tier1_violations.is_empty()
        || !result.tier2_violations.is_empty()
        || !result.tier3_violations.is_empty()
        || !result.panic_patterns.is_empty()
        || !result.tests_in_src.is_empty()
        || result.lines_of_code > 300
        || orphaned_methods.contains_key(&file_path_str)
}

/// Generate a single task file (SYNC version)
fn generate_single_task_file_sync(
    result: &FileAnalysisResult,
    project_name: &str,
    orphaned_methods: &std::collections::HashMap<String, Vec<OrphanedMethod>>,
    unused_deps: &[UnusedDependency],
    config: &Config,
) -> anyhow::Result<()> {
    use std::path::Path;
    use std::env;

    let file_path_str = result.file_path.to_string_lossy().to_string();

    let file_orphaned_methods = orphaned_methods
        .get(&file_path_str)
        .cloned()
        .unwrap_or_default();

    let context_builder = ContextBuilder {
        project_name: project_name.to_string(),
        file_path: result.file_path.clone(),
        absolute_path: env::current_dir()?,
    };

    let violations = ViolationData {
        tier1_violations: result.tier1_violations.clone(),
        tier2_violations: result.tier2_violations.clone(),
        tier3_violations: result.tier3_violations.clone(),
        panic_patterns: result.panic_patterns.clone(),
        tests_in_src: result.tests_in_src.clone(),
        orphaned_modules: vec![],
        orphaned_methods: file_orphaned_methods,
        // Do not include unused dependencies in .rs task files
        unused_dependencies: unused_deps.to_vec(),
    };

    let context = context_builder.build(violations, result.lines_of_code);

    let tier = context.highest_tier();
    let file_hash = context.file_hash.clone();

    let template_dir = Path::new("./prompt");
    let rendered = render_task_file(context, template_dir)?;

    let file_name = result
        .file_path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("File path has no stem"))?
        .to_string_lossy()
        .to_string();

    write_task_file(
        &rendered,
        &config.output_dir,
        project_name,
        &file_name,
        &file_hash,
        tier,
    )?;

    Ok(())
}
