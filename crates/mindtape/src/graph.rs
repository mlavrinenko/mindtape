//! Transitive dependency ordering over the task graph: topological sort with
//! cycle detection.
//!
//! Nodes are tasks keyed by [`Task::file`]; edges are the `depends-on` links
//! that resolve to a loaded task. This is the graph substrate that the one-hop
//! readiness query ([`crate::ready`]) and the transitive [`toposort`] both build
//! on: both resolve edges through the shared [`DepIndex`], differing only in what
//! they ask of it — readiness checks a single hop against the terminal set,
//! toposort walks the whole graph. A `depends-on` target that names no loaded
//! task (dangling, or escaping the root) is not an edge: it cannot join a cycle
//! and does not constrain the order.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use crate::model::{Task, resolve_link_path};

/// Index of a task set by file path, resolving raw `depends-on` links to the
/// loaded task they name. The shared substrate of the task graph: readiness and
/// topological ordering walk edges through it rather than each re-deriving link
/// resolution.
pub(crate) struct DepIndex<'a> {
    by_path: HashMap<&'a Path, &'a Task>,
}

impl<'a> DepIndex<'a> {
    /// Index `tasks` by their file path.
    pub(crate) fn build(tasks: &'a [Task]) -> Self {
        Self {
            by_path: tasks
                .iter()
                .map(|task| (task.file.as_path(), task))
                .collect(),
        }
    }

    /// The loaded task that `target` (a raw link written on `from`) resolves to.
    /// `None` when the link escapes the project root or names no loaded task.
    pub(crate) fn resolve(&self, from: &Task, target: &str) -> Option<&'a Task> {
        resolve_link_path(&from.file, target)
            .and_then(|path| self.by_path.get(path.as_path()).copied())
    }
}

/// A dependency cycle: the tasks on a `depends-on` loop, by id, in the order the
/// walk entered them. The loop closes from the last member back to the first;
/// [`Display`](fmt::Display) renders it as `a → b → c → a`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    /// Ids of the tasks forming the loop, in traversal order.
    pub members: Vec<String>,
}

impl fmt::Display for Cycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut members = self.members.iter();
        let Some(first) = members.next() else {
            return Ok(());
        };
        write!(f, "{first}")?;
        for member in members {
            write!(f, " \u{2192} {member}")?;
        }
        write!(f, " \u{2192} {first}")
    }
}

/// Topologically order `tasks` so each appears after every task it depends on.
///
/// In-set `depends-on` links define the edges; a target that names no loaded
/// task is ignored (see [`DepIndex::resolve`]). The order is deterministic:
/// independent tasks keep their input order, which the loader sorts by id.
///
/// # Errors
/// Returns a [`Cycle`] when the dependency graph contains a loop.
pub fn toposort(tasks: &[Task]) -> Result<Vec<&Task>, Cycle> {
    toposort_kind(tasks, "depends-on")
}

/// Topologically order `tasks` over edges of one `kind` only (e.g. `parent`,
/// or a custom kind flagged acyclic). Otherwise identical to [`toposort`],
/// which is this specialised to `"depends-on"`.
///
/// # Errors
/// Returns a [`Cycle`] when that kind's edges contain a loop.
pub fn toposort_kind<'a>(tasks: &'a [Task], kind: &str) -> Result<Vec<&'a Task>, Cycle> {
    let index = DepIndex::build(tasks);
    let mut walk = Walk::new(tasks.len());
    for task in tasks {
        if !walk.seen(task) {
            walk.visit(task, &index, kind)?;
        }
    }
    Ok(walk.order)
}

/// Per-node DFS state: a node on the current path (a cycle if re-entered) or one
/// fully emitted into the order.
enum Mark {
    OnPath,
    Done,
}

/// Depth-first post-order walk accumulating the topological `order` and the
/// active `path` used to reconstruct a cycle.
struct Walk<'a> {
    mark: HashMap<&'a Path, Mark>,
    path: Vec<&'a Task>,
    order: Vec<&'a Task>,
}

impl<'a> Walk<'a> {
    fn new(len: usize) -> Self {
        Self {
            mark: HashMap::with_capacity(len),
            path: Vec::new(),
            order: Vec::with_capacity(len),
        }
    }

    fn seen(&self, task: &Task) -> bool {
        self.mark.contains_key(task.file.as_path())
    }

    /// Visit `task`, recursing into its resolved edges of `kind` first so it
    /// lands in `order` after all of them. Errors with the loop when an edge
    /// target is already on the active path.
    fn visit(&mut self, task: &'a Task, index: &DepIndex<'a>, kind: &str) -> Result<(), Cycle> {
        self.mark.insert(task.file.as_path(), Mark::OnPath);
        self.path.push(task);
        for target in task.link_targets(kind) {
            let Some(dep) = index.resolve(task, target) else {
                continue;
            };
            match self.mark.get(dep.file.as_path()) {
                Some(Mark::Done) => {}
                Some(Mark::OnPath) => return Err(self.cycle_to(dep)),
                None => self.visit(dep, index, kind)?,
            }
        }
        self.path.pop();
        self.mark.insert(task.file.as_path(), Mark::Done);
        self.order.push(task);
        Ok(())
    }

    /// Reconstruct the cycle from the active path: the segment from the
    /// re-entered node `dep` to the current tip.
    fn cycle_to(&self, dep: &Task) -> Cycle {
        let start = self
            .path
            .iter()
            .position(|task| task.file == dep.file)
            .unwrap_or(0);
        Cycle {
            members: self
                .path
                .iter()
                .skip(start)
                .map(|task| task.id.clone())
                .collect(),
        }
    }
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
