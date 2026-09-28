//! Graph-derived per-task ranks: where a task sits in the dependency DAG
//! ([`Task::topo`]) and how much work waits behind it ([`Task::blocks`]) —
//! plus how far its children have got ([`Task::children`],
//! [`Task::children_done`]), counted over `parent` edges by [`crate::children`].
//!
//! `links:` already carries the true working order of a backlog, but a scoring
//! framework ranks a task on its own properties alone — so the decision five
//! other tasks depend on scores low for being expensive and sorts last. These
//! two ranks are the graph read back out as numbers, so `--sort topo` and
//! `--sort blocks:desc` (and the filters `topo == 0`, `blocks > 3`) can order a
//! list the way it can actually be worked.
//!
//! # The two ranks
//!
//! `topo` is **position**: 0 for a task with no in-set blocking dependency,
//! else one more than the deepest task it depends on. A dependency's level is
//! strictly below its dependents', so ascending `topo` is a valid topological
//! order — and, being a level rather than a linear index, it leaves independent
//! tasks *tied*, which is what lets `--sort topo --sort priority:desc` mean
//! "graph order, priority within a layer". A linear position would admit no
//! tiebreak at all.
//!
//! `blocks` is **leverage**: how many non-terminal tasks transitively wait on
//! this one. It is the rank that actually reorders a backlog, since it measures
//! what finishing a task unlocks rather than where the task sits.
//!
//! # Edges
//!
//! Both read the *blocking* kinds ([`blocking_kinds`]) — `depends-on` plus any
//! custom kind a project flags `blocking = true` — resolved through the shared
//! [`DepIndex`], the same edge set [`crate::ready`] classifies one hop of. So
//! `ready`, `blocked`, `topo` and `blocks` never disagree about what an edge
//! means. A target naming no loaded task is not an edge; nor is a self-edge,
//! which constrains no order.
//!
//! # Status
//!
//! `topo` is status-blind: it is a fact about the graph, and a done dependency
//! still sits before its dependent in it. `blocks` is not — it counts only
//! non-terminal dependents and walks only *through* them, because a done task
//! is waiting on nothing and passes nothing on. A terminal task therefore
//! blocks 0. That makes the number shrink as work closes, which is the point:
//! leverage is a live measure of what is still waiting, not a historical one.
//!
//! # Cycles
//!
//! `mt check` rejects a `depends-on` cycle, but a sort must survive a tree that
//! has not been checked yet, so neither rank can hang or panic on one. `blocks`
//! is a reachability walk with a visited set, so a loop terminates by
//! construction. `topo` is computed by Kahn's algorithm: a task on a cycle
//! never reaches zero unresolved dependencies, so it keeps whatever level the
//! resolved part of its history gave it. Both stay deterministic — every walk
//! is seeded in the loader's id order.
//!
//! Neither walk is written as an unbounded `while let … pop_front()`. Each
//! runs a `for` over the node count, which is the exact maximum a correct pass
//! can pop, so the no-hang promise above is structural rather than argued from
//! the visited check being right. That is also what a mutation sweep found:
//! two mutants of these loops reached an unbounded state and timed out instead
//! of failing a test.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use crate::config::Config;
use crate::graph::DepIndex;
use crate::link::blocking_kinds;
use crate::model::Task;

/// Compute [`Task::topo`] and [`Task::blocks`] over `tasks` and write them back
/// onto each task.
///
/// Call once over the *full* loaded set, the way [`crate::ready::Readiness`] is
/// computed: both ranks are facts about the whole graph, so deriving them from
/// a narrowed set would count a dependency a view happens to hide as absent.
/// The loader is the single caller for exactly that reason.
pub(crate) fn annotate(tasks: &mut [Task], config: &Config) {
    let computed = ranks(tasks, config);
    let counts = child_counts(tasks, config);
    for ((task, rank), (children, done)) in tasks.iter_mut().zip(computed).zip(counts) {
        task.topo = rank.topo;
        task.blocks = rank.blocks;
        task.children = children;
        task.children_done = done;
    }
}

/// `(children, terminal children)` for every task, parallel to `tasks`, over
/// the `parent` edges [`crate::children`] resolves.
fn child_counts(tasks: &[Task], config: &Config) -> Vec<(u32, u32)> {
    let terminal = config.tasks.status.terminal();
    let by_parent = crate::children::children_by_parent(tasks);
    tasks
        .iter()
        .map(|task| {
            let children = by_parent
                .get(task.file.as_path())
                .map_or(&[][..], Vec::as_slice);
            let done = children
                .iter()
                .filter(|child| child.is_terminal(&terminal))
                .count();
            (count(children.len()), count(done))
        })
        .collect()
}

/// A task count as `u32`, saturating: no loaded set comes near the bound.
fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// One task's place in the dependency graph.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Rank {
    topo: u32,
    blocks: u32,
}

/// Both ranks for every task, parallel to `tasks` — positional rather than
/// keyed, so [`annotate`]'s write-back needs no second lookup and cannot
/// mismatch a task to another's rank.
fn ranks(tasks: &[Task], config: &Config) -> Vec<Rank> {
    let graph = Graph::build(tasks, config);
    let level = graph.levels();
    let leverage = graph.leverage();
    tasks
        .iter()
        .map(|task| Rank {
            topo: level.get(task.file.as_path()).copied().unwrap_or_default(),
            blocks: leverage
                .get(task.file.as_path())
                .copied()
                .unwrap_or_default(),
        })
        .collect()
}

