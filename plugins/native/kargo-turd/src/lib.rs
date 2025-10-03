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
/// - plugin.rs in async context via tokio::task::spawn_blocking
/// - watch.rs action handler (sync context)
///
/// NOTE: This is sync because all the heavy lifting is rayon-based
/// (AnalysisExecutor uses rayon for parallel file processing)
pub fn run_analysis_sync(config: &Config) -> anyhow::Result<()> {
    use std::env;
    use log::info;

    println!("🔍 Scanning for Rust projects...");
    info!("Starting kargo-turd analysis");

    // Step 1: Find all Rust projects
    let current_dir = env::current_dir()?;
    info!("Working directory: {:?}", current_dir);

    let projects = find_projects_with_progress(&current_dir)?;

    if projects.is_empty() {
        println!("No Rust projects found.");
        info!("No projects found in {:?}", current_dir);
        return Ok(());
    }

    println!("Found {} project(s)\n", projects.len());
    info!("Discovered {} project(s)", projects.len());

    // Step 2: Analyze each project
    let mut success_count = 0;
    let mut failure_count = 0;

    for project in projects {
        match analyze_project_sync(&project, config) {
            Ok(_) => {
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
    info!("Analysis finished: {} succeeded, {} failed", success_count, failure_count);

    if failure_count > 0 {
        println!("⚠️  {} project(s) failed to analyze", failure_count);
    }

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
        orphaned_methods.values().map(|v| v.len()).sum::<usize>(),
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
        &orphaned_methods,
        &unused_deps,
        config,
    )?;

    println!("  ✅ Generated {} task file(s)\n", task_files_generated);

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
    let has_build_rs = project_dir.join("build.rs").exists();

    analyze_unused_dependencies_with_context(
        &project.cargo_toml_path,
        &src_contents,
        &test_contents,
        has_build_rs,
    )
}

/// Generate task files for all files with violations (SYNC version)
fn generate_task_files_sync(
    results: &[FileAnalysisResult],
    project_name: &str,
    orphaned_methods: &std::collections::HashMap<String, Vec<OrphanedMethod>>,
    unused_deps: &[UnusedDependency],
    config: &Config,
) -> anyhow::Result<usize> {
    let mut count = 0;

    for result in results {
        if should_generate_task_file_sync(result, orphaned_methods) {
            generate_single_task_file_sync(
                result,
                project_name,
                orphaned_methods,
                unused_deps,
                config,
            )?;
            count += 1;
        }
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
