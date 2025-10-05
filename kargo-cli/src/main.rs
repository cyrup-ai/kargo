use anyhow::Result;

mod builtin;
mod cli;
mod plugins;

use cli::{build_root_cli, dispatch};
use plugins::manager::PluginManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse args first to check verbosity before plugin discovery
    let app = build_root_cli(&PluginManager::new());
    let matches = app.get_matches_from(std::env::args_os());

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
    let mut pm = PluginManager::new();
    pm.discover_and_load_plugins()?;

    // Rebuild CLI with discovered plugins
    let app = build_root_cli(&pm);
    let matches = app.get_matches_from(std::env::args_os());

    dispatch(&pm, &matches).await
}
