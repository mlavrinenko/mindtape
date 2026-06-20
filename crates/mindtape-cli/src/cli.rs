//! Command-line surface: argument parsing and dispatch.

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::LevelFilter;

use crate::render::color::ColorChoice;
use crate::{check, eval, init, list, review, rm};

/// `MindTape`: a file-based task tracker using per-file Typst artifacts.
#[derive(Parser)]
#[command(name = "mindtape", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Increase verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Colour output: auto (default), always, or never.
    ///
    /// `auto` enables colour only when stdout is a TTY and `NO_COLOR` is
    /// unset. `always` forces colour regardless of TTY or `NO_COLOR`.
    /// `never` suppresses all ANSI colour sequences.
    #[arg(long, value_name = "WHEN", default_value_t, global = true)]
    pub color: ColorChoice,
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
    /// Retire tasks (by ref, filter, or the default), rewriting inbound links to
    /// SWHID tombstones.
    Rm(rm::RmArgs),
    /// Stamp `reviewed:` with today's date on a task.
    Review(review::ReviewArgs),
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
            Command::Rm(args) => rm::run(&args),
            Command::Review(args) => review::run(&args),
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use clap::Parser;

    use super::Cli;
    use crate::render::color::ColorChoice;

    #[test]
    fn color_flag_before_subcommand() {
        let cli = Cli::try_parse_from(["mindtape", "--color", "always", "check"]).unwrap();
        assert_eq!(cli.color, ColorChoice::Always);
    }

    #[test]
    fn color_flag_after_subcommand() {
        let cli = Cli::try_parse_from(["mindtape", "check", "--color", "never"]).unwrap();
        assert_eq!(cli.color, ColorChoice::Never);
    }

    #[test]
    fn color_default_is_auto() {
        let cli = Cli::try_parse_from(["mindtape", "check"]).unwrap();
        assert_eq!(cli.color, ColorChoice::Auto);
    }
}
