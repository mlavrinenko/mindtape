//! Command-line surface: argument parsing and dispatch.
//!
//! # Files
//!
//! - `check_argv.rs` — `mt _cli --check-argv`, which judges one command line as `mt` would parse it.
//! - `command_class.rs` — each command's read or write class.
//! - `completion/` — shell completions answered by the binary.
//! - `task_ref.rs` — the `<task>` argument every single-task command shares.
//! - `tree_json.rs` — `mt _cli`, the command tree as JSON for the docs.

pub(crate) mod check_argv;
pub(crate) mod command_class;
pub(crate) mod completion;
pub(crate) mod task_ref;
pub(crate) mod tree_json;

use std::path::PathBuf;

use crate::diag::Result;
use clap::{CommandFactory, Parser, Subcommand};
use log::LevelFilter;

use crate::render::color::ColorChoice;
use crate::render::link::HyperlinkChoice;
use crate::{
    cmd::add, cmd::allow, cmd::check, cmd::config as config_cmd, cmd::dicts, cmd::eval, cmd::find,
    cmd::flip, cmd::go, cmd::graph, cmd::import, cmd::init, cmd::install, cmd::link as link_cmd,
    cmd::ls, cmd::mv, cmd::preset as preset_cmd, cmd::recur, cmd::review, cmd::rm, cmd::set,
    cmd::views, cmd::worktrees,
};

/// Mindtape keeps your tasks as Typst files and checks them against your rules.
#[derive(Parser)]
#[command(
    name = "mt",
    version,
    after_help = "Every status is a command too, like `mt wip <task>` and `mt done <task>`.\n\
                  Run `mt flip --list` to see them all."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Use the project at PATH, not the one the current directory is in.
    ///
    /// PATH is the directory holding `.mindtape/`. Without the flag, mt
    /// searches up from the current directory. Defaults to
    /// `$MINDTAPE_WORKTREE`.
    #[arg(short = 'w', long, value_name = "PATH", global = true)]
    pub worktree: Option<PathBuf>,

    /// Print more logging: `-v` for info, `-vv` for debug, `-vvv` for trace.
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// When to color the output.
    #[arg(long, value_name = "WHEN", default_value_t, global = true)]
    pub color: ColorChoice,

    /// When to make the `#` and `file` columns clickable links.
    ///
    /// Defaults to `[ls] hyperlinks`, which is `never` unless your project
    /// changes it.
    #[arg(long, value_name = "WHEN", global = true)]
    pub hyperlinks: Option<HyperlinkChoice>,
}

// The order here is the order `mt --help` lists: the commands a newcomer runs
// first, then everyday edits, then reading, then setup.
#[derive(Subcommand)]
enum Command {
    /// Create `.mindtape/config.toml` in the current directory.
    Init(init::InitArgs),
    /// File a new task from a template.
    Add(add::AddArgs),
    /// List tasks, optionally filtered.
    Ls(ls::LsArgs),
    /// Find tasks that contain every word you give.
    ///
    /// It takes the same flags as `mt ls`.
    Find(find::FindArgs),
    /// Change the status of tasks: `mt flip done <task>...`.
    ///
    /// Each status also works as a command of its own, so `mt done 1b 2c` is
    /// `mt flip done 1b 2c`. Those commands aren't listed here, so run
    /// `mt flip --list` to see them. When some tasks fail, mt still changes
    /// the rest and reports the failures.
    Flip(flip::FlipArgs),
    /// Check every task and every template against your project's rules.
    ///
    /// A template is checked by the task it would file.
    Check(check::CheckArgs),
    /// Change one field of a task: `mt set <task> <field>=<value>`.
    #[command(long_about = set::long_about())]
    Set(set::SetArgs),
    /// Link tasks: one link at a time, or many from a file.
    ///
    /// `mt link <task> <kind> <target>...` adds a link to each target, and
    /// `mt link --apply <file>` adds every link a YAML or JSON file lists.
    /// Running either twice adds nothing the second time.
    Link(link_cmd::LinkArgs),
    /// Rename a task's file and update every link that points to it.
    ///
    /// Inside a git repository it runs `git mv`. It keeps the task's title and
    /// folder, and refuses a stem another task has.
    Mv(mv::MvArgs),
    /// Delete tasks, keeping links to them findable in git history.
    ///
    /// Links to a deleted task become a SWHID, an ID that still finds its file
    /// in git history.
    Rm(rm::RmArgs),
    /// Evaluate one task file and print it, even outside a project.
    Eval(eval::EvalArgs),
    /// Open a task in your editor, set by `[go] editor`.
    Go(go::GoArgs),
    /// Mark a task as reviewed today.
    Review(review::ReviewArgs),
    /// Write the next occurrence of a recurring task.
    ///
    /// By default it writes a new task linked back to this one. With
    /// `materialize: "mutate"`, it resets this task in place instead.
    /// `mt done` closes a recurring task without writing the next one.
    Recur(recur::RecurArgs),
    /// Draw tasks and the links between them, as Graphviz DOT or Mermaid.
    Graph(graph::GraphArgs),
    /// List your saved views, or show one.
    Views(views::ViewsArgs),
    /// List your dictionaries, or the entries of one.
    Dicts(dicts::DictsArgs),
    /// Import tasks from JSON (the `mt ls --json` format), checking them all first.
    ///
    /// The whole batch is checked against your project's rules before mt
    /// writes any task.
    Import(import::ImportArgs),
    /// Install the Typst package, defaults, schema and completions.
    ///
    /// It installs what mt needs on this machine: the Typst package, your
    /// user-wide defaults, the config's JSON Schema, and shell completions.
    /// It installs all four unless you pass `--no-lib`, `--no-defaults`,
    /// `--no-schema` or `--no-completions`. Completions call mt itself, so
    /// they stay current when you upgrade.
    Install(install::InstallArgs),
    /// Show the config mt uses.
    Config(config_cmd::ConfigArgs),
    /// Trust the commands your config runs in this repo.
    ///
    /// Those are its `[[tasks.waits]]` checkers and `[[on.flip]]` `run`
    /// actions. It prints each command and records them for this repository
    /// and every worktree of it. Until you do, mt runs none of them, and a
    /// changed command asks again. Read them first: they run with your rights.
    Allow(allow::AllowArgs),
    /// Save this project's setup as a preset for `mt init`, or list presets.
    Preset(preset_cmd::PresetArgs),
    /// List your worktree aliases, the `@name` you put right after `mt`.
    Worktrees(worktrees::WorktreesArgs),
    /// Dump the command tree as JSON for the docs generator.
    ///
    /// Hidden. It reflects `Cli::command()`, so the rendered CLI reference
    /// cannot drift.
    #[command(name = "_cli", hide = true)]
    CliDump(tree_json::CliDumpArgs),
    /// Per-status shortcut verb (`<status> <task>...`), dispatched to `flip`.
    #[command(external_subcommand)]
    Status(Vec<String>),
}

