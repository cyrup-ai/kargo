#![allow(unsafe_code)]
use anyhow::Result;
use clap::{Arg, Command, CommandFactory};
use clap_complete::Shell;
use globset::Glob;
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use std::path::PathBuf;

pub struct MdlintPlugin;

impl PluginCommand for MdlintPlugin {
    fn clap(&self) -> Command {
        Command::new("mdlint")
            .about("Lint markdown files using mado")
            .long_about("Fast markdown linter powered by mado - checks markdown files for common issues and style violations")
            .arg(
                Arg::new("files")
                    .help("List of files or directories to check")
                    .value_name("FILES")
                    .num_args(0..)
                    .default_value(".")
            )
            .arg(
                Arg::new("output-format")
                    .long("output-format")
                    .help("Output format for violations")
                    .value_name("FORMAT")
                    .value_parser(["concise", "markdownlint", "mdl"])
            )
            .arg(
                Arg::new("quiet")
                    .long("quiet")
                    .help("Only log errors")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("exclude")
                    .long("exclude")
                    .help("List of file patterns to exclude from linting")
                    .value_name("PATTERNS")
                    .value_delimiter(',')
                    .num_args(0..)
            )
            .arg(
                Arg::new("config")
                    .long("config")
                    .help("Path to TOML configuration file")
                    .value_name("FILE")
            )
            .arg(
                Arg::new("shell")
                    .long("generate-shell-completion")
                    .help("Generate shell completion script")
                    .value_name("SHELL")
                    .value_parser(["bash", "fish", "zsh", "powershell", "elvish"])
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        let cmd = self.clap();
        Box::pin(async move {
            let matches = cmd.get_matches_from(&ctx.matched_args);

            // Check if shell completion was requested
            if let Some(shell_str) = matches.get_one::<String>("shell") {
                let shell = match shell_str.as_str() {
                    "bash" => Shell::Bash,
                    "fish" => Shell::Fish,
                    "zsh" => Shell::Zsh,
                    "powershell" => Shell::PowerShell,
                    "elvish" => Shell::Elvish,
                    _ => return Err(anyhow::anyhow!("Unsupported shell: {}", shell_str)),
                };

                let mado_cmd = mado::Cli::command();
                let mut generator = mado::command::generate_shell_completion::ShellCompletionGenerator::new(mado_cmd);
                generator.generate(shell);
                return Ok(());
            }

            // Collect files
            let files: Vec<PathBuf> = matches
                .get_many::<String>("files")
                .unwrap_or_default()
                .map(PathBuf::from)
                .collect();

            // Parse output format - let mado handle the format parsing
            let output_format = None; // Will use mado's default format handling

            let quiet = matches.get_flag("quiet");

            // Parse exclude patterns
            let exclude = matches
                .get_many::<String>("exclude")
                .map(|values| {
                    values
                        .map(|pattern| {
                            Glob::new(pattern)
                                .map_err(|e| anyhow::anyhow!("Invalid glob pattern '{}': {}", pattern, e))
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .transpose()?;

            let config_path = matches.get_one::<String>("config").map(PathBuf::from);

            // Create mado options
            let options = mado::command::check::Options {
                output_format,
                config_path,
                quiet,
                exclude,
            };

            // Convert to mado config
            let config = options.to_config().map_err(|e| anyhow::anyhow!("Config error: {}", e))?;

            // Create checker and run
            let checker = mado::command::check::Checker::new(&files, config)
                .map_err(|e| anyhow::anyhow!("Checker creation error: {}", e))?;
            let exit_code = checker.check()
                .map_err(|e| anyhow::anyhow!("Check error: {}", e))?;

            // Convert ExitCode to Result for plugin API
            match exit_code {
                std::process::ExitCode::SUCCESS => Ok(()),
                _ => Err(anyhow::anyhow!("Linting found issues")),
            }
        })
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
#[allow(unsafe_code)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(MdlintPlugin)
}