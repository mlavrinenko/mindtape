//! Caches for loading commit states across a range or a `commit-msg` hook.
//!
//! A linear range only needs the previous state. The loader reuses its
//! unchanged tasks by source identity, and evaluates changed files again. A
//! regular file's identity is its blob id. The identity of a symlink is the
//! blob id of its final target, so a target change cannot reuse a stale task.
//! The working-tree load seeds the same reuse map for a hook, so a clean index
//! does not re-evaluate the whole backlog.
//!
//! A task's markers also depend on the project modules its source imports
//! (`truth/goals.typ` is one), which an evaluation reads from the working tree
//! even for a past state. The reuse key holds every project file the harvest
//! read, hashed the way the content cache hashes its own dependencies. An
//! edited module then re-evaluates the tasks that read it. The seed takes the
//! same list from the content cache entry the working-tree load wrote.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::commit::CommitMsgError;
use crate::commit::packages::{PackageCache, working_packages};
use crate::commit::state_cache::{
    Build, Outcome, Parts, Reuse, Reused, Seen, SeenFile, load_env, lookup, record,
    record_evaluated, working_oid,
};
use crate::commit::tasks::{CommitStates, ProjectState, StatePlan, StateSource};
use crate::commit::tree::{StateConfig, StateTree, TaskEntry, TaskSource};
use crate::config::Config;
use crate::loader::cache::TaskCache;
use crate::loader::{LoadError, StatePackages};
use crate::model::Task;
use crate::validate::candidate::{Candidate, HarvestedSource, harvest_source_with};

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

/// What a new state needs to evaluate its files.
struct Request<'a, 't> {
    tree: &'a StateTree<'t>,
    state: &'a StateConfig,
    packages: Option<&'a StatePackages>,
    reuse: Option<Reuse<'a>>,
}

