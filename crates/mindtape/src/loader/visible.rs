//! The working-tree view the loader, the write guard and `mt fix` share.
//!
//! Discovery asks Git, once per configured root and repository, for the files
//! it lists: tracked regular files plus untracked files its ignore rules do not
//! hide. That answer keeps a tracked task under an ignore pattern visible while
//! an untracked ignored file stays out, and it keeps `mt ls` in step with the
//! commit states. Git's paths are bytes, so they are decoded exactly as the
//! query engine decodes them.
//!
//! A root outside a work tree, or a machine without `git`, falls back to a
//! filesystem walk that honors the task-source policy's skips and never
//! follows a directory symlink. Any other Git failure is a [`LoadError`]
//! naming the root. A root Git ignores is still listed, but only its tracked
//! files can be seen, and one warning per root says so.
//!
//! # Files
//!
//! - `visible_git.rs`: the Git queries beneath discovery.
//! - `visible_project.rs`: the project-wide `.typ` listing `mt fix --all` reads.
//! - `visible_tests.rs`: the destination probe, both visible listings, Git
//!   failure handling, and the fallback walk.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::config::{Config, RootSpec};
use crate::query::GitFailure;
use crate::task_source::{TaskPath, TaskPathKind, TaskSource};

use super::visible_git::{RepoListing, existing_dir, ignored_at, repo_of, tracked_at};
pub use super::visible_project::{project_is_worktree, visible_typ_files};
use super::{LoadError, LoadWarning};

/// Why a prospective task path would not be loaded from the working tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotVisible {
    /// The path lies outside every configured `[[tasks.roots]]` entry.
    OutsideRoots,
    /// The path is not a `.typ` file.
    NotTyp,
    /// The path lies under a reserved `.mindtape`/`.git` component or a
    /// configured `[templates] dirs` entry.
    Reserved,
    /// The path is a symbolic link; a task source is never a symlink.
    Symlink,
    /// A directory, or another non-file entry, already sits at the path.
    NotAFile,
    /// Git ignores the path and does not track it.
    Ignored,
    /// Git could not answer for the path's root. The message names the root
    /// and Git's reason.
    Git(String),
}

impl std::fmt::Display for NotVisible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutsideRoots => f.write_str("outside every configured task root"),
            Self::NotTyp => f.write_str("not a `.typ` file"),
            Self::Reserved => f.write_str("under reserved metadata or a template directory"),
            Self::Symlink => f.write_str("a symbolic link, which is never a task source"),
            Self::NotAFile => f.write_str("a directory, not a file"),
            Self::Ignored => f.write_str("ignored by git and not tracked"),
            Self::Git(reason) => f.write_str(reason),
        }
    }
}

impl std::error::Error for NotVisible {}

/// Every configured root's visible candidates, built once per load.
///
/// The candidates are absolute paths to regular `.typ` files that are either
/// task sources or reserved files whose `task.with` call the loader warns
/// about. Each root is listed exactly once, never once per file.
pub(crate) struct VisibleFiles {
    source: TaskSource,
    roots: Vec<RootView>,
}

/// What discovery found for one configured root.
pub(crate) struct RootView {
    /// The normalized absolute root directory.
    pub(super) dir: PathBuf,
    /// The root relative to the project, for warnings and errors.
    pub(super) root_rel: PathBuf,
    /// Absolute candidate paths, sorted.
    pub(super) candidates: BTreeSet<PathBuf>,
    mode: RootMode,
    /// Whether Git ignores the root itself.
    ignored: bool,
    /// A Git failure other than the two fallback cases.
    error: Option<LoadError>,
}

/// How one root's files were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootMode {
    /// Git listed the root.
    Git,
    /// Git is unavailable or the root is outside a work tree.
    Walk,
    /// Git failed; the root has no files and contributes an error.
    Failed,
}

impl VisibleFiles {
    /// Read every configured root once. Git failures are recorded on their
    /// root; a root that falls back is walked with the task-source policy.
    pub(crate) fn new(root: &Path, config: &Config) -> Self {
        let source = TaskSource::new(root, config);
        let mut repos = BTreeMap::new();
        let roots = (source.roots().iter())
            .map(|dir| RootView::discover(&source, dir, &mut repos))
            .collect();
        Self { source, roots }
    }

    /// The policy this view was built for.
    pub(crate) fn source(&self) -> &TaskSource {
        &self.source
    }

    /// One view per configured root, in config order.
    pub(crate) fn roots(&self) -> &[RootView] {
        &self.roots
    }

    /// Every root's Git failure, in config order.
    pub(crate) fn errors(&self) -> impl Iterator<Item = &LoadError> {
        self.roots.iter().filter_map(|view| view.error.as_ref())
    }

    /// One warning for each distinct ignored root, in config order.
    pub(crate) fn warnings(&self) -> Vec<LoadWarning> {
        let mut seen = BTreeSet::new();
        (self.roots.iter())
            .filter(|view| view.ignored && seen.insert(view.root_rel.clone()))
            .map(|view| ignored_warning(&view.root_rel))
            .collect()
    }

