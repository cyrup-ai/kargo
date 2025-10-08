mod install;
mod remove;
mod list;
mod git;
mod metadata;
mod parser;
mod build;
pub(crate) mod artifact;

use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

pub fn command() -> Command {
    Command::new("plugin")
        .about("Manage kargo plugins")
        .subcommand_required(true)
        .subcommand(
            Command::new("install")
                .about("Install a plugin from GitHub or local path")
                .arg(Arg::new("source")
                    .help("GitHub URL, org/repo, org/repo/plugin, or local path")
                    .required(true)
                    .index(1))
                .arg(Arg::new("branch")
                    .long("branch")
                    .short('b')
                    .help("Git branch to use (default: main)")
                    .value_name("BRANCH"))
                .arg(Arg::new("package")
                    .long("package")
                    .short('p')
                    .help("Plugin package name within the repository (monorepo support)")
                    .value_name("PACKAGE"))
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

pub async fn execute(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("install", sub_m)) => {
            let source = sub_m.get_one::<String>("source")
                .ok_or_else(|| anyhow::anyhow!("Source argument is required"))?;
            let branch = sub_m.get_one::<String>("branch");
            let package = sub_m.get_one::<String>("package");
            install::install_plugin(source, branch, package).await?;
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
}
