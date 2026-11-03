//! Caches for loading commit states across a range or a `commit-msg` hook.
//!
//! A linear range only needs the previous state: its unchanged tasks are
//! reused by blob id, and changed files are evaluated again. The working-tree
//! load seeds the same reuse map for a hook, so a clean index does not
//! re-evaluate the whole backlog.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha1::{Digest, Sha1};

use crate::commit::CommitMsgError;
use crate::commit::packages::{PackageCache, working_packages};
use crate::commit::tasks::{CommitStates, ProjectState, StatePlan, StateSource};
use crate::commit::tree::{StateConfig, StateTree, TaskEntry, TaskSource};
use crate::config::Config;
use crate::loader::{LoadError, StatePackages};
use crate::model::Task;
use crate::validate::candidate::{Candidate, harvest_source_with};

/// What one task file produced in a state.
#[derive(Clone, Copy)]
enum Outcome {
    /// The index of its task in the state's task list.
    Task(usize),
    /// It evaluated cleanly to no task marker.
    Missing,
    /// It failed to evaluate, and so is never reused.
    Failed,
}

/// One task file's blob and outcome in a state.
struct SeenFile {
    oid: String,
    mode: String,
    outcome: Outcome,
}

/// Config discovery directories a test injects instead of reading the machine.
struct ConfigDirs {
    home: Option<PathBuf>,
    xdg: Option<PathBuf>,
    pack_cache: Option<PathBuf>,
}

/// One cached state with its load environment and file outcomes.
type LastState = (
    StateKey,
    Arc<ProjectState>,
    String,
    HashMap<PathBuf, SeenFile>,
);

/// Which tree and project prefix a cached state belongs to.
#[derive(Clone, PartialEq, Eq)]
struct StateKey {
    root_prefix: String,
    rev: String,
}

/// The tasks and file outcomes a state may reuse.
struct Reuse<'a> {
    tasks: &'a [Task],
    files: &'a HashMap<PathBuf, SeenFile>,
    /// True for the working-tree load, whose root choice must be checked.
    seeded: bool,
}

/// One evaluated task file in a new state.
struct Seen {
    entry: TaskEntry,
    outcome: Outcome,
}

/// The result of evaluating one state's files.
struct Build {
    tasks: Vec<Task>,
    errors: Vec<LoadError>,
    seen: Vec<Seen>,
}

/// What a new state needs to evaluate its files.
struct Request<'a, 't> {
    tree: &'a StateTree<'t>,
    state: &'a StateConfig,
    packages: Option<&'a StatePackages>,
    reuse: Option<Reuse<'a>>,
}

/// Loads commit states with a cache shared across a range and seeded from the
/// working-tree load for a `commit-msg` hook.
pub struct StateLoader<'a> {
    root: &'a Path,
    base: &'a [Task],
    config: &'a Config,
    /// The last config parsed, keyed by its project-config blob (empty for a
    /// missing config).
    last_config: Option<(String, Arc<StateConfig>)>,
    packages: PackageCache,
    /// The working-tree load's OIDs and task indices, for a hook.
    working_seed: Option<(String, HashMap<PathBuf, SeenFile>)>,
    /// The previous state only: a linear range needs it as the next `head`.
    last_state: Option<LastState>,
    /// Injected only by tests, so no test reads the developer's config layers.
    layers: Option<ConfigDirs>,
    seeded: bool,
    #[cfg(test)]
    evaluations: std::cell::Cell<usize>,
}

impl<'a> StateLoader<'a> {
    /// A loader over `root`, whose working-tree load is `base` under `config`.
    #[must_use]
    pub fn new(root: &'a Path, base: &'a [Task], config: &'a Config) -> Self {
        Self {
            root,
            base,
            config,
            last_config: None,
            packages: PackageCache::default(),
            working_seed: None,
            last_state: None,
            layers: None,
            seeded: false,
            #[cfg(test)]
            evaluations: std::cell::Cell::new(0),
        }
    }

    /// The number of live evaluations, for a test that pins cache reuse.
    #[cfg(test)]
    pub(crate) fn evaluations(&self) -> usize {
        self.evaluations.get()
    }

