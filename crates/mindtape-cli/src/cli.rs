//! Command-line surface: argument parsing and dispatch.

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::LevelFilter;

use crate::{check, eval, init, list};

/// `MindTape`: a file-based task tracker using per-file Typst artifacts.
#[derive(Parser)]
#[command(name = "mindtape", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Increase verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand)]
enum Command {
    /// List tasks under the configured roots, applying a filter.
    List(list::ListArgs),
    /// Validate tasks against the project configuration.
    Check,
    /// Scaffold `.mindtape/config.toml` in the current directory.
    Init(init::InitArgs),
    /// Install `@local/mindtape` into the user-local Typst packages directory.
    InstallLib,
    /// Evaluate a single task file and print it.
    Eval(eval::EvalArgs),
}

impl Cli {
    /// Dispatch to the selected command.
    ///
    /// # Errors
    /// Propagates the command's failure.
    pub fn run(self) -> Result<()> {
        match self.command {
            Command::List(args) => list::run(&args),
            Command::Check => check::run(),
            Command::Init(args) => init::run(&args),
            Command::InstallLib => init::run_install_lib(),
            Command::Eval(args) => eval::run(&args),
        }
    }
}

/// Initialize logging from the `-v` count; `RUST_LOG` overrides.
pub fn init_logger(verbosity: u8) {
    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(LevelFilter::Warn)
        .filter_module("mindtape", level)
        .filter_module("mindtape_cli", level)
        .filter_module("typst_harvest", level)
        .format_target(false)
        .format_timestamp(None)
        .init();
}
