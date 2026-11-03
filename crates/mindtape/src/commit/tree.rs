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
//! apply. Packs, dictionary paths and other files a layer names are read from
//! the working tree: only the project config text and the files under the
//! task roots and package pins come from the state's git tree.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use typst::syntax::package::{PackageManifest, PackageSpec};

use crate::commit::packages::{PackageBlob, PackageCache, PackageSet};
use crate::commit::tasks::{StatePlan, StateSource};
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

    /// The state's own `.mindtape/config.toml`, layered over the embedded,
    /// XDG and ancestor-default layers a working-tree load uses. A missing
    /// file leaves both fields empty; a config that cannot load leaves the
    /// config empty and carries the reason, so the caller can fall back to the
    /// working tree's config and still say why.
    #[cfg(test)]
    pub(crate) fn config(&self) -> Result<StateConfig, CommitMsgError> {
        let Some(text) = self.config_text()? else {
            return Ok(StateConfig {
                config: None,
                error: None,
            });
        };
        Ok(self.parse_config(&text))
    }

    /// Build [`StateConfig`] from `text`, the state's project config layer.
    pub(crate) fn parse_config(&self, text: &str) -> StateConfig {
        match crate::config::layers::load_config_with_project(self.root, text) {
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
            let Some(prefix) = repo_path(self.prefix, Path::new(spec.path())) else {
                continue;
            };
            for (path, oid) in self.list_entries(&prefix)? {
                if !path.ends_with(".typ") {
                    continue;
                }
                let Some(rel) = project_rel(self.prefix, &path) else {
                    continue;
                };
                entries.insert(rel, (path, oid));
            }
        }
        Ok(entries
            .into_iter()
            .map(|(rel, (repo, oid))| TaskEntry { repo, rel, oid })
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
            .map(|(path, _)| path)
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

    /// Every repository-relative blob this state holds under `prefix`, with
    /// its object id, through one `git ls-tree` or `git ls-files`.
    fn list_entries(&self, prefix: &str) -> Result<Vec<(String, String)>, CommitMsgError> {
        let pathspec = if prefix.is_empty() { "." } else { prefix };
        let (command, args) = if self.rev.is_empty() {
            (
                "ls-files",
                vec!["ls-files", "-s", "-z", "--cached", "--full-name", "--"],
            )
        } else {
            ("ls-tree", vec!["ls-tree", "-r", "-z", self.rev, "--"])
        };
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

/// Which git listing `parse_git_entries` is reading.
#[derive(Clone, Copy)]
enum Listing {
    /// `git ls-tree -r -z`: `<mode> <type> <oid>\t<path>`.
    Tree,
    /// `git ls-files -s -z`: `<mode> <oid> <stage>\t<path>`.
    Index,
}

/// Split a NUL-delimited git listing into `(path, oid)` pairs, keeping tree
/// blobs and stage-zero index entries.
fn parse_git_entries(out: &[u8], listing: Listing) -> Vec<(String, String)> {
    out.split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .filter_map(|entry| {
            let text = String::from_utf8_lossy(entry);
            let (meta, path) = text.split_once('\t')?;
            let mut fields = meta.split_whitespace();
            let _mode = fields.next()?;
            let second = fields.next()?;
            let third = fields.next()?;
            let oid = match listing {
                Listing::Tree if second == "blob" => third,
                Listing::Index if third == "0" => second,
                _ => return None,
            };
            Some((path.to_owned(), oid.to_owned()))
        })
        .collect()
}

/// The project-relative path `path` names under `prefix`, or `None` when it
/// lies outside the project.
fn project_rel(prefix: &str, path: &str) -> Option<PathBuf> {
    let rel = path.strip_prefix(prefix)?.trim_start_matches('/');
    (!rel.is_empty()).then(|| PathBuf::from(rel))
}

/// The repository-relative path of project-relative `rel` under `prefix`.
fn project_repo_path(prefix: &str, rel: &Path) -> Option<String> {
    let rel = crate::slash_path(rel);
    (!rel.is_empty()).then(|| format!("{prefix}{rel}"))
}

/// The repository-relative path of `pkg` under `root_prefix`, or `None` when
/// it is absolute or escapes the repository. `.` and `..` are resolved
/// lexically, since a package may sit outside the project but inside the repo.
fn repo_path(root_prefix: &str, pkg: &Path) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for component in Path::new(root_prefix).components().chain(pkg.components()) {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::Normal(name) => parts.push(name.to_str()?),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts.join("/"))
}

/// The repository root above `root`, whose path within the repo is
/// `root_prefix`.
fn repo_root(root: &Path, root_prefix: &str) -> PathBuf {
    let mut repo = root.to_path_buf();
    for _ in Path::new(root_prefix).components() {
        repo.pop();
    }
    repo
}

/// The error for a git command that failed, carrying its stderr.
fn git_failed(command: &str, output: &std::process::Output) -> CommitMsgError {
    CommitMsgError::Git {
        command: command.to_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

/// Each object `specs` names, in order, through one `git cat-file --batch`:
/// its text, or `None` where the revision holds no such path.
fn read_blobs(root: &Path, specs: &[String]) -> Result<Vec<Option<String>>, CommitMsgError> {
    Ok(read_objects(root, specs)?
        .into_iter()
        .map(|blob| blob.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
        .collect())
}

/// Each object `specs` names, in order, through one `git cat-file --batch`:
/// its bytes, or `None` where the revision holds no such path. A package may
/// carry images or fonts, so this keeps the bytes a task file would read as
/// text.
fn read_objects(root: &Path, specs: &[String]) -> Result<Vec<Option<Vec<u8>>>, CommitMsgError> {
    if specs.is_empty() {
        return Ok(Vec::new());
    }
    let mut child = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let input: String = specs.iter().map(|spec| format!("{spec}\n")).collect();
    let mut stdin = child.stdin.take();
    // Written from its own thread: git answers as it reads, so a large batch
    // written inline would fill the output pipe before the input drains.
    let writer = std::thread::spawn(move || {
        stdin
            .as_mut()
            .map_or(Ok(()), |pipe| pipe.write_all(input.as_bytes()))
    });
    let mut out = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_end(&mut out)?;
    }
    let output = child.wait_with_output()?;
    writer.join().unwrap_or(Ok(()))?;
    if !output.status.success() {
        return Err(git_failed("cat-file", &output));
    }
    Ok(parse_batch_bytes(&out, specs.len()))
}

/// Split `git cat-file --batch` output into `count` objects: `<oid> <type>
/// <size>\n<content>\n` each, or `<spec> missing\n` for an absent one.
fn parse_batch_bytes(mut out: &[u8], count: usize) -> Vec<Option<Vec<u8>>> {
    let mut blobs = Vec::with_capacity(count);
    while blobs.len() < count {
        let Some(end) = out.iter().position(|byte| *byte == b'\n') else {
            break;
        };
        let header = String::from_utf8_lossy(out.get(..end).unwrap_or_default());
        out = out.get(end + 1..).unwrap_or_default();
        let size = header
            .rsplit(' ')
            .next()
            .and_then(|size| size.parse::<usize>().ok())
            .filter(|_| header.split(' ').count() == 3);
        let Some(size) = size else {
            blobs.push(None);
            continue;
        };
        let body = out.get(..size).unwrap_or_default();
        blobs.push(Some(body.to_vec()));
        out = out.get(size + 1..).unwrap_or_default();
    }
    blobs.resize(count, None);
    blobs
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tests;