    /// [`Self::new`] with config layer discovery injected, for tests that must
    /// not read the developer's global defaults.
    #[cfg(test)]
    pub(crate) fn with_layers(
        root: &'a Path,
        base: &'a [Task],
        config: &'a Config,
        layers: (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>),
    ) -> Self {
        let (home, xdg, pack_cache) = layers;
        let mut loader = Self::new(root, base, config);
        loader.layers = Some(ConfigDirs {
            home,
            xdg,
            pack_cache,
        });
        loader
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
        let state_key = StateKey {
            root_prefix: plan.root_prefix.clone(),
            rev: source.rev.clone(),
        };
        // An index state is never cached: its revision is empty and the index
        // can change between calls.
        if !source.rev.is_empty()
            && let Some((key, found, _, _)) = &self.last_state
            && *key == state_key
        {
            return Ok((**found).clone());
        }
        let tree = StateTree::new(self.root, plan, source);
        let config = self.config_for(&tree)?;
        let state_config = config.config.as_ref().unwrap_or(self.config);
        let packages = tree.packages_cached(state_config, &mut self.packages)?;
        let env = load_env(state_config, packages.as_ref());
        let reuse = self.reuse(&env);
        let build = self.build_tasks(Request {
            tree: &tree,
            state: &config,
            packages: packages.as_ref(),
            reuse,
        })?;
        let state = ProjectState::new(build.tasks, state_config, self.root, build.errors);
        if !source.rev.is_empty() {
            let indices: HashMap<PathBuf, usize> = state
                .tasks
                .iter()
                .enumerate()
                .map(|(index, task)| (task.file.clone(), index))
                .collect();
            let files = build
                .seen
                .into_iter()
                .map(|seen| {
                    let outcome = match seen.outcome {
                        Outcome::Task(_) => indices
                            .get(&seen.entry.rel)
                            .copied()
                            .map_or(Outcome::Failed, Outcome::Task),
                        other => other,
                    };
                    (
                        seen.entry.rel,
                        SeenFile {
                            oid: seen.entry.oid,
                            mode: seen.entry.mode,
                            outcome,
                        },
                    )
                })
                .collect();
            self.last_state = Some((state_key, Arc::new(state.clone()), env, files));
        }
        Ok(state)
    }

