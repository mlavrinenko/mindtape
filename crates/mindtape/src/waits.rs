//! `waits-on`: a task that waits on something outside the repo, answered by a
//! configured command.
//!
//! ```toml
//! [[tasks.waits]]
//! name = "locked"
//! run = "scripts/locked {id}"
//! ```
//!
//! A task names what it waits on as `checker:id` entries, such as
//! `waits-on: ("locked:redis@0.27.6",)`. The checker's `run` line, with `{id}`
//! replaced, goes through [`crate::shell`] from the project root. Exit 0 means
//! the wait is over, so the entry blocks nothing. Exit 1 means the task still
//! waits. Any other exit, a signal, or a shell that cannot start means the
//! answer is unknown, and an unknown answer blocks too. The first line the
//! command prints is the answer's text, which the `why` column shows.
//!
//! Read commands run checkers, since `mt ls ready` is the point. So a checker
//! must answer from local state, fast: refreshing that state over the network
//! is the tool's own job. [`annotate`] runs each distinct command line once per
//! process, and only for open tasks.
//!
//! A string `run` pastes `{id}` into a shell line, so [`command_line`] refuses
//! an id with a character outside [`ID_PUNCT`] and [`annotate`] never runs
//! one. An array `run` fills `{id}` into each item, where no shell reads it.
//! The id rule holds there too, so an entry is valid under either form.
//!
//! No checker runs until the user allows the config's commands with `mt allow`
//! ([`crate::allow`]). Until then every entry answers [`Answer::NotAllowed`],
//! which blocks like an unknown answer, and [`crate::allow::held_back`] names
//! the commands for the one warning the caller prints.
//!
//! A checker runs with [`NESTED_ENV`] set, and an `mt` that finds it set runs
//! no checker: every entry answers [`Answer::Nested`]. So a checker that runs
//! `mt` against its own project, which loads the project and asks its waits
//! again, gets one answer instead of spawning itself without end. Nesting
//! stops at one level by construction, whatever the checkers run.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::Deserialize;

use crate::config::{Config, quote};
use crate::model::Task;
use crate::shell::ConfigCommand;
use crate::suggest::Unknown;

/// One `[[tasks.waits]]` entry: a named command that answers whether a task
/// still waits.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Checker {
    /// The name a `waits-on` entry writes before its colon.
    pub name: String,
    /// The command, with `{id}` standing for the entry's id: a line that runs
    /// through `sh -c`, or `cmd /C` on Windows, or an array whose program runs
    /// with no shell and whose every item gets `{id}` filled. It runs from the
    /// project root. Exit 0 means the wait is over, 1 means it goes on,
    /// anything else is unknown.
    pub run: ConfigCommand,
}

/// What a checker answered for one `waits-on` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// Exit 0: the wait is over, and the entry blocks nothing.
    Resolved,
    /// Exit 1: the task still waits, for the reason the line gives.
    Waiting(String),
    /// Anything else: the checker could not tell, for the reason the line
    /// gives. Blocks like [`Answer::Waiting`].
    Unknown(String),
    /// The checker did not run, since the user has not allowed the config's
    /// commands. Blocks like [`Answer::Unknown`].
    NotAllowed,
    /// The checker did not run, since this `mt` runs inside a checker
    /// ([`NESTED_ENV`]). Blocks like [`Answer::Unknown`].
    Nested,
}

impl Answer {
    /// The text `why` shows beside the entry; `None` for a resolved wait.
    #[must_use]
    pub fn reason(&self) -> Option<String> {
        match self {
            Self::Resolved => None,
            Self::Waiting(line) => Some(line.clone()),
            Self::Unknown(line) => Some(format!("unknown: {line}")),
            Self::NotAllowed => Some("unknown: not allowed".to_owned()),
            Self::Nested => Some("unknown: asked inside a checker".to_owned()),
        }
    }
}

/// The task field that lists what a task waits on, and the `kind` of the
/// [`crate::Blocker`] each unanswered entry becomes.
pub const FIELD: &str = "waits-on";

/// The punctuation an id may hold beside ASCII letters and digits. None of it
/// means anything to `sh` or `cmd` inside a word, and an id starts with a
/// letter or digit, so it cannot open an option or a comment either.
pub const ID_PUNCT: &str = "._@/#:+-";

/// The variable every checker runs with. An `mt` that finds it set runs no
/// checker of its own and answers [`Answer::Nested`] instead.
pub const NESTED_ENV: &str = "MINDTAPE_WAITS_CHECKER";

/// Whether checkers may run in this process, and if not, why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Permit {
    /// Run them.
    Run,
    /// The user has not allowed the config's commands.
    NotAllowed,
    /// This `mt` runs inside a checker.
    Nested,
}

/// Every answer this process has had, keyed by project root and command line,
/// so a checker that several tasks or several loads ask about runs once.
static ANSWERS: Mutex<BTreeMap<(PathBuf, ConfigCommand), Answer>> = Mutex::new(BTreeMap::new());

/// Stamp each open task's [`Task::answers`] with what its checkers say, running
/// each command line at most once per process. A terminal task runs nothing,
/// and no checker runs while the config's commands are not allowed or inside
/// another checker.
pub fn annotate(tasks: &mut [Task], config: &Config, root: &Path) {
    annotate_with(tasks, config, root, || permit(config, root));
}

/// [`annotate`], with `permit` saying whether checkers may run. It is asked
/// only when some open task waits on something.
fn annotate_with(
    tasks: &mut [Task],
    config: &Config,
    root: &Path,
    permit: impl FnOnce() -> Permit,
) {
    let terminal = config.tasks.status.terminal();
    let asked: Vec<&mut Task> = tasks
        .iter_mut()
        .filter(|task| !task.waits_on.is_empty() && !task.is_terminal(&terminal))
        .collect();
    if asked.is_empty() {
        return;
    }
    let permit = permit();
    for task in asked {
        stamp(task, config, root, permit);
    }
}