    /// Whether a file at `path` would be loaded from the working tree.
    ///
    /// The policy half (root, `.typ`, reserved and template rules) is the
    /// caller's; this answers the working-tree half for an existing source.
    pub(crate) fn admits(&self, path: &TaskPath) -> Result<(), NotVisible> {
        let Some(view) = (self.roots.iter()).find(|view| path.abs().starts_with(&view.dir)) else {
            return Err(NotVisible::OutsideRoots);
        };
        view.admits(path)
    }
}

impl RootView {
    /// Discover one root, using `repos` as the per-load repository cache.
    fn discover(
        source: &TaskSource,
        dir: &Path,
        repos: &mut BTreeMap<PathBuf, Result<RepoListing, GitFailure>>,
    ) -> Self {
        let root_rel = source.task_path(dir).rel().to_path_buf();
        match repo_of(dir) {
            Ok(Some(repo)) => {
                let listing = repos
                    .entry(repo.top.clone())
                    .or_insert_with(|| RepoListing::read(&repo.top));
                match listing {
                    Ok(listing) => {
                        let candidates = git_candidates(source, dir, &repo.prefix, listing);
                        Self::git(dir, root_rel, candidates, &repo.top)
                    }
                    Err(failure) if failure.permits_fallback() => Self::walk(source, dir, root_rel),
                    Err(failure) => Self::failed(dir, root_rel, failure),
                }
            }
            Ok(None) => Self::walk(source, dir, root_rel),
            Err(failure) => Self::failed(dir, root_rel, &failure),
        }
    }

    /// A Git-listed root, plus the one ignored-root check.
    fn git(dir: &Path, root_rel: PathBuf, candidates: BTreeSet<PathBuf>, repo_top: &Path) -> Self {
        let mut view = Self {
            dir: dir.to_path_buf(),
            root_rel,
            candidates,
            mode: RootMode::Git,
            ignored: false,
            error: None,
        };
        match ignored_root(&view.dir, repo_top) {
            Ok(ignored) => view.ignored = ignored,
            Err(failure) => view.error = Some(root_error(&view.root_rel, &failure)),
        }
        view
    }

    /// A policy walk for a root Git cannot see.
    fn walk(source: &TaskSource, dir: &Path, root_rel: PathBuf) -> Self {
        let candidates = (source.typ_files(dir).into_iter())
            .filter_map(|file| {
                let path = source.task_path(&file);
                is_candidate(source, &path).then(|| path.abs().to_path_buf())
            })
            .collect();
        Self {
            dir: dir.to_path_buf(),
            root_rel,
            candidates,
            mode: RootMode::Walk,
            ignored: false,
            error: None,
        }
    }

    /// A root Git could not list.
    fn failed(dir: &Path, root_rel: PathBuf, failure: &GitFailure) -> Self {
        let error = Some(root_error(&root_rel, failure));
        Self {
            dir: dir.to_path_buf(),
            root_rel,
            candidates: BTreeSet::new(),
            mode: RootMode::Failed,
            ignored: false,
            error,
        }
    }

    /// Whether a file at `path` would be loaded from this root.
    fn admits(&self, path: &TaskPath) -> Result<(), NotVisible> {
        if let Some(error) = &self.error {
            return Err(NotVisible::Git(error.message.clone()));
        }
        match self.mode {
            RootMode::Walk => Ok(()),
            RootMode::Git => {
                if self.candidates.contains(path.abs()) {
                    return Ok(());
                }
                match tracked_at(path.abs()) {
                    Ok(true) => return Ok(()),
                    Ok(false) => {}
                    Err(failure) => {
                        return Err(NotVisible::Git(reason(&self.root_rel, &failure)));
                    }
                }
                match ignored_at(path.abs(), false) {
                    Ok(true) => Err(NotVisible::Ignored),
                    Ok(false) => Ok(()),
                    Err(failure) => Err(NotVisible::Git(reason(&self.root_rel, &failure))),
                }
            }
            RootMode::Failed => Err(NotVisible::Git(
                self.error
                    .as_ref()
                    .map_or_else(|| "git failed".to_owned(), |error| error.message.clone()),
            )),
        }
    }

    /// The task sources this root exposes, absolute and sorted.
    pub(super) fn sources(&self, source: &TaskSource) -> Vec<PathBuf> {
        (self.candidates.iter())
            .filter(|file| source.is_source(&source.task_path(file)))
            .cloned()
            .collect()
    }
}

/// Whether `path` is a candidate the loader must read: a source, or a reserved
/// `.typ` file a `task.with` call would explain, inside the walked area.
fn is_candidate(source: &TaskSource, path: &TaskPath) -> bool {
    (source.is_source(path) || source.is_reserved(path))
        && path
            .abs()
            .parent()
            .is_none_or(|parent| source.descends_into(parent))
}

