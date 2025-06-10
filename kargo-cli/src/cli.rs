use crate::events::EventBus;
use crate::kargo::KargoExecutor;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Krater: A Rust dependency management tool
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Update dependencies to their latest versions
    Update(UpdateArgs),

    /// Run cargo commands with LLM-friendly output processing
    Kargo(KargoArgs),
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Path to update (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct KargoArgs {
    /// Cargo subcommand to run (e.g., build, test, check)
    pub subcommand: String,

    /// Arguments to pass to the cargo subcommand
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,

    /// Path to run the command in (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Run command asynchronously with streaming output
    #[arg(short, long)]
    pub async_mode: bool,
}

/// Parse command line arguments
pub fn parse_args() -> Cli {
    Cli::parse()
}

/// Handle the CLI commands
pub async fn handle_command(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Update(args)) => {
            println!("Updating dependencies in {}", args.path.display());
            update_dependencies(&args).await
        }
        Some(Commands::Kargo(args)) => {
            println!("Running cargo {} with LLM-friendly output", args.subcommand);
            run_kargo_command(&args).await
        }
        None => {
            // No command provided, show help
            println!("No command provided. Run with --help to see available commands.");
            Ok(())
        }
    }
}

/// Run a cargo command with LLM-friendly output processing
async fn run_kargo_command(args: &KargoArgs) -> Result<()> {
    // Create event bus for progress notifications
    let events = EventBus::new();

    // Create the kargo executor
    let executor = KargoExecutor::new(events.clone());

    // Create a vector with the subcommand and all arguments
    let mut cargo_args = vec![args.subcommand.clone()];
    cargo_args.extend(args.args.clone());

    // Run the command either synchronously or asynchronously
    if args.async_mode {
        executor.run_async(&cargo_args, &args.path).await?;
    } else {
        let output = executor.run_sync(&cargo_args, &args.path)?;
        println!("{}", output);
    }

    Ok(())
}

/// Update dependencies
async fn update_dependencies(args: &UpdateArgs) -> Result<()> {
    use crate::events::EventBus;
    use crate::up2date::{coordinator::start_update, types::UpdateOptions};

    println!("Updating dependencies in {}", args.path.display());

    // Create event bus for progress notifications
    let events = EventBus::new();

    // Subscribe to events for CLI output
    let mut rx = events.subscribe();

    // Spawn a task to handle events
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                crate::events::Event::ScanStarted { dirs } => {
                    println!("Scanning directories: {:?}", dirs);
                }
                crate::events::Event::CargoTomlFound { path } => {
                    println!("Found Cargo.toml: {}", path.display());
                }
                crate::events::Event::RustScriptFound { path } => {
                    println!("Found Rust script: {}", path.display());
                }
                crate::events::Event::DependencyUpdated { path, from, to } => {
                    println!(
                        "Updated dependency in {}: {} -> {}",
                        path.display(),
                        from,
                        to
                    );
                }
                crate::events::Event::Error { message } => {
                    eprintln!("Error: {}", message);
                }
                _ => {} // Ignore other events
            }
        }
    });

    // Use default update options
    let options = UpdateOptions::default();

    // Start the update process
    let mut session = start_update(args.path.clone(), options, events);

    // Process results as they arrive
    let mut update_count = 0;
    while let Some(result) = session.watch().next().await {
        if !result.updates.is_empty() {
            update_count += result.updates.len();
        }

        if let Some(error) = result.error {
            eprintln!("Error updating {}: {}", result.path.display(), error);
        }
    }

    println!("Update complete. Updated {} dependencies.", update_count);
    Ok(())
}
