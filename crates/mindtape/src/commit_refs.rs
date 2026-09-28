//! Which tasks shaped a file: the `[commits] trailer` values of every commit
//! that changed it, read from `git log` — the index behind the filter
//! language's `touches("<path>")`.
//!
//! One `git log -M --name-status` over the whole history of `HEAD` yields,
//! per commit, its message and the paths it changed. [`parse_log`] indexes
//! every changed path by the stems its trailer names, through the same
//! trailer parse `mt check --commit-msg` uses ([`crate::commit_msg`]), so a
//! path-form value (`Refs: truth/tasks/<stem>.typ`) and a stem-form one
//! (`Refs: <stem>`) index the same stem. A rename indexes both of its sides.
//!
//! A stem names the task by its name when the trailer was written; `mt mv`
//! has renamed some since. Every `.typ` rename is recorded, and the ones
//! under the task roots resolve a stem to the task it is now
//! ([`RefsCache::touching`]). A stem no loaded task answers to, even through
//! a rename, belongs to a retired task: it matches nothing, and is kept so
//! `mt ls --json` can list it.
//!
//! The index depends only on `HEAD` and the trailer key, so it is cached in
//! `<root>/.mindtape/cache/commit-refs.json` keyed by both, rebuilt whenever
//! either moves. Nothing reads git until a filter calls `touches()`.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use serde::{Deserialize, Serialize};

use crate::commit_msg::{is_task_file, parse_message, task_stem, value_stem};
use crate::config::Config;
use crate::git_state::git;
use crate::model::Task;

/// Bump when the cached shape changes: a mismatch is a miss.
const FORMAT_VERSION: u32 = 1;

/// The cache file's name inside `<root>/.mindtape/cache/`.
const CACHE_FILE: &str = "commit-refs.json";

/// Opens each commit's record in the log format.
const RECORD: char = '\x1e';

/// Ends each commit's message in the log format.
const MESSAGE_END: char = '\x1f';

/// What the history says, before any stem is resolved against the tasks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitRefs {
    /// Every path a commit naming a task changed, repository-relative, mapped
    /// to the stems those commits' trailers name — oldest first, each once.
    pub touched: BTreeMap<String, Vec<String>>,
    /// Every `.typ` rename, `(from, to)`, oldest first.
    pub renames: Vec<(String, String)>,
}

/// Index `git log --reverse -M --name-status -z --format=%x1e%B%x1f` output
/// by the stems each commit's `key` trailer names.
#[must_use]
pub fn parse_log(text: &str, key: &str) -> CommitRefs {
    let mut refs = CommitRefs::default();
    for record in text.split(RECORD).filter(|record| !record.is_empty()) {
        let (message, files) = record.split_once(MESSAGE_END).unwrap_or((record, ""));
        let stems: Vec<String> = parse_message(message, key)
            .trailers
            .iter()
            .map(|trailer| value_stem(&trailer.value))
            .collect();
        let mut fields = files
            .split('\0')
            .map(|field| field.trim_start_matches('\n'))
            .filter(|field| !field.is_empty());
        while let Some(status) = fields.next() {
            let sides = if status.starts_with(['R', 'C']) { 2 } else { 1 };
            let paths: Vec<&str> = fields.by_ref().take(sides).collect();
            if let (true, [from, to]) = (status.starts_with('R'), paths.as_slice())
                && from.ends_with(".typ")
                && to.ends_with(".typ")
            {
                refs.renames.push(((*from).to_owned(), (*to).to_owned()));
            }
            for path in paths.into_iter().filter(|_| !stems.is_empty()) {
                let indexed = refs.touched.entry(path.to_owned()).or_default();
                for stem in &stems {
                    if !indexed.contains(stem) {
                        indexed.push(stem.clone());
                    }
                }
            }
        }
    }
    refs
}

/// The tasks one path's commits name, resolved against the loaded set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Touched {
    /// The stems of the tasks that match.
    pub(crate) stems: HashSet<String>,
    /// The named stems no loaded task answers to, even through a rename.
    pub(crate) retired: Vec<String>,
}

/// The on-disk cache entry: `R` is the index, owned when read and borrowed
/// when written.
#[derive(Serialize, Deserialize)]
struct Entry<R> {
    format: u32,
    head: String,
    trailer: String,
    refs: R,
}

/// The history read for `root`, and what came of it.
#[derive(Debug)]
struct Read {
    refs: CommitRefs,
    /// The project root's path within the repository.
    root_prefix: String,
}

/// The commit-refs index of one project root, read the first time a filter
/// asks, plus every answer given and every retired stem met. Carried on
/// [`crate::git_state::GitCache`]; clones share all three.
#[derive(Debug, Clone, Default)]
pub(crate) struct RefsCache {
    read: Arc<OnceLock<Result<Read, String>>>,
    answers: Arc<Mutex<HashMap<String, Arc<Touched>>>>,
    retired: Arc<Mutex<BTreeSet<String>>>,
}

