//! Dependency readiness: a one-hop, status-only classification of a task set.
//!
//! A task is *ready* when it is non-terminal and every direct `depends-on`
//! target is terminal — it can be started now. It is *blocked* when some direct
//! target is non-terminal or names no loaded task. Terminal tasks are *settled*
//! (neither). The check is one hop, not transitive: a still-open grandparent
//! keeps the parent non-terminal, which keeps this task blocked anyway, so no
//! toposort is needed and dependency cycles cannot hang the classification.
//!
//! History-pointer targets (SWHID tombstones) are already absent from
//! [`Task::depends_on`], so a dependency retired by `cleanup` stops blocking.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::Config;
use crate::graph::DepIndex;
use crate::model::Task;

/// A task's readiness, derived from its direct `depends-on` targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadyState {
    /// Non-terminal with every direct dependency terminal — startable now.
    Ready,
    /// Non-terminal with some dependency non-terminal or unresolved.
    Blocked,
    /// Terminal — neither ready nor blocked.
    Settled,
}

/// Dependency-readiness over a task set: which tasks are ready vs blocked.
///
/// Build this once over the *full* loaded set with [`compute`](Self::compute): a
/// dependency hidden by a view filter (e.g. a `done` task under
/// `status != "done"`) must still count as satisfied, so a narrowed set would
/// misread every leaf as blocked. Membership is keyed by [`Task::file`], the
/// stable per-task identity.
#[derive(Debug, Clone, Default)]
pub struct Readiness {
    ready: HashSet<PathBuf>,
    blocked: HashSet<PathBuf>,
}

impl Readiness {
    /// An empty readiness: every query returns false, every task is settled. Use
    /// where no task-set context exists (single-file eval, per-task gating).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Classify every task in `tasks` by its direct `depends-on` targets, using
    /// `config`'s terminal status set.
    #[must_use]
    pub fn compute(tasks: &[Task], config: &Config) -> Self {
        let terminal = config.tasks.status.terminal();
        // Resolve dependency links against the shared graph index. History-pointer
        // targets are already absent from `depends_on`.
        let index = DepIndex::build(tasks);
        let mut out = Self::default();
        for task in tasks {
            if task.is_terminal(&terminal) {
                continue; // settled — neither ready nor blocked
            }
            let ready = task.depends_on.iter().all(|target| {
                index
                    .resolve(task, target)
                    .is_some_and(|dep| dep.is_terminal(&terminal))
            });
            if ready {
                out.ready.insert(task.file.clone());
            } else {
                out.blocked.insert(task.file.clone());
            }
        }
        out
    }

    /// Whether `task` is ready: non-terminal with every dependency terminal.
    #[must_use]
    pub fn is_ready(&self, task: &Task) -> bool {
        self.ready.contains(&task.file)
    }

    /// Whether `task` is blocked: non-terminal with an unmet dependency.
    #[must_use]
    pub fn is_blocked(&self, task: &Task) -> bool {
        self.blocked.contains(&task.file)
    }

    /// `task`'s readiness state.
    #[must_use]
    pub fn state(&self, task: &Task) -> ReadyState {
        if self.is_ready(task) {
            ReadyState::Ready
        } else if self.is_blocked(task) {
            ReadyState::Blocked
        } else {
            ReadyState::Settled
        }
    }
}

#[cfg(test)]
#[path = "ready_tests.rs"]
mod tests;
