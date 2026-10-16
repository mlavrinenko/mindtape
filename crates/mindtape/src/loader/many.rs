//! Load several projects at once: each root's config and tasks on a scoped
//! thread of its own, so a set of projects loads in about the time of its
//! slowest member rather than the sum of them all.

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
/// member's error; the others load regardless.
///
/// A thread that panics resumes its panic here, as a single load would.
#[must_use]
pub fn load_many(roots: &[PathBuf]) -> Vec<Result<Loaded, ConfigError>> {
    std::thread::scope(|scope| {
        let handles: Vec<_> = roots
            .iter()
            .map(|root| scope.spawn(move || load_one(root)))
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

/// The config of `root`, then its tasks under that config.
fn load_one(root: &Path) -> Result<Loaded, ConfigError> {
    let config = load_config(root)?;
    let outcome = load_project(root, &config);
    Ok(Loaded { config, outcome })
}

#[cfg(test)]
#[path = "many_tests.rs"]
mod tests;
