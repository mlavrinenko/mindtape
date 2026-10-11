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
//! `{id}` is pasted into a shell line, so [`command_line`] refuses an id with a
//! character outside [`ID_PUNCT`] and [`annotate`] never runs one.
//!
//! No checker runs until the user allows the config's commands with `mt allow`
//! ([`crate::allow`]). Until then every entry answers [`Answer::NotAllowed`],
//! which blocks like an unknown answer, and [`held_back`] names the commands
//! for the one warning the caller prints.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::Deserialize;

use crate::config::{Config, quote};
use crate::model::Task;
use crate::suggest::Unknown;

/// One `[[tasks.waits]]` entry: a named command that answers whether a task
/// still waits.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Checker {
    /// The name a `waits-on` entry writes before its colon.
    pub name: String,
    /// The command line, with `{id}` standing for the entry's id. It runs
    /// through `sh -c`, or `cmd /C` on Windows, from the project root. Exit 0
    /// means the wait is over, 1 means it goes on, anything else is unknown.
    pub run: String,
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

/// Every answer this process has had, keyed by project root and command line,
/// so a checker that several tasks or several loads ask about runs once.
static ANSWERS: Mutex<BTreeMap<(PathBuf, String), Answer>> = Mutex::new(BTreeMap::new());

/// Every command this process held back because the user has not allowed it.
static HELD: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// The commands this process would have run but held back, sorted: what the
/// warning that asks for `mt allow` lists. Empty when every checker ran.
#[must_use]
pub fn held_back() -> Vec<String> {
    let held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    held.iter().cloned().collect()
}

/// Stamp each open task's [`Task::answers`] with what its checkers say, running
/// each command line at most once per process. A terminal task runs nothing,
/// and no checker runs while the config's commands are not allowed.
pub fn annotate(tasks: &mut [Task], config: &Config, root: &Path) {
    annotate_with(tasks, config, root, || allowed(config, root));
}

/// [`annotate`], with `allowed` saying whether checkers may run. It is asked
/// only when some open task waits on something.
fn annotate_with(tasks: &mut [Task], config: &Config, root: &Path, allowed: impl FnOnce() -> bool) {
    let terminal = config.tasks.status.terminal();
    let asked: Vec<&mut Task> = tasks
        .iter_mut()
        .filter(|task| !task.waits_on.is_empty() && !task.is_terminal(&terminal))
        .collect();
    if asked.is_empty() {
        return;
    }
    let allowed = allowed();
    for task in asked {
        stamp(task, config, root, allowed);
    }
}

/// Stamp `task`'s [`Task::answers`] whatever its status: what a flip out of a
/// terminal status needs, since [`annotate`] asks nothing for that task.
pub fn ask(task: &mut Task, config: &Config, root: &Path) {
    let allowed = task.waits_on.is_empty() || allowed(config, root);
    stamp(task, config, root, allowed);
}

/// Whether the user allowed the commands `config` runs for `root`'s repository.
fn allowed(config: &Config, root: &Path) -> bool {
    crate::allow::is_allowed(root, &crate::allow::commands(config))
}

/// Stamp `task`'s [`Task::answers`], running its checkers only when `allowed`.
fn stamp(task: &mut Task, config: &Config, root: &Path, allowed: bool) {
    task.answers = task
        .waits_on
        .iter()
        .map(|entry| (entry.clone(), answer(entry, config, root, allowed)))
        .collect();
}

/// What `entry`'s checker says, from the process-wide memo or a fresh run, or
/// [`Answer::NotAllowed`] without a run when the commands are not `allowed`.
fn answer(entry: &str, config: &Config, root: &Path, allowed: bool) -> Answer {
    let command = match command_line(entry, config) {
        Ok(command) => command,
        Err(problem) => return Answer::Unknown(problem.message),
    };
    if !allowed {
        let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
        held.extend(crate::allow::commands(config));
        return Answer::NotAllowed;
    }
    let mut answers = ANSWERS.lock().unwrap_or_else(PoisonError::into_inner);
    answers
        .entry((root.to_path_buf(), command))
        .or_insert_with_key(|(root, command)| run(command, root))
        .clone()
}

/// Run one checker command and read its exit code as an [`Answer`].
fn run(command: &str, root: &Path) -> Answer {
    match crate::shell::run(command, root, &[]) {
        Ok(output) => {
            let line = first_line(&output.stdout).or_else(|| first_line(&output.stderr));
            match output.code {
                Some(0) => Answer::Resolved,
                Some(1) => Answer::Waiting(line.unwrap_or_else(|| "waiting".to_owned())),
                Some(code) => Answer::Unknown(line.unwrap_or_else(|| format!("exit {code}"))),
                None => Answer::Unknown(line.unwrap_or_else(|| "terminated by signal".to_owned())),
            }
        }
        Err(err) => Answer::Unknown(format!("can't run {}: {err}", quote(command))),
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
pub fn command_line(entry: &str, config: &Config) -> Result<String, EntryProblem> {
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
    Ok(checker.run.replace("{id}", id))
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
        if checker.run.trim().is_empty() {
            problems.push(format!("{at}.run: empty command"));
        }
    }
    problems
}

#[cfg(test)]
#[path = "waits_tests.rs"]
mod tests;
