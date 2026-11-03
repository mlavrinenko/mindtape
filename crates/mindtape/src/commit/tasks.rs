//! The project as a commit sees it: its tasks as of `HEAD`, the index and the
//! working tree — the three states a `scope = "task"` commit rule reads
//! through `head(...)`, `staged(...)` and `worktree(...)`.
//!
//! One load, three views. The loaded project is the working tree; each other
//! state loads every task file from its own git tree — the index for an empty
//! revision — through one `git cat-file --batch` and evaluates it like a file
//! that is not on disk ([`crate::validate::candidate`]). A file the state lacks
//! is absent from it, so `exists` reads false there. The state's own
//! `.mindtape/config.toml` names its task roots and package pins, so an
//! unchanged task file still resolves `@local` against that tree's `lib/`.
//!
//! Where a state's versions come from is a [`StateSource`]: a git revision and
//! the paths it may differ at. [`StatePlan::index`] builds the hook's plan —
//! the index as `staged`, `HEAD` as `head`, the working tree kept. A plan for
//! a past commit ([`crate::commit::range`]) swaps the sources — the commit as
//! `staged`, its first parent as `head`, no working tree — and loads through
//! the same [`CommitStates::load`].

use std::collections::BTreeSet;
use std::path::Path;

use crate::commit::tree::StateTree;
use crate::commit::{CommitMsgError, changed_paths, git, git_optional, task_stem};
use crate::config::Config;
use crate::git::state::GitStatus;
use crate::graph::ready::Readiness;
use crate::loader::LoadError;
use crate::model::Task;
use crate::validate::candidate::{Candidate, harvest_source_with};

/// Where one state's versions of the files it differs at come from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateSource {
    /// The revision `git cat-file` reads `<rev>:<path>` from; empty reads the
    /// index (`:<path>`).
    pub rev: String,
    /// Every repository-relative path at which this state may differ from the
    /// loaded working tree. The plan records it, though a state reads its whole
    /// tree rather than only these paths.
    pub differs: Vec<String>,
}

/// Which states a commit is checked against, and where each comes from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatePlan {
    /// The project root's path within the repository (`git rev-parse
    /// --show-prefix`): empty at the top level, else `/`-terminated.
    pub root_prefix: String,
    /// The state before the commit.
    pub head: StateSource,
    /// The state the commit records.
    pub staged: StateSource,
    /// Whether the loaded working tree is a state of its own — false for a
    /// past commit, which has none.
    pub worktree: bool,
}

impl StatePlan {
    /// The plan a `commit-msg` hook checks: the index is what the commit
    /// records, `HEAD` what it follows, and the working tree stays readable.
    /// One `git status` names every path either differs at.
    ///
    /// # Errors
    /// [`CommitMsgError`] when git cannot answer.
    pub fn index(root: &Path) -> Result<Self, CommitMsgError> {
        let root_prefix = git(root, &["rev-parse", "--show-prefix"])?;
        let status = GitStatus::read_unpaired(root).map_err(|stderr| CommitMsgError::Git {
            command: "status".to_owned(),
            stderr,
        })?;
        // `HEAD`, or its parent while `git commit --amend` replaces it; the
        // paths where that differs from `HEAD` join the ones `status` names.
        let base = crate::commit::amend::base_rev(root)?;
        // No `HEAD` before the first commit: the base is the empty tree, and
        // there is no `HEAD` to diff it against.
        let head_exists =
            git_optional(root, &["rev-parse", "--verify", "--quiet", "HEAD"])?.is_some();
        let behind_head = if base == "HEAD" || !head_exists {
            Vec::new()
        } else {
            changed_paths(root, &[&base, "HEAD"])?
        };
        let paths = |pick: fn(bool, bool) -> bool| {
            status
                .entries()
                .iter()
                .filter(|entry| pick(entry.staged, entry.worktree))
                .map(|entry| entry.path.clone())
                .collect()
        };
        Ok(Self {
            root_prefix: root_prefix.trim().to_owned(),
            // The index against the working tree.
            staged: StateSource {
                rev: String::new(),
                differs: paths(|_, worktree| worktree),
            },
            // The base differs from the working tree wherever either step does.
            head: StateSource {
                rev: base,
                differs: paths(|staged, worktree| staged || worktree)
                    .into_iter()
                    .chain(behind_head)
                    .collect::<BTreeSet<String>>()
                    .into_iter()
                    .collect(),
            },
            worktree: true,
        })
    }
}