    /// The state(s) a new load environment may reuse, preferring the previous
    /// state over the working-tree seed.
    fn reuse(&self, env: &str) -> Option<Reuse<'_>> {
        if let Some((_, state, last_env, files)) = &self.last_state
            && last_env == env
        {
            return Some(Reuse {
                tasks: &state.tasks,
                files,
                seeded: false,
            });
        }
        let (work_env, files) = self.working_seed.as_ref()?;
        (work_env == env).then_some(Reuse {
            tasks: self.base,
            files,
            seeded: true,
        })
    }

    /// The state's config, built once per distinct project-config blob.
    fn config_for(&mut self, tree: &StateTree<'_>) -> Result<Arc<StateConfig>, CommitMsgError> {
        let oid = tree.config_oid()?;
        let key = oid.clone().unwrap_or_default();
        if let Some((found_key, found)) = &self.last_config
            && *found_key == key
        {
            return Ok(found.clone());
        }
        // A tree with no project config still gets the embedded, XDG and
        // ancestor-default layers, with an empty project layer on top.
        let text = tree.config_text()?.unwrap_or_default();
        let config = match &self.layers {
            Some(dirs) => tree.parse_config_in(
                &text,
                dirs.home.as_deref(),
                dirs.xdg.as_deref(),
                dirs.pack_cache.as_deref(),
            ),
            None => tree.parse_config(&text),
        };
        let config = Arc::new(config);
        self.last_config = Some((key, config.clone()));
        Ok(config)
    }

    /// Evaluate the files `request.env` does not already hold and assemble the
    /// state's task list.
    fn build_tasks(&self, request: Request<'_, '_>) -> Result<Build, CommitMsgError> {
        let Request {
            tree,
            state,
            packages,
            reuse,
        } = request;
        let config = state.config.as_ref().unwrap_or(self.config);
        let mut tasks = Vec::new();
        let mut errors: Vec<LoadError> = state.error.clone().into_iter().collect();
        let mut seen = Vec::new();
        let mut regular = Vec::new();
        let mut links = Vec::new();
        for entry in tree.task_entries(config)? {
            match reuse
                .as_ref()
                .and_then(|reuse| lookup(reuse, &entry, config))
            {
                Some(Some(task)) => {
                    tasks.push(task);
                    seen.push(Seen {
                        entry,
                        outcome: Outcome::Task(0),
                    });
                }
                Some(None) => seen.push(Seen {
                    entry,
                    outcome: Outcome::Missing,
                }),
                None => {
                    if entry.mode == "120000" {
                        links.push(entry);
                    } else {
                        regular.push(entry);
                    }
                }
            }
        }
        let paths: Vec<&str> = regular.iter().map(|entry| entry.repo.as_str()).collect();
        let blobs = tree.read(&paths)?;
        for (entry, blob) in regular.into_iter().zip(blobs) {
            let found = match blob {
                Some(text) => self.evaluate(config, packages, &entry.rel, &text),
                None => Ok(None),
            };
            let outcome = record(found, &mut tasks, &mut errors);
            seen.push(Seen { entry, outcome });
        }
        for entry in links {
            let found = match tree.task_source(&entry)? {
                TaskSource::Text(text) => self.evaluate(config, packages, &entry.rel, &text),
                TaskSource::Missing => Ok(None),
                TaskSource::Unsupported(message) => Err(LoadError {
                    file: entry.rel.clone(),
                    message,
                    ..LoadError::default()
                }),
            };
            let outcome = record(found, &mut tasks, &mut errors);
            seen.push(Seen { entry, outcome });
        }
        Ok(Build {
            tasks,
            errors,
            seen,
        })
    }

    /// Harvest `text` as the task at project-relative `rel`.
    fn evaluate(
        &self,
        config: &Config,
        packages: Option<&StatePackages>,
        rel: &Path,
        text: &str,
    ) -> Result<Option<Task>, LoadError> {
        #[cfg(test)]
        self.evaluations.set(self.evaluations.get() + 1);
        harvest_source_with(
            self.root,
            config,
            Candidate { rel, source: text },
            packages.cloned(),
        )
    }

    /// Seed the reuse map from the working-tree load when its package and
    /// config environment matches a state's.
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
        let mut files = HashMap::new();
        for (index, task) in self.base.iter().enumerate() {
            if let Some(oid) = working_oid(self.root, &task.file) {
                files.insert(
                    task.file.clone(),
                    SeenFile {
                        oid,
                        mode: "100644".to_owned(),
                        outcome: Outcome::Task(index),
                    },
                );
            }
        }
        self.working_seed = Some((env, files));
    }
}

/// Look up `entry` in `reuse`, validating a seeded task's root choice.
fn lookup(reuse: &Reuse<'_>, entry: &TaskEntry, config: &Config) -> Option<Option<Task>> {
    let seen = reuse.files.get(&entry.rel)?;
    if seen.oid != entry.oid || seen.mode != entry.mode {
        return None;
    }
    match seen.outcome {
        Outcome::Task(index) => {
            let task = reuse.tasks.get(index)?;
            if reuse.seeded && !fits(task, config, &entry.rel) {
                return None;
            }
            Some(Some(task.clone()))
        }
        Outcome::Missing => Some(None),
        Outcome::Failed => None,
    }
}

/// Add one evaluation to the state, recording its outcome.
fn record(
    found: Result<Option<Task>, LoadError>,
    tasks: &mut Vec<Task>,
    errors: &mut Vec<LoadError>,
) -> Outcome {
    match found {
        Ok(Some(task)) => {
            let index = tasks.len();
            tasks.push(task);
            Outcome::Task(index)
        }
        Ok(None) => Outcome::Missing,
        Err(err) => {
            errors.push(err);
            Outcome::Failed
        }
    }
}

/// Whether a seeded task's folder-derived tags and depth still match what the
/// state's own root choice gives this path.
fn fits(task: &Task, config: &Config, rel: &Path) -> bool {
    let (tags, depth) = crate::loader::derivation(config, rel);
    task.derived_tags == tags && task.depth == depth
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
