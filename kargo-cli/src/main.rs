use anyhow::Result;
use env_logger;
use krater::cli::{handle_command, parse_args};
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    info!("Starting Krater - Rust package manager and documentation tool");

    // Parse command line arguments
    let cli = parse_args();

    // Handle the command
    handle_command(cli).await
}
