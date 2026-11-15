//! `count(<expr>)`: how many loaded tasks match `<expr>`.
//!
//! The argument ranges over the whole loaded set, never a listing's narrowed
//! rows. The rule `ready` follows the same path, for the same reason.
//! [`crate::graph::ready::Readiness::compute`] fixes the set before any filter
//! runs, and carries a [`LoadedSet`] alongside its dependency classification.
//!
//! Inside `count` every field reads the task under count, and the result never
//! depends on the task the outer expression evaluates. That is what makes it
//! cacheable. The code evaluates each distinct argument over the set once per
//! command and reuses the number. A rule asserting `count(...)` on every task
//! then stays linear in the set rather than quadratic.
//!
//! Where no set exists ([`crate::graph::ready::Readiness::empty`]: `required`,
//! a gate, a single-file eval) `count` is an error rather than zero. Zero is a
//! real answer, and reading it where the set never loaded would pass a cap it
//! never measured.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use evalexpr::{DefaultNumericTypes, Node};

use crate::filter::FilterError;
use crate::model::Task;

/// The loaded set `count` ranges over, and the per-command cache of its
/// answers keyed by the argument's canonical text. Cloning shares both, so a
/// clone of the owning [`crate::graph::ready::Readiness`] never recomputes.
#[derive(Debug, Clone, Default)]
pub(crate) struct LoadedSet {
    /// `None` when there is no set context at all, as distinct from an empty
    /// loaded set.
    tasks: Option<Arc<[Task]>>,
    counts: Arc<Mutex<HashMap<String, usize>>>,
    passes: Arc<AtomicUsize>,
}

impl LoadedSet {
    /// A snapshot of `tasks` to count over.
    pub(crate) fn new(tasks: &[Task]) -> Self {
        Self {
            tasks: Some(tasks.into()),
            ..Self::default()
        }
    }

    /// The loaded tasks. `None` with no set context.
    pub(crate) fn tasks(&self) -> Option<&[Task]> {
        self.tasks.as_deref()
    }

    /// How many passes the code has made over the set, one per distinct
    /// argument, however many tasks asked.
    #[cfg(test)]
    pub(crate) fn passes(&self) -> usize {
        self.passes.load(Ordering::Relaxed)
    }

    /// How many tasks satisfy `arg`, `matches` evaluating it against one.
    ///
    /// On the first request for an argument the code walks the set once and
    /// caches the answer. It evaluates `probe` too when the set is empty, so an
    /// unknown field inside the argument still surfaces at a load-time dry run
    /// that has nothing to count. The evaluator runs without a lock, since an
    /// argument may itself hold a `count`.
    ///
    /// # Errors
    /// [`FilterError::Invalid`] with no loaded set, and otherwise whatever
    /// evaluating the argument raises.
    pub(crate) fn count(
        &self,
        arg: &Node<DefaultNumericTypes>,
        probe: &Task,
        matches: impl Fn(&Task) -> Result<bool, FilterError>,
    ) -> Result<usize, FilterError> {
        let Some(tasks) = &self.tasks else {
            return Err(FilterError::Invalid(
                "count() reads every task, and this expression sees only one".to_owned(),
            ));
        };
        let key = arg.to_string();
        if let Some(hit) = self.cache().get(&key) {
            return Ok(*hit);
        }
        self.passes.fetch_add(1, Ordering::Relaxed);
        if tasks.is_empty() {
            matches(probe)?;
        }
        let mut total = 0;
        for task in tasks.iter() {
            total += usize::from(matches(task)?);
        }
        self.cache().insert(key, total);
        Ok(total)
    }

    fn cache(&self) -> std::sync::MutexGuard<'_, HashMap<String, usize>> {
        // A poisoned cache still holds only finished counts.
        self.counts.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
#[path = "aggregate_tests.rs"]
mod tests;
