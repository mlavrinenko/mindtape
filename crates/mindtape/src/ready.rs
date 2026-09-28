//! Dependency readiness: a one-hop, status-only classification of a task set.
//!
//! A task is *ready* when it is non-terminal and every direct blocking-kind
//! edge target is terminal — it can be started now. `depends-on` is the only
//! built-in kind flagged blocking (see [`crate::link::blocking_kinds`]); a
//! project's custom `[[tasks.links.kinds]]` can opt another kind in. It is
//! *blocked* when some direct target is non-terminal or names no loaded task.
//! Terminal tasks are *settled* (neither). The check is one hop, not
//! transitive: a still-open grandparent keeps the parent non-terminal, which
//! keeps this task blocked anyway, so no toposort is needed and dependency
//! cycles cannot hang the classification.
//!
//! History-pointer targets (SWHID tombstones) are already absent from
//! [`Task::links`], so a dependency retired by `cleanup` stops blocking.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::filter_aggregate::LoadedSet;
use crate::graph::DepIndex;
use crate::link::blocking_kinds;
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

/// One unfinished direct blocking-kind target: what a blocked task waits on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blocker {
    /// The target's stem, or the edge's own target stem when it names no
    /// loaded task.
    pub stem: String,
    /// The target's status name; `None` when the target is not loaded or
    /// carries no status.
    pub status: Option<String>,
    /// The blocking edge kind.
    pub kind: String,
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
    blocked: HashMap<PathBuf, Vec<Blocker>>,
    /// The same set, snapshotted for `count(...)`; see [`LoadedSet`].
    set: LoadedSet,
}

impl Readiness {
    /// An empty readiness: every query returns false, every task is settled. Use
    /// where no task-set context exists (single-file eval, per-task gating).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Classify every task in `tasks` by its direct blocking-kind edge targets,
    /// using `config`'s terminal status set and link-kind registry.
    #[must_use]
    pub fn compute(tasks: &[Task], config: &Config) -> Self {
        let terminal = config.tasks.status.terminal();
        // Resolve dependency links against the shared graph index. History-pointer
        // targets are already absent from `links`.
        let index = DepIndex::build(tasks);
        let kinds = blocking_kinds(&config.tasks.links);
        let mut out = Self {
            set: LoadedSet::new(tasks),
            ..Self::default()
        };
        for task in tasks {
            if task.is_terminal(&terminal) {
                continue; // settled — neither ready nor blocked
            }
            let blockers = collect_blockers(task, &index, &kinds, &terminal);
            if blockers.is_empty() {
                out.ready.insert(task.file.clone());
            } else {
                out.blocked.insert(task.file.clone(), blockers);
            }
        }
        out
    }

    /// The loaded set `count(...)` ranges over; no set at all for
    /// [`empty`](Self::empty).
    pub(crate) fn set(&self) -> &LoadedSet {
        &self.set
    }

    /// Whether `task` is ready: non-terminal with every dependency terminal.
    #[must_use]
    pub fn is_ready(&self, task: &Task) -> bool {
        self.ready.contains(&task.file)
    }

    /// Whether `task` is blocked: non-terminal with an unmet dependency.
    #[must_use]
    pub fn is_blocked(&self, task: &Task) -> bool {
        self.blocked.contains_key(&task.file)
    }

    /// What `task` waits on: its unfinished direct blocking-kind targets, in
    /// edge order. Empty unless the task is blocked.
    #[must_use]
    pub fn blockers(&self, task: &Task) -> &[Blocker] {
        self.blocked.get(&task.file).map_or(&[], Vec::as_slice)
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

/// What `task` waits on regardless of its own status: its unfinished direct
/// blocking-kind targets within `tasks`, in edge order. Unlike
/// [`Readiness::blockers`] this answers for a terminal task too, which is what
/// a guard on *entering* a status needs.
#[must_use]
pub fn blockers_of(tasks: &[Task], task: &Task, config: &Config) -> Vec<Blocker> {
    let terminal = config.tasks.status.terminal();
    let kinds = blocking_kinds(&config.tasks.links);
    collect_blockers(task, &DepIndex::build(tasks), &kinds, &terminal)
}

/// The unfinished direct `kinds` targets of `task`, resolved through `index`:
/// a target that is non-terminal or names no loaded task.
pub(crate) fn collect_blockers(
    task: &Task,
    index: &DepIndex<'_>,
    kinds: &[&str],
    terminal: &[String],
) -> Vec<Blocker> {
    let mut blockers = Vec::new();
    for &kind in kinds {
        for target in task.link_targets(kind) {
            let dep = index.resolve(task, target);
            if dep.is_some_and(|dep| dep.is_terminal(terminal)) {
                continue;
            }
            blockers.push(Blocker {
                stem: dep.map_or_else(|| target_stem(target), |dep| dep.id.clone()),
                status: dep.and_then(|dep| dep.status.as_ref().map(|s| s.name.clone())),
                kind: kind.to_owned(),
            });
        }
    }
    blockers
}

/// Why entering `status` is refused while `task` waits on `blockers`:
/// `` `done` requires an unblocked task; waits on: a (proposed) `` — shared
/// by the `mt flip` refusal and the `mt check` warning.
#[must_use]
pub fn unblocked_violation(status: &str, blockers: &[Blocker]) -> String {
    format!(
        "`{status}` requires an unblocked task; {}",
        waits_on(blockers)
    )
}

/// `blockers` as one line — `waits on: a (proposed), b (wip)` — or empty when
/// there are none. A target with no known status reads `unknown`.
#[must_use]
pub fn waits_on(blockers: &[Blocker]) -> String {
    if blockers.is_empty() {
        return String::new();
    }
    format!("waits on: {}", list_targets(blockers))
}

/// `targets` as `a (proposed), b (wip)`; a target with no known status reads
/// `unknown`.
pub(crate) fn list_targets(targets: &[Blocker]) -> String {
    let list: Vec<String> = targets
        .iter()
        .map(|b| format!("{} ({})", b.stem, b.status.as_deref().unwrap_or("unknown")))
        .collect();
    list.join(", ")
}

/// The stem an unresolved edge target names: its file stem, else the target
/// verbatim.
fn target_stem(target: &str) -> String {
    Path::new(target).file_stem().map_or_else(
        || target.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
#[path = "ready_tests.rs"]
mod tests;
