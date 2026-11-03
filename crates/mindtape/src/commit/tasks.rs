//! The project as a commit sees it: its tasks as of `HEAD`, the index and the
//! working tree — the three states a `scope = "task"` commit rule reads
//! through `head(...)`, `staged(...)` and `worktree(...)`.
//!
//! A state loads every task file from its own git tree — the index for an
//! empty revision — and evaluates it like a file that is not on disk
//! ([`crate::validate::candidate`]). A file the state lacks is absent from it,
//! so `exists` reads false there. The state's own `.mindtape/config.toml`
//! layers over the XDG and ancestor-default config layers, and its package
//! pins resolve against that tree's files, so an unchanged task file still
//! resolves `@local` against that tree's `lib/`.
//!
//! [`StateLoader`] caches evaluations by task-file blob id, package-tree
//! fingerprint and the config facts a harvest reads. A range reuses one loader
//! across commits, and a `commit-msg` hook seeds it from the loaded working
//! tree, so only files whose bytes or package/config environment changed are
//! evaluated again.

pub use crate::commit::state_loader::StateLoader;
use crate::commit::{CommitMsgError, git, task_stem};
use crate::config::Config;
use crate::graph::ready::Readiness;
use crate::loader::LoadError;
use crate::model::Task;
use std::path::Path;

/// Where one state's versions come from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateSource {
    /// The revision `git cat-file` reads `<rev>:<path>` from; empty reads the
    /// index (`:<path>`).
    pub rev: String,
    /// Always empty. Kept for API compatibility: a state reads its whole tree,
    /// so no caller supplies or consumes a path list.
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
    ///
    /// # Errors
    /// [`CommitMsgError`] when git cannot answer.
    pub fn index(root: &Path) -> Result<Self, CommitMsgError> {
        // `HEAD`, or its parent while `git commit --amend` replaces it.
        let base = crate::commit::amend::base_rev(root)?;
        Ok(Self {
            root_prefix: git(root, &["rev-parse", "--show-prefix"])?
                .trim()
                .to_owned(),
            staged: StateSource {
                rev: String::new(),
                differs: Vec::new(),
            },
            head: StateSource {
                rev: base,
                differs: Vec::new(),
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
    /// each open task's `waits-on` checkers asked from `root`. `config` is the
    /// state's own config, so readiness and query annotations read the same
    /// status legend and rules as the harvest did.
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
        StateLoader::new(root, base, config).load(plan)
    }
}

#[cfg(test)]
#[path = "tasks_tests.rs"]
mod tests;
