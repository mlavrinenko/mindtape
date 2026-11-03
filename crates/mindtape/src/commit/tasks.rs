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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha1::{Digest, Sha1};

use crate::commit::packages::{PackageCache, working_packages};
use crate::commit::tree::{StateConfig, StateTree, TaskEntry};
use crate::commit::{CommitMsgError, git, task_stem};
use crate::config::Config;
use crate::graph::ready::Readiness;
use crate::loader::{LoadError, StatePackages};
use crate::model::Task;
use crate::validate::candidate::{Candidate, harvest_source_with};

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

/// A task file's cached evaluation, keyed by the state environment and blob.
type CachedTask = Result<Option<Task>, LoadError>;

/// One evaluated task file: same environment, same project path, same bytes.
#[derive(Clone, PartialEq, Eq, Hash)]
struct FileKey {
    env: String,
    rel: PathBuf,
    oid: String,
}

/// Loads commit states with a cache shared across a range and seeded from the
/// working-tree load for a `commit-msg` hook.
pub struct StateLoader<'a> {
    root: &'a Path,
    base: &'a [Task],
    config: &'a Config,
    configs: HashMap<String, Arc<StateConfig>>,
    packages: PackageCache,
    files: HashMap<FileKey, CachedTask>,
    states: HashMap<String, ProjectState>,
    seeded: bool,
}

impl<'a> StateLoader<'a> {
    /// A loader over `root`, whose working-tree load is `base` under `config`.
    #[must_use]
    pub fn new(root: &'a Path, base: &'a [Task], config: &'a Config) -> Self {
        Self {
            root,
            base,
            config,
            configs: HashMap::new(),
            packages: PackageCache::default(),
            files: HashMap::new(),
            states: HashMap::new(),
            seeded: false,
        }
    }

    /// Build every state `plan` names, reusing anything an earlier call
    /// already evaluated.
    ///
    /// # Errors
    /// [`CommitMsgError`] when git cannot read a state's files.
    pub fn load(&mut self, plan: &StatePlan) -> Result<CommitStates, CommitMsgError> {
        self.seed_working();
        Ok(CommitStates {
            head: self.state(plan, &plan.head)?,
            staged: self.state(plan, &plan.staged)?,
            worktree: plan
                .worktree
                .then(|| ProjectState::new(self.base.to_vec(), self.config, self.root, Vec::new())),
        })
    }

    /// The project `source` names in `plan`, evaluated against its own config
    /// and packages.
    fn state(
        &mut self,
        plan: &StatePlan,
        source: &StateSource,
    ) -> Result<ProjectState, CommitMsgError> {
        if let Some(found) = self.states.get(&source.rev) {
            return Ok(found.clone());
        }
        let tree = StateTree::new(self.root, plan, source);
        let config = self.config_for(&tree)?;
        let state_config = config.config.as_ref().unwrap_or(self.config);
        let packages = tree.packages_cached(state_config, &mut self.packages)?;
        let env = load_env(state_config, packages.as_ref());
        let (tasks, errors) = self.build_tasks(&tree, &env, &config, packages.as_ref())?;
        let state = ProjectState::new(tasks, state_config, self.root, errors);
        self.states.insert(source.rev.clone(), state.clone());
        Ok(state)
    }

    /// The state's config, built once per distinct project-config blob.
    fn config_for(&mut self, tree: &StateTree<'_>) -> Result<Arc<StateConfig>, CommitMsgError> {
        let Some(oid) = tree.config_oid()? else {
            return Ok(Arc::new(StateConfig {
                config: None,
                error: None,
            }));
        };
        if let Some(found) = self.configs.get(&oid) {
            return Ok(found.clone());
        }
        let text = tree.config_text()?.unwrap_or_default();
        let config = Arc::new(tree.parse_config(&text));
        self.configs.insert(oid, config.clone());
        Ok(config)
    }

    /// Evaluate the files `env` does not already hold and assemble the state's
    /// task list.
    fn build_tasks(
        &mut self,
        tree: &StateTree<'_>,
        env: &str,
        state: &StateConfig,
        packages: Option<&StatePackages>,
    ) -> Result<(Vec<Task>, Vec<LoadError>), CommitMsgError> {
        let config = state.config.as_ref().unwrap_or(self.config);
        let mut tasks = Vec::new();
        let mut errors: Vec<LoadError> = state.error.clone().into_iter().collect();
        let mut misses: Vec<(FileKey, TaskEntry)> = Vec::new();
        for entry in tree.task_entries(config)? {
            let key = FileKey {
                env: env.to_owned(),
                rel: entry.rel.clone(),
                oid: entry.oid.clone(),
            };
            match self.files.get(&key) {
                Some(found) => record(found.clone(), &mut tasks, &mut errors),
                None => misses.push((key, entry)),
            }
        }
        let paths: Vec<&str> = misses
            .iter()
            .map(|(_, entry)| entry.repo.as_str())
            .collect();
        let blobs = tree.read(&paths)?;
        for ((key, entry), blob) in misses.into_iter().zip(blobs) {
            let found = match blob {
                Some(text) => harvest_source_with(
                    self.root,
                    config,
                    Candidate {
                        rel: &entry.rel,
                        source: &text,
                    },
                    packages.cloned(),
                ),
                None => Ok(None),
            };
            self.files.insert(key, found.clone());
            record(found, &mut tasks, &mut errors);
        }
        Ok((tasks, errors))
    }

    /// Seed the cache from the working-tree load when its package and config
    /// environment matches a state's. A range near the working tree reuses
    /// those evaluations for every unchanged file; a state with different
    /// package bytes or config falls through to its own evaluation.
    fn seed_working(&mut self) {
        if self.seeded {
            return;
        }
        self.seeded = true;
        let packages = working_packages(self.root, self.config);
        // A pin that cannot be read leaves `working_packages` empty, and the
        // working load may have resolved through the filesystem pin instead.
        // Do not claim those evaluations for a git-tree state.
        if packages.is_none() && !self.config.typst.local_packages.is_empty() {
            return;
        }
        let env = load_env(self.config, packages.as_ref());
        for task in self.base {
            if let Some(oid) = working_oid(self.root, &task.file) {
                self.files.insert(
                    FileKey {
                        env: env.clone(),
                        rel: task.file.clone(),
                        oid,
                    },
                    Ok(Some(task.clone())),
                );
            }
        }
    }
}

/// Add one cached or fresh file evaluation to the state.
fn record(found: CachedTask, tasks: &mut Vec<Task>, errors: &mut Vec<LoadError>) {
    match found {
        Ok(Some(task)) => tasks.push(task),
        Ok(None) => {}
        Err(err) => errors.push(err),
    }
}

/// The config facts a harvest reads, plus the exact package bytes it resolves.
fn load_env(config: &Config, packages: Option<&StatePackages>) -> String {
    format!(
        "{:?}\0{:?}\0{:?}\0{}",
        config.tasks.roots,
        config.tasks.derive_tags,
        config.typst.local_packages,
        packages.map_or("", |packages| packages.fingerprint.as_str()),
    )
}

/// The git blob id of `rel`'s working-tree bytes, as `git hash-object` would
/// report it. `None` when the file cannot be read.
fn working_oid(root: &Path, rel: &Path) -> Option<String> {
    let bytes = std::fs::read(root.join(rel)).ok()?;
    let mut hasher = Sha1::new();
    hasher.update(format!("blob {}\0", bytes.len()).as_bytes());
    hasher.update(&bytes);
    Some(crate::loader::cache::hex(&hasher.finalize()))
}

#[cfg(test)]
#[path = "tasks_tests.rs"]
mod tests;
