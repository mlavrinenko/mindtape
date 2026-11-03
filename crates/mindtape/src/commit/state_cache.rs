//! The reusable part of commit-state loading: file outcomes, the previous
//! state's task identities, and the small helpers over them.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

use crate::commit::tree::TaskEntry;
use crate::config::Config;
use crate::loader::{LoadError, StatePackages};
use crate::model::Task;

/// What one task file produced in a state.
#[derive(Clone, Copy)]
pub(crate) enum Outcome {
    /// The index of its task in the state's task list.
    Task(usize),
    /// It evaluated cleanly to no task marker.
    Missing,
    /// It failed to evaluate, and so is never reused.
    Failed,
}

/// One task file's resolved source identity and outcome in a state.
pub(crate) struct SeenFile {
    pub(crate) source: String,
    pub(crate) mode: String,
    pub(crate) outcome: Outcome,
}

/// Look up `entry` in `reuse`, validating a seeded task's root choice.
pub(crate) fn lookup(
    reuse: &Reuse<'_>,
    entry: &TaskEntry,
    source: &str,
    config: &Config,
) -> Option<Option<Task>> {
    let seen = reuse.files.get(&entry.rel)?;
    if seen.source != source || seen.mode != entry.mode {
        return None;
    }
    match seen.outcome {
        Outcome::Task(index) => {
            let task = reuse.tasks.get(index)?;
            if reuse.seeded && !fits(task, config, &entry.rel) {
                return None;
            }
            Some(Some(task.clone()))
        }
        Outcome::Missing => Some(None),
        Outcome::Failed => None,
    }
}

/// Add one evaluation to the state, recording its outcome.
pub(crate) fn record(
    found: Result<Option<Task>, LoadError>,
    tasks: &mut Vec<Task>,
    errors: &mut Vec<LoadError>,
) -> Outcome {
    match found {
        Ok(Some(task)) => {
            let index = tasks.len();
            tasks.push(task);
            Outcome::Task(index)
        }
        Ok(None) => Outcome::Missing,
        Err(err) => {
            errors.push(err);
            Outcome::Failed
        }
    }
}

/// Whether a seeded task's folder-derived tags and depth still match what the
/// state's own root choice gives this path.
pub(crate) fn fits(task: &Task, config: &Config, rel: &Path) -> bool {
    let (tags, depth) = crate::loader::derivation(config, rel);
    task.derived_tags == tags && task.depth == depth
}

/// The config facts a harvest reads, plus the exact package bytes it resolves.
pub(crate) fn load_env(config: &Config, packages: Option<&StatePackages>) -> String {
    format!(
        "{:?}\0{:?}\0{:?}\0{}",
        config.tasks.roots,
        config.tasks.derive_tags,
        config.typst.local_packages,
        packages.map_or("", |packages| packages.fingerprint.as_str()),
    )
}

/// The git blob id of `rel`'s working-tree bytes, as `git hash-object` would
/// report it. `None` when the file cannot be read.
pub(crate) fn working_oid(root: &Path, rel: &Path) -> Option<String> {
    let bytes = std::fs::read(root.join(rel)).ok()?;
    let mut hasher = Sha1::new();
    hasher.update(format!("blob {}\0", bytes.len()).as_bytes());
    hasher.update(&bytes);
    Some(crate::loader::cache::hex(&hasher.finalize()))
}

/// The tasks and file outcomes a state may reuse.
pub(crate) struct Reuse<'a> {
    pub(crate) tasks: &'a [Task],
    pub(crate) files: &'a HashMap<PathBuf, SeenFile>,
    /// True for the working-tree load, whose root choice must be checked.
    pub(crate) seeded: bool,
}

/// One evaluated task file in a new state.
pub(crate) struct Seen {
    pub(crate) entry: TaskEntry,
    pub(crate) source: String,
    pub(crate) outcome: Outcome,
}

/// The result of evaluating one state's files.
pub(crate) struct Build {
    pub(crate) tasks: Vec<Task>,
    pub(crate) errors: Vec<LoadError>,
    pub(crate) seen: Vec<Seen>,
}

/// The task and error lists a state is being assembled into.
pub(crate) struct Parts<'a> {
    pub(crate) tasks: &'a mut Vec<Task>,
    pub(crate) errors: &'a mut Vec<LoadError>,
}
