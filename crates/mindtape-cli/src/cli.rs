//! Command-line surface: argument parsing and dispatch.

use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use log::LevelFilter;

use crate::render::color::ColorChoice;
use crate::render::link::HyperlinkChoice;
use crate::{
    add, check, cli_dump, config_cmd, eval, find, flip, go, graph, import, init, install, link_cmd,
    ls, preset_cmd, recur, review, rm, views, worktrees,
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

    /// Hyperlink output for the `file` column: auto, always, or never.
    ///
    /// `auto` emits OSC 8 hyperlink sequences only when stdout is a TTY and
    /// `TERM` is not `dumb` — a heuristic guess, since no terminal capability
    /// query exists for OSC 8 support. Wins over `[ls] hyperlinks` when
    /// passed; omitted defers to that config key, which itself defaults to
    /// `never` — unlike colour, which just degrades to nothing, an
    /// unsupported terminal can render the OSC 8 payload as visible garbage,
    /// so opting in stays explicit rather than automatic.
    #[arg(long, value_name = "WHEN", global = true)]
    pub hyperlinks: Option<HyperlinkChoice>,
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
    /// Search by text — sugar over `contains()`: `mt find login bug` is
    /// `mt ls 'contains("login") && contains("bug")'`. Every `ls` flag
    /// forwards verbatim; no `--in`, field targeting stays in the filter
    /// language.
    Find(find::FindArgs),
    /// Emit the task dependency graph as DOT or Mermaid, scoped by a filter.
    Graph(graph::GraphArgs),
    /// List or inspect named views (`[views]`).
    Views(views::ViewsArgs),
    /// List worktree aliases (`@name` sugar; `~/.config/mindtape/worktrees.toml`).
    Worktrees(worktrees::WorktreesArgs),
    /// Validate tasks against the project configuration.
    Check(check::CheckArgs),
    /// Inspect the effective merged configuration (`show` is the default action).
    Config(config_cmd::ConfigArgs),
    /// Save or list named `mt init` starter shapes: `save <name>` captures the
    /// current project's config and templates as a reusable user preset,
    /// `ls` lists built-in and user presets together.
    Preset(preset_cmd::PresetArgs),
    /// Scaffold `.mindtape/config.toml` in the current directory.
    Init(init::InitArgs),
    /// Set up the user environment: the `@local/mindtape` Typst package, the
    /// XDG global `defaults.toml` app-config layer, and the JSON Schema for
    /// `.mindtape/config.toml`.
    ///
    /// Performs all three by default;
    /// `--no-lib`/`--no-defaults`/`--no-schema` skip any of them.
    Install(install::InstallArgs),
    /// Install `@local/mindtape` into the user-local Typst packages directory.
    ///
    /// Deprecated: superseded by `mt install`, which does this too and also
    /// seeds the XDG global defaults and the JSON Schema. Equivalent to
    /// `mt install --no-defaults --no-schema`; kept hidden as an alias for
    /// existing docs and muscle memory.
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
    /// Write typed `links:` edges: one at a time, or a whole adjacency map.
    ///
    /// `mt link <task> <kind> <target>...` upserts one edge per target;
    /// `mt link --apply <file>` (`-` for stdin) reads a YAML/JSON mapping of
    /// stem to kind to targets and merges every entry. Merging dedupes on
    /// (kind, target), so re-running either form writes nothing the second
    /// time; `--replace` makes the map the complete edge set instead.
    /// `--dry-run` prints the plan without writing. A per-entry failure is
    /// reported and the rest still land.
    Link(link_cmd::LinkArgs),
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
            Command::Find(args) => find::run(&args),
            Command::Graph(args) => graph::run(&args),
            Command::Views(args) => views::run(&args),
            Command::Worktrees(args) => worktrees::run(&args),
            Command::Check(args) => check::run(&args),
            Command::Config(args) => config_cmd::run(&args),
            Command::Preset(args) => preset_cmd::run(&args),
            Command::Init(args) => init::run(&args),
            Command::Install(args) => install::run(&args),
            Command::InstallLib => init::run_install_lib(),
            Command::Eval(args) => eval::run(&args),
            Command::Go(args) => go::run(&args),
            Command::Rm(args) => rm::run(&args),
            Command::Review(args) => review::run(&args),
            Command::Flip(args) => flip::run(&args),
            Command::Recur(args) => recur::run(&args),
            Command::Link(args) => link_cmd::run(&args),
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

/// Resolve a `-v` count to its log level.
#[must_use]
pub fn verbosity_level(verbosity: u8) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

/// Initialize logging from the `-v` count; `RUST_LOG` overrides.
pub fn init_logger(verbosity: u8) {
    let level = verbosity_level(verbosity);
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
    use log::LevelFilter;

    use super::{Cli, builtin_subcommand_names, init_logger, verbosity_level};
    use crate::render::color::ColorChoice;
    use crate::render::link::HyperlinkChoice;

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

    #[test]
    fn hyperlinks_flag_parses() {
        let cli = Cli::try_parse_from(["mt", "--hyperlinks", "always", "check"]).unwrap();
        assert_eq!(cli.hyperlinks, Some(HyperlinkChoice::Always));
    }

    #[test]
    fn hyperlinks_flag_absent_defers_to_config() {
        // Unlike `--color`, an absent `--hyperlinks` is `None`, not a default
        // value — the flag must not shadow `[ls] hyperlinks` when unpassed.
        let cli = Cli::try_parse_from(["mt", "check"]).unwrap();
        assert_eq!(cli.hyperlinks, None);
    }

    /// `check` is a built-in, which also proves the list is non-empty — a
    /// separate emptiness test asserted strictly less than this one.
    #[test]
    fn builtin_names_contains_check() {
        let names = builtin_subcommand_names();
        assert!(
            names.contains(&"check".to_owned()),
            "check must be in builtins: {names:?}"
        );
    }

    /// The whole `-v` ladder, including both ends of the saturating `_ =>` arm.
    #[test]
    fn verbosity_ladder_maps_each_count_to_its_level() {
        let cases = [
            (0, LevelFilter::Warn),
            (1, LevelFilter::Info),
            (2, LevelFilter::Debug),
            (3, LevelFilter::Trace),
            (255, LevelFilter::Trace),
        ];
        for (count, expected) in cases {
            assert_eq!(verbosity_level(count), expected, "-v x{count}");
        }
    }

    // `init_logger` installs the `log` crate's process-global logger exactly
    // once, so its only observable effect — the global max level — needs a
    // fresh process to assert. Re-exec the test binary for this one test name.

    const CHILD: &str = "_MT_CLI_CHILD";

    #[test]
    fn init_logger_raises_global_max_level() {
        if std::env::var(CHILD).is_err() {
            let out = crate::test_util::run_self_test("init_logger_raises_global_max_level", CHILD);
            crate::test_util::assert_child_ok(&out);
            return;
        }
        assert_eq!(
            log::max_level(),
            LevelFilter::Off,
            "no logger installed before init_logger"
        );
        init_logger(2);
        assert_eq!(
            log::max_level(),
            LevelFilter::Debug,
            "-vv must install the logger at Debug"
        );
    }
}