/// A state's task entries, split by what an earlier state already answers for.
struct Classified {
    /// The entries an earlier state answered for, with its outcome.
    seen: Vec<Seen>,
    /// The regular files this state must evaluate.
    regular: Vec<TaskEntry>,
    /// The symlinked files this state must resolve and evaluate.
    links: Vec<(TaskEntry, TaskSource)>,
    /// The project files each reused file read.
    deps: HashMap<PathBuf, Vec<(PathBuf, String)>>,
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
    /// Holds the previous state only, because a linear range needs it as the
    /// next `head`.
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
    /// and packages. The returned state carries that config. A caller then
    /// builds the commit's record from the same config the state loaded with.
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
        // `HEAD` is a moving name, so only a resolved revision may hit the
        // state cache; the hook's literal name is loaded once and dropped.
        let cacheable = !source.rev.is_empty() && source.rev != "HEAD";
        if cacheable
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
        if cacheable {
            let indices: HashMap<PathBuf, usize> = state
                .tasks
                .iter()
                .enumerate()
                .map(|(index, task)| (task.file.clone(), index))
                .collect();
            let mut deps = build.deps;
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
                        seen.entry.rel.clone(),
                        SeenFile {
                            source: seen.source,
                            mode: seen.entry.mode,
                            deps: deps.remove(&seen.entry.rel).unwrap_or_default(),
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
                root: self.root,
                seeded: false,
            });
        }
        let (work_env, files) = self.working_seed.as_ref()?;
        (work_env == env).then_some(Reuse {
            tasks: self.base,
            files,
            root: self.root,
            seeded: true,
        })
    }

    /// The state's config, built once per distinct project-config blob and kept
    /// for the state that carries it.
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
        let Classified {
            mut seen,
            regular,
            links,
            mut deps,
        } = Self::classify(tree, config, reuse.as_ref(), &mut tasks)?;
        let paths: Vec<&str> = regular.iter().map(|entry| entry.repo.as_str()).collect();
        let blobs = tree.read(&paths)?;
        for (entry, blob) in regular.into_iter().zip(blobs) {
            let found = blob.as_deref().map_or(Ok(None), |text| {
                self.evaluate(config, packages, &entry.rel, text)
            });
            let (outcome, read) = record_evaluated(self.root, found, &mut tasks, &mut errors);
            deps.insert(entry.rel.clone(), read);
            seen.push(Seen {
                source: entry.oid.clone(),
                entry,
                outcome,
            });
        }
        let mut parts = Parts {
            tasks: &mut tasks,
            errors: &mut errors,
        };
        seen.extend(self.evaluate_links(links, config, packages, &mut parts));
        Ok(Build {
            tasks,
            errors,
            seen,
            deps,
        })
    }

    /// Split the state's task entries into the files a previous state already
    /// answers for and the ones this state must read. Every reused task joins
    /// `tasks`.
    fn classify(
        tree: &StateTree<'_>,
        config: &Config,
        reuse: Option<&Reuse<'_>>,
        tasks: &mut Vec<Task>,
    ) -> Result<Classified, CommitMsgError> {
        let (mut seen, mut regular, mut links) = (Vec::new(), Vec::new(), Vec::new());
        let mut deps = HashMap::new();
        for entry in tree.task_entries(config)? {
            let resolved = if entry.mode == "120000" {
                Some(tree.task_source(&entry)?)
            } else {
                None
            };
            let source = resolved
                .as_ref()
                .map_or_else(|| entry.oid.clone(), |source| source.source().to_owned());
            match reuse.and_then(|reuse| lookup(reuse, &entry, &source, config)) {
                Some(Reused { task, deps: read }) => {
                    let outcome = task.map_or(Outcome::Missing, |task| {
                        tasks.push(task);
                        Outcome::Task(0)
                    });
                    deps.insert(entry.rel.clone(), read);
                    seen.push(Seen {
                        entry,
                        source,
                        outcome,
                    });
                }
                None => {
                    if let Some(source) = resolved {
                        links.push((entry, source));
                    } else {
                        regular.push(entry);
                    }
                }
            }
        }
        Ok(Classified {
            seen,
            regular,
            links,
            deps,
        })
    }

    /// Evaluate the symlinked entries already resolved against the tree.
    fn evaluate_links(
        &self,
        links: Vec<(TaskEntry, TaskSource)>,
        config: &Config,
        packages: Option<&StatePackages>,
        parts: &mut Parts<'_>,
    ) -> Vec<Seen> {
        links
            .into_iter()
            .map(|(entry, source)| {
                let identity = source.source().to_owned();
                let found = match source {
                    TaskSource::Text { text, .. } => {
                        self.evaluate(config, packages, &entry.rel, &text)
                    }
                    TaskSource::Missing { .. } => Ok(None),
                    TaskSource::Unsupported { message, .. } => Err(LoadError {
                        file: entry.rel.clone(),
                        message,
                        ..LoadError::default()
                    }),
                };
                let outcome = record(found, parts.tasks, parts.errors);
                Seen {
                    entry,
                    source: identity,
                    outcome,
                }
            })
            .collect()
    }

    /// Harvest `text` as the task at project-relative `rel`, keeping the
    /// project files the evaluation read so a reuse can check them. `Some`
    /// because the caller holds the bytes to evaluate.
    fn evaluate(
        &self,
        config: &Config,
        packages: Option<&StatePackages>,
        rel: &Path,
        text: &str,
    ) -> Result<Option<HarvestedSource>, LoadError> {
        #[cfg(test)]
        self.evaluations.set(self.evaluations.get() + 1);
        Ok(Some(harvest_source_with(
            self.root,
            config,
            Candidate { rel, source: text },
            packages.cloned(),
        )?))
    }

    /// Seed the reuse map from the working-tree load when its package and
    /// config environment matches a state's.
    ///
    /// A seeded entry carries the project files the working-tree load read,
    /// from the content cache's own dependency list. A state then drops the
    /// seed for a task whose imported module changed on disk.
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
        let overrides = crate::loader::package_overrides(self.root, self.config);
        let cache = TaskCache::open(self.root, self.config, &overrides);
        let mut files = HashMap::new();
        for (index, task) in self.base.iter().enumerate() {
            if let Some(oid) = working_oid(self.root, &task.file) {
                let mode = if std::fs::symlink_metadata(self.root.join(&task.file))
                    .is_ok_and(|meta| meta.file_type().is_symlink())
                {
                    "120000"
                } else {
                    "100644"
                };
                // A task whose cache entry is missing records no imports, and
                // a state then evaluates it again instead of reusing it.
                if let Some(deps) = self.seeded_deps(&cache, task) {
                    files.insert(
                        task.file.clone(),
                        SeenFile {
                            source: oid,
                            mode: mode.to_owned(),
                            deps,
                            outcome: Outcome::Task(index),
                        },
                    );
                }
            }
        }
        self.working_seed = Some((env, files));
    }

    /// The project files the working-tree load read for `task`, from the
    /// content cache entry it wrote. `None` when that entry is missing: a
    /// disabled cache, an evicted entry, or an older format. The state then
    /// evaluates the task again.
    fn seeded_deps(&self, cache: &TaskCache, task: &Task) -> Option<Vec<(PathBuf, String)>> {
        let bytes = std::fs::read(self.root.join(&task.file)).ok()?;
        cache.dependencies(&TaskCache::content_hash(&bytes))
    }
}
