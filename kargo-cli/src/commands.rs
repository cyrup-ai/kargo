use crate::events::{Event, EventBus};
use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

/// Internal async implementation for running commands sequentially
async fn run_commands_impl(
    events: &EventBus,
    commands: &[String],
    working_dir: &Path,
) -> Result<()> {
    for cmd in commands {
        events.publish(Event::CommandStarted {
            command: cmd.clone(),
        });

        let parts: Vec<_> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let program = parts[0];
        let args = &parts[1..];

        let output = Command::new(program)
            .args(args)
            .current_dir(working_dir)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute command {}: {}", cmd, e))?;

        let success = output.status.success();
        events.publish(Event::CommandFinished {
            command: cmd.clone(),
            success,
        });

        if !success {
            anyhow::bail!(
                "Command failed: {}\nStderr: {}",
                cmd,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    Ok(())
}

pub struct CommandRunner {
    events: EventBus,
}

impl CommandRunner {
    pub fn new(events: EventBus) -> Self {
        Self { events }
    }

    /// Runs a series of shell commands in the specified directory.
    /// Returns a Future that executes commands asynchronously.
    pub fn run_commands<'a>(
        &'a self,
        commands: &'a [String],
        working_dir: &'a Path,
    ) -> impl std::future::Future<Output = Result<()>> + 'a {
        run_commands_impl(&self.events, commands, working_dir)
    }
}
