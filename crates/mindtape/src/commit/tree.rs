//! A commit state's git tree: the task files under its roots, the config it
//! declares, and the `@local` package files that config pins.
//!
//! [`StateTree`] reads a revision, or the index when the revision is empty.
//! [`crate::commit::tasks`] turns what it reads into a project state, so a
//! task file the state leaves unchanged still resolves against that state's
//! own package tree rather than the working tree's.
//!
//! The state's `.mindtape/config.toml` is the top layer over the embedded
//! floor, XDG defaults and ancestor `.mindtape/defaults.toml` files that a
//! working-tree load discovers, so roots and pins those layers declare still
//! apply. The state's project config text, task file bytes (following an
//! in-repository symlink) and pinned package files come from its git tree.
//! A task file's relative imports outside a pin, and files a config layer
//! names such as packs and dictionary paths, still read the working tree.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use typst::syntax::package::{PackageManifest, PackageSpec};

use crate::commit::packages::{PackageBlob, PackageCache, PackageSet};
use crate::commit::tasks::{StatePlan, StateSource};
use crate::commit::tree_io::{
    GitEntry, Listing, git_failed, parse_git_entries, project_rel, project_repo_path, read_blobs,
    read_objects, repo_path, repo_root, resolve_link,
};
use crate::commit::{CommitMsgError, git_optional};
use crate::config::Config;
use crate::loader::{LoadError, StatePackages, package_overrides};

/// The config a state's tree declares, or the reason it could not be used.
#[derive(Clone)]
pub(crate) struct StateConfig {
    /// The state's own config, absent when the tree holds none or it did not
    /// load.
    pub(crate) config: Option<Config>,
    /// Why the tree's config went unused, reported with the state's load
    /// errors.
    pub(crate) error: Option<LoadError>,
}

/// One task file in a state: its repository path, project-relative path and
/// blob id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TaskEntry {
    /// Repository-relative path, as git reports it.
    pub(crate) repo: String,
    /// Project-relative path.
    pub(crate) rel: PathBuf,
    /// The blob object id the state holds.
    pub(crate) oid: String,
    /// The git file mode, so a `120000` symlink is followed rather than read
    /// as Typst source.
    pub(crate) mode: String,
}

/// What a task entry's blob resolves to in a state.
pub(crate) enum TaskSource {
    /// The Typst source to evaluate at the entry's path, and the final blob
    /// identity that source came from.
    Text { text: String, source: String },
    /// The state holds no blob at the entry.
    Missing { source: String },
    /// A symlink the state tree cannot resolve.
    Unsupported { message: String, source: String },
}

impl TaskSource {
    /// The identity of the bytes (or absence) this source names.
    pub(crate) fn source(&self) -> &str {
        match self {
            Self::Text { source, .. }
            | Self::Missing { source }
            | Self::Unsupported { source, .. } => source,
        }
    }
}

/// One state's tree, read through git.
pub(crate) struct StateTree<'a> {
    /// The project root, used to resolve relative paths and run `cat-file`.
    root: &'a Path,
    /// The repository root above `root`.
    repo: PathBuf,
    /// `root`'s path within the repository, `/`-terminated or empty.
    prefix: &'a str,
    /// The revision read, or empty for the index.
    rev: &'a str,
}

impl<'a> StateTree<'a> {
    /// The tree `source` names for the project `plan` describes.
    pub(crate) fn new(root: &'a Path, plan: &'a StatePlan, source: &'a StateSource) -> Self {
        Self {
            root,
            repo: repo_root(root, &plan.root_prefix),
            prefix: &plan.root_prefix,
            rev: &source.rev,
        }
    }

    /// Build [`StateConfig`] from `text`, the state's project config layer.
    pub(crate) fn parse_config(&self, text: &str) -> StateConfig {
        state_config(crate::config::layers::load_config_with_project(
            self.root, text,
        ))
    }

