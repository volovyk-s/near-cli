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
    let cli = match CliArgs::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let args = std::env::args();
            return try_external_subcommand_execution();
        }
    };

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

fn try_external_subcommand_execution() -> CliResult {
    let subcommand_from_args = "TODO";
    let mut ext_args: Vec<&str> = vec![subcommand_from_args];
    //TODO: extend ext_args with all the other args
    let subcommand_exe = format!("near-{}{}", subcommand_from_args, env::consts::EXE_SUFFIX);
    let path = get_path_directories()
        .iter()
        .map(|dir| dir.join(&subcommand_exe))
        .find(|file| is_executable(file));
    let command = match path {
        Some(command) => command,
        None => {
            return Err(color_eyre::eyre::eyre!(
                "command {} does not exist",
                subcommand_exe
            ));
        }
    };

    // let cargo_exe = config.cargo_exe()?;
    let err = match ProcessBuilder::new(&command)
        // .env(cargo::CARGO_ENV, cargo_exe)
        .args(&ext_args)
        .exec_replace()
    {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };

    if let Some(perr) = err.downcast_ref::<ProcessError>() {
        if let Some(code) = perr.code {
            return Err(color_eyre::eyre::eyre!("perror occured, code: {}", code));
        }
    }
    return Err(color_eyre::eyre::eyre!(err));
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

fn get_path_directories() -> Vec<PathBuf> {
    let mut dirs = vec![];
    if let Some(val) = env::var_os("PATH") {
        dirs.extend(env::split_paths(&val));
    }
    dirs
}
