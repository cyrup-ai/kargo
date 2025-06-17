use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;

use crate::processor::OutputProcessor;

pub struct KargoExecutor {
    processor: OutputProcessor,
}

impl KargoExecutor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            processor: OutputProcessor::new()?,
        })
    }

    /// Run a cargo command synchronously
    pub fn run_sync(&self, args: &[String], working_dir: &Path) -> Result<String> {
        // Log command start if needed

        let output = Command::new("cargo")
            .args(args)
            .current_dir(working_dir)
            .output()
            .with_context(|| format!("Failed to execute cargo command: {}", args.join(" ")))?;

        let success = output.status.success();
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

        // Process stdout and stderr
        let processed_output = self.processor.process_output(&output_str);

        // Process stderr if there are errors
        if !stderr_str.is_empty() {
            eprintln!("{}", stderr_str);
        }

        // Log command finish if needed

        if !success {
            anyhow::bail!(
                "Cargo command failed: {}.\nStderr: {}",
                args.join(" "),
                stderr_str
            );
        }

        Ok(processed_output)
    }

    /// Run a cargo command asynchronously with streaming output
    pub async fn run_async(&self, args: &[String], working_dir: &Path) -> Result<()> {
        let mut child = AsyncCommand::new("cargo")
            .args(args)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute cargo command: {}", args.join(" ")))?;

        // Process stdout
        if let Some(stdout) = child.stdout.take() {
            let processor = self.processor.clone();
            let mut reader = BufReader::new(stdout).lines();

            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    let processed = processor.process_line(&line);
                    println!("{}", processed);
                }
            });
        }

        // Process stderr
        if let Some(stderr) = child.stderr.take() {
            let processor = self.processor.clone();
            let mut reader = BufReader::new(stderr).lines();

            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    let processed = processor.process_line(&line);
                    eprintln!("{}", processed);
                }
            });
        }

        // Wait for the command to complete
        let status = child.wait().await?;

        // Log command finish if needed

        if !status.success() {
            anyhow::bail!("Cargo command failed: {}", args.join(" "));
        }

        Ok(())
    }
}