    /// [`Self::parse_config`] with the config home, XDG directory and pack cache
    /// injected, so a test never reads the developer's global layers.
    pub(crate) fn parse_config_in(
        &self,
        text: &str,
        home: Option<&Path>,
        xdg: Option<&Path>,
        pack_cache: Option<&Path>,
    ) -> StateConfig {
        state_config(crate::config::layers::load_config_with_project_in(
            self.root, text, home, xdg, pack_cache,
        ))
    }

    /// The blob id of the state's `.mindtape/config.toml`, or `None` when the
    /// tree holds none.
    pub(crate) fn config_oid(&self) -> Result<Option<String>, CommitMsgError> {
        match project_repo_path(self.prefix, Path::new(CONFIG_PATH)) {
            Some(path) => self.object_oid(&path),
            None => Ok(None),
        }
    }

    /// The text of the state's `.mindtape/config.toml`, if it holds one.
    pub(crate) fn config_text(&self) -> Result<Option<String>, CommitMsgError> {
        self.project_text(Path::new(CONFIG_PATH))
    }

    /// Every `.typ` task file under the roots `config` declares, with the blob
    /// ids that identify unchanged files between states. A file under two
    /// overlapping roots is listed once.
    pub(crate) fn task_entries(&self, config: &Config) -> Result<Vec<TaskEntry>, CommitMsgError> {
        let mut entries = BTreeMap::new();
        for spec in &config.tasks.roots {
            let root = spec.dir();
            let Some(prefix) = repo_path(self.prefix, Path::new(&root)) else {
                continue;
            };
            for entry in self.list_entries(&prefix)? {
                if !entry.path.ends_with(".typ") {
                    continue;
                }
                let Some(rel) = project_rel(self.prefix, &entry.path) else {
                    continue;
                };
                // The first root that contains a file wins, matching
                // `loader::item_at` and `collect_items`.
                entries.entry(rel).or_insert(entry);
            }
        }
        Ok(entries
            .into_iter()
            .map(|(rel, entry)| TaskEntry {
                repo: entry.path,
                rel,
                oid: entry.oid,
                mode: entry.mode,
            })
            .collect())
    }

    /// [`Self::task_entries`] without the blob ids, for a caller that only
    /// wants the paths.
    #[cfg(test)]
    pub(crate) fn task_files(
        &self,
        config: &Config,
    ) -> Result<Vec<(String, PathBuf)>, CommitMsgError> {
        Ok(self
            .task_entries(config)?
            .into_iter()
            .map(|entry| (entry.repo, entry.rel))
            .collect())
    }

    /// The Typst source `entry` resolves to, following a symlink's target
    /// inside the state tree.
    pub(crate) fn task_source(&self, entry: &TaskEntry) -> Result<TaskSource, CommitMsgError> {
        if entry.mode != "120000" {
            return Ok(match self.read(&[entry.repo.as_str()])?.pop().flatten() {
                Some(text) => TaskSource::Text {
                    text,
                    source: entry.oid.clone(),
                },
                None => TaskSource::Missing {
                    source: entry.oid.clone(),
                },
            });
        }
        let mut seen = HashSet::new();
        let mut path = entry.repo.clone();
        loop {
            if !seen.insert(path.clone()) {
                return Ok(TaskSource::Unsupported {
                    message: format!("symlinked task file cycles at '{path}'"),
                    source: format!("cycle:{path}"),
                });
            }
            let Some(target) = self.read(&[path.as_str()])?.pop().flatten() else {
                return Ok(TaskSource::Missing {
                    source: format!("missing-link:{path}"),
                });
            };
            let target = target.trim_end_matches(['\n', '\r']);
            let Some(resolved) = resolve_link(&path, target) else {
                return Ok(TaskSource::Unsupported {
                    message: format!("symlinked task file '{path}' points outside the repository"),
                    source: format!("outside:{path}:{target}"),
                });
            };
            let Some(found) = self.entry_at(&resolved)? else {
                return Ok(TaskSource::Unsupported {
                    message: format!("symlinked task file '{path}' target '{resolved}' is missing"),
                    source: format!("missing:{resolved}"),
                });
            };
            if found.mode == "120000" {
                path = resolved;
                continue;
            }
            if found.kind != "blob" {
                return Ok(TaskSource::Unsupported {
                    message: format!(
                        "symlinked task file '{path}' target '{resolved}' is not a blob"
                    ),
                    source: format!("not-blob:{resolved}:{}", found.mode),
                });
            }
            let source = found.oid;
            return Ok(match self.read(&[resolved.as_str()])?.pop().flatten() {
                Some(text) => TaskSource::Text { text, source },
                None => TaskSource::Missing {
                    source: format!("missing:{resolved}"),
                },
            });
        }
    }