/// The Git-listed candidates under one root, made absolute.
fn git_candidates(
    source: &TaskSource,
    dir: &Path,
    prefix: &Path,
    listing: &RepoListing,
) -> BTreeSet<PathBuf> {
    (listing.files.iter())
        .filter_map(|path| {
            let rel = path.strip_prefix(prefix).ok()?;
            let path = source.task_path(&dir.join(rel));
            is_candidate(source, &path).then(|| path.abs().to_path_buf())
        })
        .collect()
}

/// The load error a failed root listing contributes, naming the root.
fn root_error(root_rel: &Path, failure: &GitFailure) -> LoadError {
    LoadError {
        file: root_rel.to_path_buf(),
        message: reason(root_rel, failure),
        ..LoadError::default()
    }
}

/// Whether the repository ignores the root itself. A root at the
/// repository's top level can never be ignored, and its parent is outside the
/// work tree, so the probe starts below it only when it can.
fn ignored_root(dir: &Path, repo_top: &Path) -> Result<bool, GitFailure> {
    let parent = dir.parent().unwrap_or(dir);
    if !existing_dir(parent).starts_with(repo_top) {
        return Ok(false);
    }
    ignored_at(dir, true)
}

/// A message naming a root and Git's reason.
fn reason(root_rel: &Path, failure: &GitFailure) -> String {
    format!(
        "can't list the task root `{}` with git: {}",
        crate::display_path(root_rel),
        failure.reason()
    )
}

/// The one warning an ignored root earns.
fn ignored_warning(root_rel: &Path) -> LoadWarning {
    LoadWarning {
        file: root_rel.to_path_buf(),
        line: None,
        column: None,
        message: "ignored task root: only its tracked files are visible".to_owned(),
        hints: vec!["unignore the root to load its untracked files".to_owned()],
    }
}

/// A prospective task path against the whole policy: roots, `.typ`, reserved
/// paths, templates, the kind of entry there, and Git's working-tree view.
/// The path may not exist yet.
///
/// # Errors
/// [`NotVisible`], whose variant says which rule refused the destination.
pub fn would_be_visible(root: &Path, config: &Config, path: &Path) -> Result<(), NotVisible> {
    let source = TaskSource::new(root, config);
    let task_path = source.task_path(path);
    destination(&source, &task_path)?;
    VisibleFiles::new(root, config).admits(&task_path)
}

/// The policy half of [`would_be_visible`], before any Git read.
fn destination(source: &TaskSource, path: &TaskPath) -> Result<(), NotVisible> {
    if !(source.roots().iter()).any(|root| path.abs().starts_with(root)) {
        return Err(NotVisible::OutsideRoots);
    }
    if path.kind() == TaskPathKind::Symlink {
        return Err(NotVisible::Symlink);
    }
    if path.abs().extension().and_then(OsStr::to_str) != Some("typ") {
        return Err(NotVisible::NotTyp);
    }
    if !source.would_be_source(path) {
        return Err(NotVisible::Reserved);
    }
    if path.abs().is_dir() {
        return Err(NotVisible::NotAFile);
    }
    Ok(())
}

/// Every task source under one configured root, absolute and sorted.
///
/// A tracked regular `.typ` file is included even when Git ignores it; an
/// untracked ignored file is not. A tracked file deleted from the working tree
/// is not (it is not a regular file). A root outside a work tree, or a machine
/// without `git`, walks the filesystem with the policy's skips and does not
/// follow directory symlinks. An ignored root contributes only its tracked
/// files; the loader emits its one warning.
///
/// # Errors
/// A Git failure other than a missing `git` or a non-worktree root, naming the
/// root.
pub fn root_task_sources(
    root: &Path,
    config: &Config,
    spec: &RootSpec,
) -> Result<Vec<PathBuf>, LoadError> {
    let source = TaskSource::new(root, config);
    let dir = source.task_path(&spec.normalized()).abs().to_path_buf();
    let view = RootView::discover(&source, &dir, &mut BTreeMap::new());
    match view.error {
        Some(error) => Err(error),
        None => Ok(view.sources(&source)),
    }
}

/// Every task source all configured roots expose, absolute, deduplicated and
/// sorted. Overlapping roots list a file once.
///
/// Includes a tracked file even under an ignore rule, in any root (including
/// an ignored root), and excludes an untracked ignored file, a symlink, and a
/// tracked-deleted file. A missing root contributes nothing.
///
/// # Errors
/// The first root's Git failure other than a missing `git` or a non-worktree
/// root, in config order.
pub fn visible_task_sources(root: &Path, config: &Config) -> Result<Vec<PathBuf>, LoadError> {
    let visible = VisibleFiles::new(root, config);
    if let Some(error) = visible.errors().next() {
        return Err(error.clone());
    }
    let mut files = BTreeSet::new();
    for view in visible.roots() {
        files.extend(view.sources(visible.source()));
    }
    Ok(files.into_iter().collect())
}

#[cfg(test)]
#[path = "visible_tests.rs"]
mod tests;
