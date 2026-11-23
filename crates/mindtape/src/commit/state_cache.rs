//! The reusable part of commit-state loading: file outcomes, the previous
//! state's task identities with the project files they read, and the small
//! helpers over them.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

use crate::commit::tree::TaskEntry;
use crate::config::Config;
use crate::loader::cache::TaskCache;
use crate::loader::{LoadError, StatePackages};
use crate::model::Task;
use crate::validate::candidate::HarvestedSource;

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
    /// The project files the harvest read, with the content hash each held at
    /// the time, from [`dep_hashes`]. A reuse needs them unchanged, since an
    /// imported module's bytes decide the markers the task carries.
    pub(crate) deps: Vec<(PathBuf, String)>,
    pub(crate) outcome: Outcome,
}

/// One task file a state reuses: the task, or its confirmed absence, and the
/// project files its harvest read.
pub(crate) struct Reused {
    pub(crate) task: Option<Task>,
    pub(crate) deps: Vec<(PathBuf, String)>,
}

/// Look up `entry` in `reuse`, validating a seeded task's root choice.
pub(crate) fn lookup(
    reuse: &Reuse<'_>,
    entry: &TaskEntry,
    source: &str,
    config: &Config,
) -> Option<Reused> {
    let seen = reuse.files.get(&entry.rel)?;
    if seen.source != source || seen.mode != entry.mode || !deps_unchanged(reuse.root, &seen.deps) {
        return None;
    }
    let deps = seen.deps.clone();
    match seen.outcome {
        Outcome::Task(index) => {
            let task = reuse.tasks.get(index)?;
            if reuse.seeded && !fits(task, config, &entry.rel) {
                return None;
            }
            Some(Reused {
                task: Some(task.clone()),
                deps,
            })
        }
        Outcome::Missing => Some(Reused { task: None, deps }),
        Outcome::Failed => None,
    }
}

/// Add one evaluation to the state, recording its outcome. `Ok(None)` is a file
/// the state lacks the bytes of, which loads as no task.
fn record(
    found: Result<Option<HarvestedSource>, LoadError>,
    tasks: &mut Vec<Task>,
    errors: &mut Vec<LoadError>,
) -> Outcome {
    match found {
        Ok(Some(harvested)) => match harvested.task {
            Some(task) => {
                let index = tasks.len();
                tasks.push(task);
                Outcome::Task(index)
            }
            None => Outcome::Missing,
        },
        Ok(None) => Outcome::Missing,
        Err(err) => {
            errors.push(err);
            Outcome::Failed
        }
    }
}

/// [`record`] for a file the state read itself, with the content hash of every
/// project file its harvest read.
pub(crate) fn record_evaluated(
    root: &Path,
    found: Result<Option<HarvestedSource>, LoadError>,
    tasks: &mut Vec<Task>,
    errors: &mut Vec<LoadError>,
) -> (Outcome, Vec<(PathBuf, String)>) {
    let deps = match &found {
        Ok(Some(harvested)) => dep_hashes(root, &harvested.dependencies),
        Ok(None) | Err(_) => Vec::new(),
    };
    (record(found, tasks, errors), deps)
}

/// Whether every project file a harvest read still holds the bytes it read.
/// The hash is [`TaskCache::content_hash`], the one the content cache records
/// for its own dependencies. A file the working tree lost or gained counts as
/// changed.
pub(crate) fn deps_unchanged(root: &Path, deps: &[(PathBuf, String)]) -> bool {
    deps.iter().all(|(rel, hash)| {
        std::fs::read(root.join(rel)).is_ok_and(|bytes| TaskCache::content_hash(&bytes) == *hash)
    })
}

/// The content hash of every project file a harvest read: the reuse key's copy
/// of the content cache's dependency list. A path the working tree lacks stays
/// out, because the harvest read those bytes from a package or a state pin. The
/// load environment keys those on its own.
pub(crate) fn dep_hashes(root: &Path, deps: &[PathBuf]) -> Vec<(PathBuf, String)> {
    deps.iter()
        .filter_map(|rel| {
            let bytes = std::fs::read(root.join(rel)).ok()?;
            Some((rel.clone(), TaskCache::content_hash(&bytes)))
        })
        .collect()
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
/// report it. `None` when reading the file fails.
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
    /// The project root every dependency hash is relative to.
    pub(crate) root: &'a Path,
    /// True for the working-tree load, which must check its root choice.
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
    /// The project files each seen file read, keyed by the file's
    /// project-relative path. [`record_evaluated`] fills the answer for a file
    /// the state evaluated, and the reuse map for a file a previous state
    /// answered for.
    pub(crate) deps: HashMap<PathBuf, Vec<(PathBuf, String)>>,
}
