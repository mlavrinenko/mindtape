//! A commit state's git tree: the task files under its roots, the config it
//! declares, and the `@local` package files that config pins.
//!
//! [`StateTree`] reads a revision, or the index when the revision is empty.
//! [`crate::commit::tasks`] turns what it reads into a project state, so a
//! task file the state leaves unchanged still resolves against that state's
//! own package tree rather than the working tree's.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

use typst::syntax::package::{PackageManifest, PackageSpec};
use typst_world::{BytesProvider, local_package_file_id};

use crate::commit::CommitMsgError;
use crate::commit::tasks::{StatePlan, StateSource};
use crate::config::Config;
use crate::loader::{LoadError, StatePackages, package_overrides};

/// The config a state's tree declares, or the reason it could not be used.
pub(crate) struct StateConfig {
    /// The state's own config, absent when the tree holds none or it did not
    /// parse.
    pub(crate) config: Option<Config>,
    /// Why the tree's config went unused, reported with the state's load
    /// errors.
    pub(crate) error: Option<LoadError>,
}

/// One package file: its path within the package and its bytes.
type PackageBlob = (String, Vec<u8>);

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

    /// The state's own `.mindtape/config.toml`, parsed over the embedded
    /// floor. A missing file leaves both fields empty; a malformed one leaves
    /// the config empty and carries the reason, so the caller can fall back to
    /// the working tree's config and still say why.
    pub(crate) fn config(&self) -> Result<StateConfig, CommitMsgError> {
        let Some(text) = self.project_text(Path::new(".mindtape/config.toml"))? else {
            return Ok(StateConfig {
                config: None,
                error: None,
            });
        };
        Ok(match crate::config::parse_config(&text) {
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
        })
    }

    /// Every `.typ` task file under the roots `config` declares, as its
    /// repository path and its project-relative path. A file under two
    /// overlapping roots is listed once.
    pub(crate) fn task_files(
        &self,
        config: &Config,
    ) -> Result<Vec<(String, PathBuf)>, CommitMsgError> {
        let mut files = BTreeMap::new();
        for spec in &config.tasks.roots {
            let Some(prefix) = repo_path(self.prefix, Path::new(spec.path())) else {
                continue;
            };
            for path in self.list(&prefix)? {
                if !path.ends_with(".typ") {
                    continue;
                }
                let Some(rel) = project_rel(self.prefix, &path) else {
                    continue;
                };
                files.insert(rel, path);
            }
        }
        Ok(files.into_iter().map(|(rel, path)| (path, rel)).collect())
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
    /// keyed by the version each manifest declares.
    pub(crate) fn packages(
        &self,
        config: &Config,
    ) -> Result<Option<StatePackages>, CommitMsgError> {
        let mut specs = Vec::new();
        let mut files = BytesProvider::new();
        for (name, dir) in package_overrides(self.root, config) {
            let rel = dir.strip_prefix(self.root).unwrap_or(&dir);
            let Some(prefix) = repo_path(self.prefix, rel) else {
                continue;
            };
            let Some((spec, blobs)) = self.package(&name, &prefix)? else {
                continue;
            };
            let version = spec.version.to_string();
            for (vpath, bytes) in blobs {
                let Ok(id) = local_package_file_id(&name, &version, &vpath) else {
                    continue;
                };
                files = files.with_file(id, bytes);
            }
            specs.push(spec);
        }
        if specs.is_empty() {
            return Ok(None);
        }
        Ok(Some(StatePackages {
            specs,
            files: Arc::new(files),
        }))
    }

    /// The package at repository path `prefix`: its spec and every file's
    /// package-relative path with its bytes, or `None` when the tree holds no
    /// readable manifest there.
    fn package(
        &self,
        name: &str,
        prefix: &str,
    ) -> Result<Option<(PackageSpec, Vec<PackageBlob>)>, CommitMsgError> {
        let paths = self.list(prefix)?;
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

    /// Every repository-relative path this state holds under `prefix`, through
    /// one `git ls-tree`, or `git ls-files` for the index.
    fn list(&self, prefix: &str) -> Result<Vec<String>, CommitMsgError> {
        let pathspec = if prefix.is_empty() { "." } else { prefix };
        let (command, args) = if self.rev.is_empty() {
            (
                "ls-files",
                vec!["ls-files", "-z", "--cached", "--full-name", "--"],
            )
        } else {
            (
                "ls-tree",
                vec!["ls-tree", "-r", "--name-only", "-z", self.rev, "--"],
            )
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
        Ok(output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| String::from_utf8_lossy(path).into_owned())
            .collect())
    }
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