    /// Each `paths` object as text, in order, through one `cat-file --batch`;
    /// `None` where this state holds no such path.
    pub(crate) fn read(&self, paths: &[&str]) -> Result<Vec<Option<String>>, CommitMsgError> {
        let specs: Vec<String> = paths
            .iter()
            .map(|path| format!("{}:{path}", self.rev))
            .collect();
        read_blobs(self.root, &specs)
    }

    /// The text of project-relative `rel` in this state, if it holds one.
    fn project_text(&self, rel: &Path) -> Result<Option<String>, CommitMsgError> {
        let Some(path) = project_repo_path(self.prefix, rel) else {
            return Ok(None);
        };
        Ok(self.read(&[path.as_str()])?.into_iter().next().flatten())
    }

    /// The `@local` package files this tree holds at the paths `config` pins,
    /// keyed by the version each manifest declares. A package whose pin tree
    /// `cache` already holds is not read again.
    pub(crate) fn packages_cached(
        &self,
        config: &Config,
        cache: &mut PackageCache,
    ) -> Result<Option<StatePackages>, CommitMsgError> {
        let mut set = PackageSet::default();
        for (name, dir) in package_overrides(self.root, config) {
            let rel = dir.strip_prefix(self.root).unwrap_or(&dir);
            let Some(prefix) = repo_path(self.prefix, rel) else {
                continue;
            };
            let pairs = self.package_cached(&name, &prefix, cache)?;
            if let Some((spec, blobs)) = pairs {
                set.add(spec, blobs);
            }
        }
        Ok(set.finish())
    }

    /// [`Self::packages_cached`] with a fresh cache.
    #[cfg(test)]
    pub(crate) fn packages(
        &self,
        config: &Config,
    ) -> Result<Option<StatePackages>, CommitMsgError> {
        self.packages_cached(config, &mut PackageCache::default())
    }

    /// The package at `name` and repository path `prefix`, from `cache` when
    /// the pin tree is unchanged.
    fn package_cached(
        &self,
        name: &str,
        prefix: &str,
        cache: &mut PackageCache,
    ) -> Result<Option<(PackageSpec, Vec<PackageBlob>)>, CommitMsgError> {
        if self.rev.is_empty() {
            return self.package(name, prefix);
        }
        let Some(tree) = self.tree_oid(prefix)? else {
            return Ok(None);
        };
        if let Some(found) = cache.get(name, &tree) {
            return Ok(found);
        }
        let found = self.package(name, prefix)?;
        cache.insert(name, &tree, found.clone());
        Ok(found)
    }

    /// The tree id `prefix` names in this revision, when it is a tree.
    fn tree_oid(&self, prefix: &str) -> Result<Option<String>, CommitMsgError> {
        if self.rev.is_empty() {
            return Ok(None);
        }
        self.object_oid(prefix)
    }

    /// The object id repository-relative `path` names in this revision.
    fn object_oid(&self, path: &str) -> Result<Option<String>, CommitMsgError> {
        let spec = format!("{}:{path}", self.rev);
        Ok(
            git_optional(self.root, &["rev-parse", "--verify", "--quiet", &spec])?
                .map(|oid| oid.trim().to_owned())
                .filter(|oid| !oid.is_empty()),
        )
    }

