//! Load many projects at once: each root's config and tasks on a scoped thread
//! of its own. A set of projects then loads in about the time of its slowest
//! member rather than the sum of them all.

use std::path::{Path, PathBuf};

use crate::config::{Config, ConfigError, load_config};
use crate::loader::{LoadOutcome, load_project};

/// One project [`load_many`] loaded: its config and what its tasks gave.
#[derive(Debug)]
pub struct Loaded {
    /// The project's merged config.
    pub config: Config,
    /// Its tasks, per-file errors and Typst warnings.
    pub outcome: LoadOutcome,
}

/// Load every project in `roots` in parallel, one scoped thread each, and
/// return them in the order given. A config that fails to load is that
/// member's error, and the others load regardless.
///
/// A thread that panics resumes its panic here, as a single load would.
#[must_use]
pub fn load_many(roots: &[PathBuf]) -> Vec<Result<Loaded, ConfigError>> {
    map_parallel(roots.iter().collect(), |root| load_root(root))
}

/// Run `work` on each item on a scoped thread of its own, and return the
/// results in the order the items came. A thread that panics resumes its
/// panic here.
pub fn map_parallel<T: Send, R: Send>(items: Vec<T>, work: impl Fn(T) -> R + Sync) -> Vec<R> {
    let work = &work;
    std::thread::scope(|scope| {
        let handles: Vec<_> = items
            .into_iter()
            .map(|item| scope.spawn(move || work(item)))
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}

/// Run `work` over `items` split into at most `shards` contiguous chunks, one
/// scoped thread each, and return the results in chunk order. A thread that
/// panics resumes its panic here.
///
/// [`map_parallel`] spawns one thread per item, which is right for a handful of
/// projects and wrong for a backlog of thousands of files. This splits the work
/// into one chunk per core instead. The thread count then stays bounded however
/// long the list is.
pub(crate) fn map_chunks<'a, T: Sync, R: Send>(
    items: &'a [T],
    shards: usize,
    work: impl Fn(&'a [T]) -> R + Sync,
) -> Vec<R> {
    let size = items.len().div_ceil(shards.max(1)).max(1);
    map_parallel(items.chunks(size).collect(), work)
}

/// How many chunks to split `len` items into: one per core, and one chunk below
/// `MIN_PARALLEL`, where spawning threads costs more than the work they share.
pub(crate) fn shards(len: usize) -> usize {
    /// Below this many items a load is a few milliseconds, and the threads that
    /// would split it cost about as much as it does.
    const MIN_PARALLEL: usize = 128;
    if len < MIN_PARALLEL {
        return 1;
    }
    std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(len)
}

/// The config of `root`, then its tasks under that config.
fn load_root(root: &Path) -> Result<Loaded, ConfigError> {
    let config = load_config(root)?;
    let outcome = load_project(root, &config);
    Ok(Loaded { config, outcome })
}

#[cfg(test)]
#[path = "many_tests.rs"]
mod tests;
