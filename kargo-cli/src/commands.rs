use crate::events::{Event, EventBus};
use anyhow::Result;
use futures::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;

/// A future that runs a series of shell commands
pub struct CommandExecution<'a> {
    runner: &'a CommandRunner,
    commands: Vec<String>,
    working_dir: PathBuf,
}

impl<'a> Future for CommandExecution<'a> {
    type Output = Result<()>;

    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // This implementation executes commands synchronously but returns a Future
        // that can be awaited. In a real implementation, you might want to make the
        // actual command execution asynchronous as well.
        let this = self.get_mut();

        for cmd in &this.commands {
            this.runner.events.publish(Event::CommandStarted {
                command: cmd.clone(),
            });

            let parts: Vec<_> = cmd.split_whitespace().collect();
            let program = parts[0];
            let args = &parts[1..];

            let output = match Command::new(program)
                .args(args)
                .current_dir(&this.working_dir)
                .output()
            {
                Ok(out) => out,
                Err(e) => {
                    return std::task::Poll::Ready(Err(anyhow::anyhow!(
                        "Failed to execute command {}: {}",
                        cmd,
                        e
                    )));
                }
            };

            let success = output.status.success();
            this.runner.events.publish(Event::CommandFinished {
                command: cmd.clone(),
                success,
            });

            if !success {
                return std::task::Poll::Ready(Err(anyhow::anyhow!(
                    "Command failed: {}\nStderr: {}",
                    cmd,
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }

        std::task::Poll::Ready(Ok(()))
    }
}

pub struct CommandRunner {
    events: EventBus,
}

impl CommandRunner {
    pub fn new(events: EventBus) -> Self {
        Self { events }
    }

    /// Runs a series of shell commands in the specified directory.
    /// Returns a Future that can be awaited to execute the commands.
    pub fn run_commands<'a>(
        &'a self,
        commands: &[String],
        working_dir: &Path,
    ) -> CommandExecution<'a> {
        CommandExecution {
            runner: self,
            commands: commands.to_vec(),
            working_dir: working_dir.to_path_buf(),
        }
    }
}