/// The blocking-edge adjacency of a task set, held both ways round, plus which
/// tasks are terminal. Keyed by [`Task::file`], the stable per-task identity
/// [`crate::ready`] keys on too.
struct Graph<'a> {
    /// Task files in the set's own order (the loader sorts by id), so every
    /// walk below is seeded deterministically.
    order: Vec<&'a Path>,
    /// A task → the in-set tasks it depends on. One entry per distinct pair,
    /// so the count is the number of dependencies to resolve.
    blocked_by: HashMap<&'a Path, Vec<&'a Path>>,
    /// A task → the in-set tasks that depend on it (`blocked_by` reversed).
    blocking: HashMap<&'a Path, Vec<&'a Path>>,
    terminal: HashSet<&'a Path>,
}

impl<'a> Graph<'a> {
    fn build(tasks: &'a [Task], config: &Config) -> Self {
        let index = DepIndex::build(tasks);
        let kinds = blocking_kinds(&config.tasks.links);
        let terminal_names = config.tasks.status.terminal();
        let mut graph = Self {
            order: tasks.iter().map(|task| task.file.as_path()).collect(),
            blocked_by: HashMap::new(),
            blocking: HashMap::new(),
            terminal: tasks
                .iter()
                .filter(|task| task.is_terminal(&terminal_names))
                .map(|task| task.file.as_path())
                .collect(),
        };
        for task in tasks {
            for kind in &kinds {
                for target in task.link_targets(kind) {
                    if let Some(dep) = index.resolve(task, target) {
                        graph.add_edge(task.file.as_path(), dep.file.as_path());
                    }
                }
            }
        }
        graph
    }

    /// Record that `from` depends on `to`, both directions at once. A self-edge
    /// is dropped (it orders nothing) and a pair already recorded is not
    /// doubled, so `blocked_by`'s length is a true count of what must resolve
    /// before a task — which is what [`Self::levels`] counts down.
    fn add_edge(&mut self, from: &'a Path, to: &'a Path) {
        if from == to {
            return;
        }
        let deps = self.blocked_by.entry(from).or_default();
        if deps.contains(&to) {
            return;
        }
        deps.push(to);
        self.blocking.entry(to).or_default().push(from);
    }

    /// Every task's dependency level, by Kahn's algorithm over the resolved
    /// edges: a task enters the queue once all of its dependencies have left
    /// it, carrying one more than the deepest of them. A task on (or fed only
    /// by) a cycle never enters, and keeps the level its resolved dependencies
    /// gave it — see the module doc.
    fn levels(&self) -> HashMap<&'a Path, u32> {
        let mut level: HashMap<&'a Path, u32> = self.order.iter().map(|&file| (file, 0)).collect();
        let mut unresolved: HashMap<&'a Path, usize> = self
            .order
            .iter()
            .map(|&file| (file, self.blocked_by.get(file).map_or(0, Vec::len)))
            .collect();
        let mut queue: VecDeque<&'a Path> = self
            .order
            .iter()
            .copied()
            .filter(|file| unresolved.get(file) == Some(&0))
            .collect();
        // A `for` over a fixed range, not an unbounded `while let`: a task's
        // countdown reaches zero exactly once, so a correct pass pops at most
        // one entry per task, and holding the loop to that means a defect in
        // the countdown ends the pass with a wrong level a test can catch
        // rather than re-queueing forever. Two mutants of this loop reached
        // exactly that unbounded state — see
        // `docs/contributing/mutation-testing.md` on why a reproducible
        // timeout is a finding.
        for _ in 0..self.order.len() {
            let Some(file) = queue.pop_front() else {
                break;
            };
            let next = level.get(file).copied().unwrap_or_default() + 1;
            for &dependent in self.blocking.get(file).into_iter().flatten() {
                let slot = level.entry(dependent).or_default();
                *slot = (*slot).max(next);
                if let Some(left) = unresolved.get_mut(dependent) {
                    *left = left.saturating_sub(1);
                    if *left == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }
        level
    }

    /// Every task's `blocks` count: the non-terminal tasks reachable from it
    /// along `blocking` edges, walking only through non-terminal ones. A
    /// terminal task scores 0 without a walk.
    fn leverage(&self) -> HashMap<&'a Path, u32> {
        let mut out = HashMap::with_capacity(self.order.len());
        // One pair of buffers reused across every walk rather than allocated
        // per start — the walks are independent, and `waiting_on` resets them.
        let mut seen: HashSet<&'a Path> = HashSet::new();
        let mut queue: VecDeque<&'a Path> = VecDeque::new();
        for &start in &self.order {
            let count = if self.terminal.contains(start) {
                0
            } else {
                self.waiting_on(start, &mut seen, &mut queue)
            };
            out.insert(start, count);
        }
        out
    }

    /// How many non-terminal tasks `start` transitively blocks: a breadth-first
    /// walk out along `blocking`, entering a task once and never entering a
    /// terminal one. `seen`/`queue` are caller-owned scratch, reset here.
    fn waiting_on(
        &self,
        start: &'a Path,
        seen: &mut HashSet<&'a Path>,
        queue: &mut VecDeque<&'a Path>,
    ) -> u32 {
        seen.clear();
        seen.insert(start);
        queue.clear();
        queue.push_back(start);
        let mut count = 0;
        // Bounded for the same reason [`Self::levels`]'s loop is: `seen` admits
        // each task once, so a correct walk pops at most one entry per task,
        // and a `for` over that range turns a defect in the visited check into
        // a wrong count a test catches rather than a hang.
        for _ in 0..self.order.len() {
            let Some(file) = queue.pop_front() else {
                break;
            };
            for &dependent in self.blocking.get(file).into_iter().flatten() {
                if self.terminal.contains(dependent) || !seen.insert(dependent) {
                    continue;
                }
                count += 1;
                queue.push_back(dependent);
            }
        }
        count
    }
}

#[cfg(test)]
#[path = "dep_rank_tests.rs"]
mod tests;
