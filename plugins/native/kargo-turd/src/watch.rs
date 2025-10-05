use miette::IntoDiagnostic;
use watchexec::Watchexec;
use watchexec_signals::Signal;
use crate::Config;

// ============================================================================
// WATCH MODE - Filesystem monitoring with watchexec library
// ============================================================================

/// Run watch mode: continuous analysis on file changes
///
/// Pattern from watchexec examples:
/// - `Watchexec::new()` takes a SYNC closure (action handler)
/// - Action handler is called for every event (file changes, signals)
/// - Must return the action to continue the loop
/// - `wx.main()` is async and runs until quit
///
/// Flow:
/// 1. Create Watchexec with action handler closure
/// 2. Configure paths to watch and file filters
/// 3. Run initial analysis (before watching starts)
/// 4. Send initial event to start the event loop
/// 5. Start main loop (wx.main().await)
/// 6. On each file change event → re-run analysis
/// 7. On SIGINT/SIGTERM → quit gracefully
pub async fn run_watch_mode(config: &Config) -> anyhow::Result<()> {
    let watch_path = config.watch_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("watch_path must be set for watch mode"))?;

    // Clone config for use in action handler closure
    let config_clone = config.clone();

    // Initialize watchexec with runtime context set in TLS
    // We need to call handle.enter() before Watchexec::new() because:
    // 1. Plugin cdylib has separate TLS from kargo-cli
    // 2. Watchexec::new() calls tokio::spawn() which reads runtime handle from TLS
    // 3. The guard must be active on the thread executing this code
    // Citation: ./tmp/watchexec/crates/lib/examples/only_events.rs:6-20
    let wx = {
        let handle = config.runtime_handle
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No tokio runtime handle provided"))?;
        let _guard = handle.enter();

        Watchexec::new(move |mut action| {
        // ===== Signal Handling =====
        // Check for Ctrl+C (SIGINT) or SIGTERM
        // Citation: ./tmp/watchexec/crates/lib/examples/only_events.rs:13-16
        if action.signals().any(|sig| sig == Signal::Interrupt || sig == Signal::Terminate) {
            eprintln!("\n👋 Stopping watch mode...");
            action.quit();
            return action;
        }

        // ===== File Change Handling =====
        // Check if any .rs or .toml files changed
        // Citation: ./tmp/watchexec/crates/lib/src/action/handler.rs (paths() returns Iterator<Item = (&Path, Option<&FileType>)>)
        if action.paths().any(|(p, _)| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ext == "rs" || ext == "toml")
        }) {
            // Clear screen for better readability (ANSI escape codes)
            print!("\x1B[2J\x1B[1;1H");

            // Re-run analysis (SYNC - no .await allowed in action handler)
            eprintln!("🔄 Rust file changes detected, re-running analysis...\n");

            // Call synchronous run_analysis (from lib.rs)
            match crate::run_analysis_sync(&config_clone) {
                Ok(()) => eprintln!("\n✅ Analysis complete. Watching for changes..."),
                Err(e) => {
                    // Don't quit on analysis errors - keep watching
                    // User might have syntax errors they're fixing
                    eprintln!("\n❌ Analysis failed: {e}");
                    eprintln!("Watching for changes...");
                }
            }
        }

            // MUST return action to continue the loop
            action
        })
        .into_diagnostic()
        .map_err(|e| anyhow::anyhow!("Failed to initialize watchexec: {e}"))?
    }; // _guard drops here, after Watchexec::new() completes

    // ===== Configure Paths to Watch =====
    // Citation: ./tmp/watchexec/crates/lib/examples/only_events.rs:26
    if let Some(path_str) = watch_path.to_str() {
        wx.config.pathset([path_str]);
    } else {
        return Err(anyhow::anyhow!("Invalid watch path: non-UTF8 characters"));
    }

    // ===== Run Initial Analysis =====
    // Do this BEFORE starting watch loop so user sees results immediately
    eprintln!("🔍 Running initial analysis...\n");
    crate::run_analysis_sync(config)?;

    eprintln!("\n👀 Watching for changes in {}...", watch_path.display());
    eprintln!("Press Ctrl+C to exit\n");

    // ===== Start Watchexec Event Loop =====
    // Citation: ./tmp/watchexec/crates/lib/examples/only_events.rs:23-28

    // Start the main loop (must be called before send_event)
    let main_loop = wx.main();

    // Run the main loop until quit
    let _ = main_loop.await
        .into_diagnostic()
        .map_err(|e| anyhow::anyhow!("Watch loop failed: {e}"))?;

    Ok(())
}