/// Stamp `task`'s [`Task::answers`] whatever its status: what a flip out of a
/// terminal status needs, since [`annotate`] asks nothing for that task.
pub fn ask(task: &mut Task, config: &Config, root: &Path) {
    let permit = if task.waits_on.is_empty() {
        Permit::Run
    } else {
        permit(config, root)
    };
    stamp(task, config, root, permit);
}

/// Whether checkers may run: not inside another checker, and only once the
/// user allowed the commands `config` runs for `root`'s repository.
fn permit(config: &Config, root: &Path) -> Permit {
    if std::env::var_os(NESTED_ENV).is_some() {
        Permit::Nested
    } else if crate::allow::is_allowed(root, &crate::allow::commands(config)) {
        Permit::Run
    } else {
        Permit::NotAllowed
    }
}

/// Stamp `task`'s [`Task::answers`], running its checkers only on
/// [`Permit::Run`].
fn stamp(task: &mut Task, config: &Config, root: &Path, permit: Permit) {
    task.answers = task
        .waits_on
        .iter()
        .map(|entry| (entry.clone(), answer(entry, config, root, permit)))
        .collect();
}

/// What `entry`'s checker says, from the process-wide memo or a fresh run, or
/// the answer `permit` gives without a run when it is not [`Permit::Run`].
fn answer(entry: &str, config: &Config, root: &Path, permit: Permit) -> Answer {
    let command = match command_line(entry, config) {
        Ok(command) => command,
        Err(problem) => return Answer::Unknown(problem.message),
    };
    match permit {
        Permit::Run => {}
        Permit::Nested => return Answer::Nested,
        Permit::NotAllowed => {
            crate::allow::hold(root, &crate::allow::commands(config));
            return Answer::NotAllowed;
        }
    }
    let mut answers = ANSWERS.lock().unwrap_or_else(PoisonError::into_inner);
    answers
        .entry((root.to_path_buf(), command))
        .or_insert_with_key(|(root, command)| run(command, root))
        .clone()
}

/// Run one checker command, with [`NESTED_ENV`] set, and read its exit code
/// as an [`Answer`].
fn run(command: &ConfigCommand, root: &Path) -> Answer {
    match crate::shell::run(command, root, &[(NESTED_ENV, OsStr::new("1"))]) {
        Ok(output) => {
            let line = first_line(&output.stdout).or_else(|| first_line(&output.stderr));
            match output.code {
                Some(0) => Answer::Resolved,
                Some(1) => Answer::Waiting(line.unwrap_or_else(|| "waiting".to_owned())),
                Some(code) => Answer::Unknown(line.unwrap_or_else(|| format!("exit {code}"))),
                None => Answer::Unknown(line.unwrap_or_else(|| "terminated by signal".to_owned())),
            }
        }
        Err(err) => Answer::Unknown(format!("can't run {}: {err}", quote(&command.to_string()))),
    }
}

/// The first line of `text` that holds anything but whitespace, trimmed.
fn first_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

/// Why `entry` cannot run: no `checker:id` shape, a checker the config does not
/// declare, or an id that could reach the shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryProblem {
    /// The problem, one line.
    pub message: String,
    /// The declared checker nearest a misspelled name.
    pub unknown: Option<Unknown>,
}

/// The command line `entry` runs: its checker's `run` with `{id}` replaced.
///
/// # Errors
/// Returns an [`EntryProblem`] when `entry` has no colon, names a checker
/// `[[tasks.waits]]` does not declare, or has an id that is empty, starts with
/// punctuation, or holds a character outside letters, digits and [`ID_PUNCT`].
pub fn command_line(entry: &str, config: &Config) -> Result<ConfigCommand, EntryProblem> {
    let plain = |message: String| EntryProblem {
        message,
        unknown: None,
    };
    let Some((name, id)) = entry.split_once(':') else {
        return Err(plain(format!("{} is not checker:id", quote(entry))));
    };
    let checkers = &config.tasks.waits;
    let Some(checker) = checkers.iter().find(|checker| checker.name == name) else {
        return Err(EntryProblem {
            message: format!("unknown checker: {}", quote(name)),
            unknown: Some(Unknown::among(
                name,
                checkers.iter().map(|checker| checker.name.as_str()),
            )),
        });
    };
    if !is_safe_id(id) {
        return Err(plain(format!(
            "id {} must start with a letter or digit and hold only letters, digits and {ID_PUNCT}",
            quote(id)
        )));
    }
    Ok(checker.run.map(|part| part.replace("{id}", id)))
}

/// Whether `id` can stand in a shell word as is: see [`ID_PUNCT`].
fn is_safe_id(id: &str) -> bool {
    id.starts_with(|ch: char| ch.is_ascii_alphanumeric())
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ID_PUNCT.contains(ch))
}

/// Config-load problems in `[[tasks.waits]]`: a name a `waits-on` entry
/// cannot spell, a name declared twice, or an empty `run`.
pub(crate) fn validate_checkers(config: &Config) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (index, checker) in config.tasks.waits.iter().enumerate() {
        let at = format!("tasks.waits[{index}]");
        let name = &checker.name;
        if name.is_empty()
            || !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        {
            problems.push(format!(
                "{at}.name: {} must be letters, digits, - and _",
                quote(name)
            ));
        }
        if !seen.insert(name.as_str()) {
            problems.push(format!("{at}.name: {} is declared twice", quote(name)));
        }
        if checker.run.is_blank() {
            problems.push(format!("{at}.run: empty command"));
        }
    }
    problems
}

#[cfg(test)]
#[path = "waits_tests.rs"]
mod tests;