/// One state of the project: its tasks, and the readiness computed over them
/// for `ready`, `blocked` and `count(...)`.
#[derive(Debug, Clone, Default)]
pub struct ProjectState {
    /// Every task the state holds, in id order.
    pub tasks: Vec<Task>,
    /// Readiness over [`Self::tasks`].
    pub readiness: Readiness,
    /// Task files of this state that failed to evaluate — absent from
    /// [`Self::tasks`].
    pub errors: Vec<LoadError>,
}

impl ProjectState {
    /// A state holding `tasks`, ranked and classified as a load would, with
    /// each open task's `waits-on` checkers asked from `root`.
    pub(crate) fn new(
        mut tasks: Vec<Task>,
        config: &Config,
        root: &Path,
        errors: Vec<LoadError>,
    ) -> Self {
        tasks.sort_by(|left, right| left.id.cmp(&right.id));
        crate::waits::annotate(&mut tasks, config, root);
        crate::estimate_queries::annotate(&mut tasks, config, root);
        crate::graph::dep_rank::annotate(&mut tasks, config);
        let readiness = Readiness::compute(&tasks, config);
        Self {
            tasks,
            readiness,
            errors,
        }
    }

    /// A one-task state holding [`Task::default`], whose stem is empty — the
    /// load-time dry run's.
    pub(crate) fn probe(config: &Config) -> Self {
        Self::new(vec![Task::default()], config, Path::new(""), Vec::new())
    }

    /// The task whose file stem is `stem`, if this state holds one.
    #[must_use]
    pub fn task(&self, stem: &str) -> Option<&Task> {
        self.tasks.iter().find(|task| task_stem(task) == stem)
    }
}

/// The states a commit is checked against.
#[derive(Debug, Clone, Default)]
pub struct CommitStates {
    /// The project before the commit.
    pub head: ProjectState,
    /// The project the commit records.
    pub staged: ProjectState,
    /// The project on disk, when there is one to read.
    pub worktree: Option<ProjectState>,
}

impl CommitStates {
    /// Build every state `plan` names from its own tree, and the working tree
    /// from `base`, the loaded project.
    ///
    /// # Errors
    /// [`CommitMsgError`] when git cannot read a state's files.
    pub fn load(
        root: &Path,
        base: &[Task],
        config: &Config,
        plan: &StatePlan,
    ) -> Result<Self, CommitMsgError> {
        let overlay = |source: &StateSource| overlay(root, config, plan, source);
        Ok(Self {
            head: overlay(&plan.head)?,
            staged: overlay(&plan.staged)?,
            worktree: plan
                .worktree
                .then(|| ProjectState::new(base.to_vec(), config, root, Vec::new())),
        })
    }
}

/// Every task file `source`'s tree holds, evaluated against that tree's own
/// config pins and package files rather than the working tree's.
fn overlay(
    root: &Path,
    config: &Config,
    plan: &StatePlan,
    source: &StateSource,
) -> Result<ProjectState, CommitMsgError> {
    let tree = StateTree::new(root, plan, source);
    let state = tree.config()?;
    let state_config = state.config.as_ref().unwrap_or(config);
    let files = tree.task_files(state_config)?;
    let paths: Vec<&str> = files.iter().map(|(path, _)| path.as_str()).collect();
    let packages = tree.packages(state_config)?;
    let mut tasks = Vec::new();
    let mut errors: Vec<LoadError> = state.error.into_iter().collect();
    for ((_, rel), blob) in files.iter().zip(tree.read(&paths)?) {
        let Some(text) = blob else { continue };
        let candidate = Candidate { rel, source: &text };
        match harvest_source_with(root, state_config, candidate, packages.clone()) {
            Ok(Some(task)) => tasks.push(task),
            Ok(None) => {}
            Err(err) => errors.push(err),
        }
    }
    Ok(ProjectState::new(tasks, config, root, errors))
}

#[cfg(test)]
#[path = "tasks_tests.rs"]
mod tests;
