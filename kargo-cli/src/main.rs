use anyhow::Result;
use std::time::Instant;

mod builtin;
mod cli;
mod plugins;

use cli::{build_root_cli, dispatch};
use plugins::manager::PluginManager;

// PERFORMANCE NOTE: Double CLI Build Pattern
//
// This function builds the CLI twice at startup. This is intentional and
// the cost (<2ms) is negligible compared to plugin discovery I/O (20-2000ms).
//
// Build #1 (line 13): Parse verbosity flags (-v/-q) with empty plugin manager
//   - Configures logging BEFORE plugin discovery
//   - Enables debugging plugin loading with `kargo -vvv plugin list`
//   - Cost: ~0.5-1ms
//
// Build #2 (line 35): Rebuild with discovered plugins
//   - Includes all plugin subcommands in help and dispatch
//   - Cost: ~0.5-1ms
//
// Alternatives considered:
//   A) Manual verbosity parsing: Saves ~1ms but adds complexity and loses clap validation
//   B) Default logging only: Saves ~1ms but loses debuggability of plugin issues
//
// Decision: Accept the negligible cost to preserve simplicity and debuggability.
// See: task/PLUG10.md for full analysis

#[tokio::main]
async fn main() -> Result<()> {
    let startup_time = Instant::now();

    // Parse args first to check verbosity before plugin discovery
    let build1_start = Instant::now();
    let app = build_root_cli(&PluginManager::new());
    let matches = app.get_matches_from(std::env::args_os());
    let build1_elapsed = build1_start.elapsed();

    // Initialize env_logger with appropriate level based on flags
    let log_level = if matches.get_flag("quiet") {
        "off"
    } else {
        match matches.get_count("verbose") {
            0 => "off",   // Default: silent
            1 => "error", // -v
            2 => "warn",  // -vv
            3 => "info",  // -vvv
            4 => "debug", // -vvvv
            _ => "trace", // -vvvvv+
        }
    };

    unsafe {
        std::env::set_var("RUST_LOG", log_level);
    }
    env_logger::init();

    // Now discover plugins with logging configured
    let discovery_start = Instant::now();
    let mut pm = PluginManager::new();
    pm.discover_and_load_plugins()?;
    let discovery_elapsed = discovery_start.elapsed();

    // Rebuild CLI with discovered plugins
    let build2_start = Instant::now();
    let app = build_root_cli(&pm);
    let matches = app.get_matches_from(std::env::args_os());
    let build2_elapsed = build2_start.elapsed();

    let total_elapsed = startup_time.elapsed();

    // Log timing breakdown at debug level
    log::debug!(
        "Startup timing - Total: {:?} | CLI build #1: {:?} | Plugin discovery: {:?} | CLI build #2: {:?}",
        total_elapsed,
        build1_elapsed,
        discovery_elapsed,
        build2_elapsed
    );

    // Calculate and log percentages
    let build_total = build1_elapsed + build2_elapsed;
    let build_percentage = (build_total.as_micros() as f64 / total_elapsed.as_micros() as f64) * 100.0;
    log::debug!(
        "CLI builds represent {:.2}% of total startup time",
        build_percentage
    );

    dispatch(&pm, &matches).await
}
