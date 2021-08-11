use clap::Clap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use cargo_util::{ProcessBuilder, ProcessError};

mod commands;
mod common;
mod consts;

type CliResult = color_eyre::eyre::Result<()>;

/// near-cli is a toolbox for interacting with NEAR protocol
#[derive(Debug, Clap)]
#[clap(
    version,
    author,
    about,
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands),
    // setting(clap::AppSettings::NextLineHelp)
)]
struct CliArgs {
    #[clap(subcommand)]
    top_level_command: Option<self::commands::CliTopLevelCommand>,
}

#[derive(Debug)]
struct Args {
    top_level_command: self::commands::TopLevelCommand,
}

impl From<CliArgs> for Args {
    fn from(cli_args: CliArgs) -> Self {
        let top_level_command = match cli_args.top_level_command {
            Some(cli_subcommand) => self::commands::TopLevelCommand::from(cli_subcommand),
            None => self::commands::TopLevelCommand::choose_command(),
        };
        Self { top_level_command }
    }
}

impl Args {
    async fn process(self) -> CliResult {
        self.top_level_command.process().await
    }
}

fn main() -> CliResult {
    let cli = CliArgs::parse();

    if let Some(self::commands::CliTopLevelCommand::GenerateShellCompletions(subcommand)) =
        cli.top_level_command
    {
        subcommand.process();
        return Ok(());
    }

    let args = Args::from(cli);

    color_eyre::install()?;

    actix::System::new().block_on(args.process())
}

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
#[cfg(windows)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

fn search_directories(cargo_home_directory: PathBuf) -> Vec<PathBuf> {
    let mut dirs = vec![cargo_home_directory.clone().join("bin")]; //TODO: is this string working?
    if let Some(val) = env::var_os("PATH") {
        dirs.extend(env::split_paths(&val));
    }
    dirs
}