    /// The package at repository path `prefix`: its spec and every file's
    /// package-relative path with its bytes, or `None` when the tree holds no
    /// readable manifest there.
    fn package(
        &self,
        name: &str,
        prefix: &str,
    ) -> Result<Option<(PackageSpec, Vec<PackageBlob>)>, CommitMsgError> {
        let paths: Vec<String> = self
            .list_entries(prefix)?
            .into_iter()
            .map(|entry| entry.path)
            .collect();
        let manifest_path = if prefix.is_empty() {
            "typst.toml".to_owned()
        } else {
            format!("{prefix}/typst.toml")
        };
        let Some(manifest_at) = paths.iter().position(|path| *path == manifest_path) else {
            return Ok(None);
        };
        let read_specs: Vec<String> = paths
            .iter()
            .map(|path| format!("{}:{path}", self.rev))
            .collect();
        let objects = read_objects(self.root, &read_specs)?;
        let Some(Some(manifest)) = objects.get(manifest_at) else {
            return Ok(None);
        };
        let Ok(manifest) = std::str::from_utf8(manifest) else {
            return Ok(None);
        };
        let Ok(manifest) = toml::from_str::<PackageManifest>(manifest) else {
            return Ok(None);
        };
        let version = manifest.package.version;
        let spec = PackageSpec {
            namespace: "local".into(),
            name: name.into(),
            version,
        };
        let mut blobs = Vec::new();
        for (path, bytes) in paths.iter().zip(objects) {
            let Some(bytes) = bytes else { continue };
            let vpath = path
                .strip_prefix(prefix)
                .map_or(path.as_str(), |rest| rest.trim_start_matches('/'));
            blobs.push((vpath.to_owned(), bytes));
        }
        Ok(Some((spec, blobs)))
    }

    /// The git entry repository-relative `path` holds in this state.
    fn entry_at(&self, path: &str) -> Result<Option<GitEntry>, CommitMsgError> {
        let args = if self.rev.is_empty() {
            vec!["ls-files", "-s", "-z", "--full-name", "--"]
        } else {
            vec!["ls-tree", "-z", self.rev, "--"]
        };
        Ok(self.entries(&args, path)?.into_iter().next())
    }

    /// Every repository-relative blob this state holds under `prefix`, with
    /// its object id, through one `git ls-tree` or `git ls-files`.
    fn list_entries(&self, prefix: &str) -> Result<Vec<GitEntry>, CommitMsgError> {
        let pathspec = if prefix.is_empty() { "." } else { prefix };
        let args = if self.rev.is_empty() {
            vec!["ls-files", "-s", "-z", "--cached", "--full-name", "--"]
        } else {
            vec!["ls-tree", "-r", "-z", self.rev, "--"]
        };
        Ok(self
            .entries(&args, pathspec)?
            .into_iter()
            .filter(|entry| entry.kind == "blob")
            .collect())
    }

    /// Run one git listing with `args` and `pathspec`, and parse its entries.
    fn entries(&self, args: &[&str], pathspec: &str) -> Result<Vec<GitEntry>, CommitMsgError> {
        let command = args.first().copied().unwrap_or_default();
        let output = Command::new("git")
            .arg("--no-optional-locks")
            .arg("-C")
            .arg(&self.repo)
            .args(args)
            .arg(pathspec)
            .output()?;
        if !output.status.success() {
            return Err(git_failed(command, &output));
        }
        Ok(parse_git_entries(
            &output.stdout,
            if self.rev.is_empty() {
                Listing::Index
            } else {
                Listing::Tree
            },
        ))
    }
}

/// The config file a state may override.
const CONFIG_PATH: &str = ".mindtape/config.toml";

/// Turn a layered config result into a [`StateConfig`], warning when a state's
/// own project layer could not load.
fn state_config(loaded: Result<Config, crate::config::ConfigError>) -> StateConfig {
    match loaded {
        Ok(config) => StateConfig {
            config: Some(config),
            error: None,
        },
        Err(err) => StateConfig {
            config: None,
            error: Some(LoadError {
                file: PathBuf::from(".mindtape/config.toml"),
                message: format!("{err}; the working tree's config is used for this state"),
                ..LoadError::default()
            }),
        },
    }
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tests;
