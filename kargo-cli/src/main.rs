use anyhow::Result;
use env_logger;
use log::info;

mod cli;
mod plugins;

use cli::{build_root_cli, dispatch};
use plugins::manager::PluginManager;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Kargo Flux runtime");

    let mut pm = PluginManager::new();
    pm.discover_and_load_plugins()?;

    let app = build_root_cli(&pm);
    let matches = app.get_matches();

    dispatch(&pm, &matches).await
}