impl Cli {
    /// The raw `[verb, args…]` when the line named no command and so falls to
    /// the status-shortcut fallback.
    pub(crate) fn status_shortcut(&self) -> Option<&[String]> {
        match &self.command {
            Command::Status(external) => Some(external),
            _ => None,
        }
    }

    /// Dispatch to the selected command.
    ///
    /// # Errors
    /// Propagates the command's failure.
    pub fn run(self) -> Result<()> {
        // jscpd:ignore-start
        // One arm per command, `Command::X(args) => x::run(&args)`: the match is the
        // dispatch table, and a helper would only hide which module runs which verb.
        match self.command {
            Command::Add(args) => add::run(&args),
            Command::Import(args) => import::run(&args),
            Command::Ls(args) => ls::run(&args),
            Command::Find(args) => find::run(&args),
            Command::Graph(args) => graph::run(&args),
            Command::Views(args) => views::run(&args),
            Command::Dicts(args) => dicts::run(&args),
            Command::Worktrees(args) => worktrees::run(&args),
            Command::Check(args) => check::run(&args),
            Command::Allow(args) => allow::run(&args),
            Command::Config(args) => config_cmd::run(&args),
            Command::Preset(args) => preset_cmd::run(&args),
            Command::Init(args) => init::run(&args),
            Command::Install(args) => install::run(&args),
            Command::Eval(args) => eval::run(&args),
            Command::Go(args) => go::run(&args),
            Command::Rm(args) => rm::run(&args),
            Command::Mv(args) => mv::run(&args),
            Command::Set(args) => set::run(&args),
            Command::Review(args) => review::run(&args),
            Command::Flip(args) => flip::run(&args),
            Command::Recur(args) => recur::run(&args),
            Command::Link(args) => link_cmd::run(&args),
            Command::CliDump(args) => tree_json::run(&args),
            Command::Status(argv) => flip::run_external(&argv),
        }
        // jscpd:ignore-end
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
    use clap::{CommandFactory, Parser};
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

    /// The commands a newcomer runs first open the listing, in that order.
    #[test]
    fn newcomer_commands_lead_the_listing() {
        let cmd = Cli::command();
        let names: Vec<&str> = cmd
            .get_subcommands()
            .filter(|sub| !sub.is_hide_set())
            .map(clap::Command::get_name)
            .take(6)
            .collect();
        assert_eq!(names, ["init", "add", "ls", "find", "flip", "check"]);
    }

    /// The status commands are external, so clap never lists them. The note
    /// after the listing says they exist and where to see them.
    #[test]
    fn help_points_at_the_status_commands() {
        for flag in ["--help", "-h"] {
            let Err(err) = Cli::try_parse_from(["mt", flag]) else {
                panic!("{flag} must print help");
            };
            let help = err.to_string();
            assert!(help.contains("mt flip --list"), "{flag}: {help}");
        }
    }

    /// Every short help line, a command's `about` or a flag's help, reads at
    /// a glance. Detail goes in the long help, after a blank doc line.
    #[test]
    fn every_short_help_fits_one_line() {
        const CAP: usize = 80;
        let mut long = Vec::new();
        let mut stack = vec![("mt".to_owned(), Cli::command())];
        while let Some((path, cmd)) = stack.pop() {
            let about = cmd.get_about().map(ToString::to_string).unwrap_or_default();
            if about.chars().count() > CAP {
                long.push(format!("{path}: {about}"));
            }
            for arg in cmd.get_arguments() {
                let help = arg.get_help().map(ToString::to_string).unwrap_or_default();
                if help.chars().count() > CAP {
                    long.push(format!("{path} {}: {help}", arg.get_id()));
                }
            }
            for sub in cmd.get_subcommands() {
                stack.push((format!("{path} {}", sub.get_name()), sub.clone()));
            }
        }
        assert!(long.is_empty(), "over {CAP} chars:\n{}", long.join("\n"));
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
        if mindtape::test_support::rerun_isolated(CHILD).is_some() {
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