impl RefsCache {
    /// The tasks the commits changing `path` name. With a loaded `set` a
    /// stem resolves to the first name along its renames that a task bears,
    /// and a stem that reaches none is retired; with none a stem resolves to
    /// the last name along its renames.
    ///
    /// # Errors
    /// Why git could not answer — outside a repository, most often.
    pub(crate) fn touching(
        &self,
        root: &Path,
        path: &str,
        config: &Config,
        set: Option<&[Task]>,
    ) -> Result<Arc<Touched>, String> {
        let path = path.trim_start_matches("./").replace('\\', "/");
        let key = format!("{}\0{path}", set.is_some());
        if let Some(hit) = lock(&self.answers).get(&key) {
            return Ok(Arc::clone(hit));
        }
        let read = self
            .read
            .get_or_init(|| read(root, &config.commits.trailer))
            .as_ref()
            .map_err(Clone::clone)?;
        let touched = Arc::new(resolve(read, &path, config, set));
        lock(&self.retired).extend(touched.retired.iter().cloned());
        lock(&self.answers).insert(key, Arc::clone(&touched));
        Ok(touched)
    }

    /// Every retired stem a `touches()` answer met, sorted.
    pub(crate) fn retired(&self) -> Vec<String> {
        lock(&self.retired).iter().cloned().collect()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    // A poisoned lock still holds only finished answers.
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Resolve the stems the commits changing `path` name.
fn resolve(read: &Read, path: &str, config: &Config, set: Option<&[Task]>) -> Touched {
    let current: Option<HashSet<String>> = set.map(|tasks| tasks.iter().map(task_stem).collect());
    let renamed: HashMap<String, String> = read
        .refs
        .renames
        .iter()
        .filter(|(from, to)| {
            is_task_file(from, &read.root_prefix, config)
                && is_task_file(to, &read.root_prefix, config)
        })
        .map(|(from, to)| (value_stem(from), value_stem(to)))
        .filter(|(from, to)| from != to)
        .collect();
    let mut touched = Touched::default();
    for stem in read.refs.touched.get(path).into_iter().flatten() {
        match current_name(stem, &renamed, current.as_ref()) {
            Some(name) => {
                touched.stems.insert(name);
            }
            None => touched.retired.push(stem.clone()),
        }
    }
    touched
}

/// The name `stem` goes by now: the first along its renames a task in
/// `current` bears, `None` when none does; with no `current`, the last.
fn current_name(
    stem: &str,
    renamed: &HashMap<String, String>,
    current: Option<&HashSet<String>>,
) -> Option<String> {
    let mut name = stem;
    // A rename cycle cannot outlast one pass over every rename.
    for _ in 0..=renamed.len() {
        if current.is_some_and(|set| set.contains(name)) {
            return Some(name.to_owned());
        }
        match renamed.get(name) {
            Some(next) => name = next,
            None => break,
        }
    }
    current.is_none().then(|| name.to_owned())
}

/// Read the index for `root`'s `HEAD`, from the cache when it holds this
/// `HEAD` and trailer key, else from `git log`, caching the result.
fn read(root: &Path, trailer: &str) -> Result<Read, String> {
    let root_prefix = git(root, &["rev-parse", "--show-prefix"])?
        .trim()
        .to_owned();
    // No `HEAD` yet: an unborn branch has no history to index.
    let Ok(head) = git(root, &["rev-parse", "--verify", "-q", "HEAD"]) else {
        return Ok(Read {
            refs: CommitRefs::default(),
            root_prefix,
        });
    };
    let head = head.trim().to_owned();
    let cache = cache_path(root);
    if let Some(refs) = cache
        .as_deref()
        .and_then(|path| cached(path, &head, trailer))
    {
        return Ok(Read { refs, root_prefix });
    }
    let log = git(
        root,
        &[
            "log",
            "--reverse",
            "-M",
            "--name-status",
            "-z",
            "--no-relative",
            "--format=%x1e%B%x1f",
            &head,
        ],
    )?;
    let refs = parse_log(&log, trailer);
    if let Some(path) = cache {
        store(&path, &head, trailer, &refs);
    }
    Ok(Read { refs, root_prefix })
}

/// Where `root`'s index is cached; `None` when the cache is disabled or
/// there is no root to put it under.
fn cache_path(root: &Path) -> Option<std::path::PathBuf> {
    (!crate::loader::cache::disabled() && !root.as_os_str().is_empty())
        .then(|| root.join(".mindtape").join("cache").join(CACHE_FILE))
}

/// The cached index, when the entry at `path` holds this `head` and
/// `trailer`; any unreadable or mismatched entry is a miss.
fn cached(path: &Path, head: &str, trailer: &str) -> Option<CommitRefs> {
    let entry: Entry<CommitRefs> =
        serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    (entry.format == FORMAT_VERSION && entry.head == head && entry.trailer == trailer)
        .then_some(entry.refs)
}

/// Cache `refs` at `path`. Best-effort: a failure leaves the next read a miss.
fn store(path: &Path, head: &str, trailer: &str, refs: &CommitRefs) {
    let Some(dir) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    crate::loader::cache::ensure_ignored(dir);
    let entry = Entry {
        format: FORMAT_VERSION,
        head: head.to_owned(),
        trailer: trailer.to_owned(),
        refs,
    };
    if let Ok(json) = serde_json::to_string(&entry) {
        let _ = crate::fs_atomic::write_atomic(path, json.as_bytes());
    }
}

#[cfg(test)]
#[path = "commit_refs_tests.rs"]
mod tests;
