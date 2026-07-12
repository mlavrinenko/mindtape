//! Command-line surface: argument parsing and dispatch.

use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use log::LevelFilter;

use crate::render::color::ColorChoice;
use crate::{
    add, check, cli_dump, config_cmd, eval, flip, go, graph, import, init, install, ls, recur,
    review, rm, views, worktrees,
};

/// `MindTape`: a file-based task tracker using per-file Typst artifacts.
#[derive(Parser)]
#[command(name = "mt", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Operate on the `MindTape` project at `<PATH>` instead of discovering one
    /// by walking up from the current directory.
    ///
    /// The path must contain a `.mindtape/` directory. Relative paths resolve
    /// against the current directory. Falls back to `$MINDTAPE_WORKTREE` when
    /// unset; the flag wins over the env var.
    #[arg(short = 'w', long, value_name = "PATH", global = true)]
    pub worktree: Option<PathBuf>,

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
    /// File a new task from a Typst slot template.
    Add(add::AddArgs),
    /// Import canonical task-JSON (a file arg or stdin `-`) as native `.typ`
    /// tasks: map onto this project's config policy, emit, validate the whole
    /// batch, and report. See `mt ls --json` for the schema this reads.
    Import(import::ImportArgs),
    /// List tasks under the configured roots, applying a filter.
    Ls(ls::LsArgs),
    /// Emit the task dependency graph as DOT or Mermaid, scoped by a filter.
    Graph(graph::GraphArgs),
    /// List or inspect named views (`[views]`).
    Views(views::ViewsArgs),
    /// List worktree aliases (`@name` sugar; `~/.config/mindtape/worktrees.toml`).
    Worktrees(worktrees::WorktreesArgs),
    /// Validate tasks against the project configuration.
    Check,
    /// Inspect the effective merged configuration (`show` is the default action).
    Config(config_cmd::ConfigArgs),
    /// Scaffold `.mindtape/config.toml` in the current directory.
    Init(init::InitArgs),
    /// Set up the user environment: the `@local/mindtape` Typst package and
    /// the XDG global `defaults.toml` app-config layer.
    ///
    /// Performs both by default; `--no-lib`/`--no-defaults` skip either half.
    Install(install::InstallArgs),
    /// Install `@local/mindtape` into the user-local Typst packages directory.
    ///
    /// Deprecated: superseded by `mt install`, which does this too and also
    /// seeds the XDG global defaults. Equivalent to `mt install --no-defaults`;
    /// kept hidden as an alias for existing docs and muscle memory.
    #[command(hide = true)]
    InstallLib,
    /// Evaluate a single task file and print it.
    Eval(eval::EvalArgs),
    /// Open a task in the configured editor (`[go] editor`).
    Go(go::GoArgs),
    /// Retire tasks (by ref, filter, or the default), rewriting inbound links to
    /// SWHID tombstones.
    Rm(rm::RmArgs),
    /// Stamp `reviewed:` with today's date on a task.
    Review(review::ReviewArgs),
    /// Set a task's status (`flip <status> <task>...`); each legend value is a verb.
    ///
    /// Stamps `<status>` dated today onto each task; `<status>` must be a
    /// configured legend value. Every legend value also works as a top-level
    /// shortcut verb — `mt done 1b 2c` == `mt flip done 1b 2c` —
    /// generated from the legend, so they do not show in `--help`; run
    /// `mt flip --list` to see them. Built-in subcommands win on a name
    /// clash. Targets are resolved before any write, then valid edits apply and
    /// any resolution or validation problem is reported (the rest still land).
    Flip(flip::FlipArgs),
    /// Advance a recurring task to its next occurrence — the only command
    /// that spawns or mutates a recurrence (see `[tasks.recur]`).
    ///
    /// Dispatches on the task's effective `materialize`: `spawn` (the
    /// default) writes the next instance as a new file and links it back via
    /// `recurred-from`; `mutate` refreshes the same file in place. `--dry-run`
    /// previews without writing; `--advance N` pre-materializes N future
    /// spawns without closing the current instance; `--skip` advances the
    /// anchor/`count` without materializing anything. `mt done` never
    /// advances a recurrence — it stays a pure close.
    Recur(recur::RecurArgs),
    /// Dump the command tree as JSON for the docs generator (hidden;
    /// reflects `Cli::command()`, so the rendered CLI reference cannot drift).
    #[command(name = "_cli", hide = true)]
    CliDump(cli_dump::CliDumpArgs),
    /// Per-status shortcut verb (`<status> <task>...`), dispatched to `flip`.
    #[command(external_subcommand)]
    Status(Vec<String>),
}

impl Cli {
    /// Dispatch to the selected command.
    ///
    /// # Errors
    /// Propagates the command's failure.
    pub fn run(self) -> Result<()> {
        match self.command {
            Command::Add(args) => add::run(&args),
            Command::Import(args) => import::run(&args),
            Command::Ls(args) => ls::run(&args),
            Command::Graph(args) => graph::run(&args),
            Command::Views(args) => views::run(&args),
            Command::Worktrees(args) => worktrees::run(&args),
            Command::Check => check::run(),
            Command::Config(args) => config_cmd::run(&args),
            Command::Init(args) => init::run(&args),
            Command::Install(args) => install::run(&args),
            Command::InstallLib => init::run_install_lib(),
            Command::Eval(args) => eval::run(&args),
            Command::Go(args) => go::run(&args),
            Command::Rm(args) => rm::run(&args),
            Command::Review(args) => review::run(&args),
            Command::Flip(args) => flip::run(&args),
            Command::Recur(args) => recur::run(&args),
            Command::CliDump(args) => cli_dump::run(&args),
            Command::Status(argv) => flip::run_external(&argv),
        }
    }
}

/// Built-in subcommand names, for detecting legend values that shadow them.
pub fn builtin_subcommand_names() -> Vec<String> {
    Cli::command()
        .get_subcommands()
        .map(|cmd| cmd.get_name().to_owned())
        .collect()
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
        let cli = Cli::try_parse_from(["mt", "--color", "always", "check"]).unwrap();
        assert_eq!(cli.color, ColorChoice::Always);
    }

    #[test]
    fn color_flag_after_subcommand() {
        let cli = Cli::try_parse_from(["mt", "check", "--color", "never"]).unwrap();
        assert_eq!(cli.color, ColorChoice::Never);
    }

    #[test]
    fn color_default_is_auto() {
        let cli = Cli::try_parse_from(["mt", "check"]).unwrap();
        assert_eq!(cli.color, ColorChoice::Auto);
    }
}
