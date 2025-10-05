#![allow(unsafe_code)]
use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use crate::{install, remove, list};

pub struct PluginManagementPlugin;

impl PluginCommand for PluginManagementPlugin {
    fn clap(&self) -> Command {
        Command::new("plugin")
            .about("Manage kargo plugins")
            .subcommand_required(true)
            .subcommand(
                Command::new("install")
                    .about("Install a plugin from GitHub or local path")
                    .arg(Arg::new("source")
                        .help("GitHub URL, org/repo, or local path")
                        .required(true)
                        .index(1))
                    .arg(Arg::new("branch")
                        .long("branch")
                        .short('b')
                        .help("Git branch to use (default: main)")
                        .value_name("BRANCH"))
            )
            .subcommand(
                Command::new("remove")
                    .about("Remove an installed plugin")
                    .arg(Arg::new("source")
                        .help("GitHub URL or org/repo identifier")
                        .required(true)
                        .index(1))
            )
            .subcommand(
                Command::new("list")
                    .about("List installed plugins")
                    .arg(Arg::new("remote")
                        .long("remote")
                        .help("Scan plugins in a remote repository")
                        .value_name("URL"))
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        let cmd = self.clap();
        Box::pin(async move {
            let matches = cmd.get_matches_from(&ctx.matched_args);

            match matches.subcommand() {
                Some(("install", sub_m)) => {
                    let source = sub_m.get_one::<String>("source")
                        .ok_or_else(|| anyhow::anyhow!("Source argument is required"))?;
                    let branch = sub_m.get_one::<String>("branch");
                    install::install_plugin(source, branch).await?;
                }
                Some(("remove", sub_m)) => {
                    let source = sub_m.get_one::<String>("source")
                        .ok_or_else(|| anyhow::anyhow!("Source argument is required"))?;
                    remove::remove_plugin(source)?;
                }
                Some(("list", sub_m)) => {
                    let remote = sub_m.get_one::<String>("remote");
                    list::list_plugins(remote).await?;
                }
                _ => unreachable!(),
            }

            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(PluginManagementPlugin)
}
