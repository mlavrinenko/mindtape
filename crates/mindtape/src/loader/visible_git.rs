//! The Git plumbing under the working-tree view.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::git::state::git_bytes;
use crate::query::{GitFailure, LS_FILES, nul_paths, path_from_bytes};

/// One repository's working-tree listing, read once per load.
pub(super) struct RepoListing {
    /// Repository-relative paths, exactly as Git printed them.
    pub(super) files: BTreeSet<PathBuf>,
}

/// A root's place in its repository.
pub(super) struct RepoRef {
    /// The repository root, used as the listing cache key.
    pub(super) top: PathBuf,
    /// The root's prefix inside the repository, with a trailing separator.
    pub(super) prefix: PathBuf,
}

/// Ask Git for `dir`'s repository and prefix, or `None` when discovery must
/// fall back. Only a missing Git or a non-worktree directory falls back; every
/// other failure is returned.
pub(super) fn repo_of(dir: &Path) -> Result<Option<RepoRef>, GitFailure> {
    let base = existing_dir(dir);
    let inside = match git_bytes(base, &["rev-parse", "--is-inside-work-tree"]) {
        Ok(out) => out,
        Err(reason) => {
            let failure = GitFailure::classify(&reason);
            return if failure.permits_fallback() {
                Ok(None)
            } else {
                Err(failure)
            };
        }
    };
    if !is_worktree(&inside) {
        return Ok(None);
    }
    let out = match git_bytes(base, &["rev-parse", "--show-toplevel", "--show-prefix"]) {
        Ok(out) => out,
        Err(reason) => {
            let failure = GitFailure::classify(&reason);
            return if failure.permits_fallback() {
                Ok(None)
            } else {
                Err(failure)
            };
        }
    };
    parse_repo_ref(&out, dir, base).map(Some)
}

/// Whether `git rev-parse --is-inside-work-tree` printed `true`.
fn is_worktree(out: &[u8]) -> bool {
    String::from_utf8_lossy(out).trim() == "true"
}

/// Parse `git rev-parse --show-toplevel --show-prefix` output, adjusting the
/// prefix for a root below the nearest existing directory `base`.
fn parse_repo_ref(out: &[u8], dir: &Path, base: &Path) -> Result<RepoRef, GitFailure> {
    let mut lines = out.split(|byte| *byte == b'\n');
    let (Some(top), Some(prefix)) = (lines.next(), lines.next()) else {
        return Err(GitFailure::Other(
            "git rev-parse printed no work-tree prefix".to_owned(),
        ));
    };
    let prefix = path_from_bytes(prefix);
    let prefix = dir.strip_prefix(base).map_or(prefix.clone(), |extra| {
        crate::config::roots::normalize_path(&prefix.join(extra))
    });
    Ok(RepoRef {
        top: path_from_bytes(top),
        prefix,
    })
}

/// `dir` or its nearest existing ancestor, the directory Git can run in for a
/// root that does not exist yet.
pub(super) fn existing_dir(dir: &Path) -> &Path {
    dir.ancestors().find(|path| path.is_dir()).unwrap_or(dir)
}

impl RepoListing {
    /// List one repository: tracked files and untracked files Git does not
    /// ignore, from the repository root, NUL-separated.
    pub(super) fn read(top: &Path) -> Result<Self, GitFailure> {
        let listed = git_bytes(top, &LS_FILES).map_err(|reason| GitFailure::classify(&reason))?;
        Ok(Self {
            files: nul_paths(&listed).into_iter().collect(),
        })
    }
}

/// Whether Git ignores `path`, reading only exclude rules (`--no-index`) so a
/// tracked file under an ignore rule still reports ignored. `path` may be
/// absolute and may not exist. When `as_dir` is set, a path that is not a
/// directory yet is probed with a trailing slash, so a `target/` rule matches
/// a root that does not exist.
///
/// The query runs from the path's nearest existing parent, with the path
/// relative to it: Git rejects an absolute pathspec that traverses a symlink,
/// and a `/.` suffix defeats a negated directory rule (`!tasks/`).
pub(super) fn ignored_at(path: &Path, as_dir: bool) -> Result<bool, GitFailure> {
    let (base, mut rel) = probe_path(path);
    if as_dir && !path.is_dir() {
        rel = with_trailing_slash(&rel);
    }
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(base)
        .args(["check-ignore", "-q", "--no-index", "--"])
        .arg(rel)
        .output()
        .map_err(|_| GitFailure::Unavailable)?;
    ignored_result(output.status.code(), &output.stderr)
}

/// Whether the index tracks `path`. A tracked file deleted from the work tree
/// is tracked, so a write there lands on a path Git still lists.
pub(super) fn tracked_at(path: &Path) -> Result<bool, GitFailure> {
    let (base, rel) = probe_path(path);
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(base)
        .args(["ls-files", "-z", "--cached", "--error-unmatch", "--"])
        .arg(rel)
        .output()
        .map_err(|_| GitFailure::Unavailable)?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(failure_from_stderr(&output.stderr)),
    }
}

/// `path`'s nearest existing parent and the path relative to it. The
/// remaining components do not exist, so they contain no symlink Git must
/// traverse.
fn probe_path(path: &Path) -> (&Path, PathBuf) {
    let parent = path.parent().unwrap_or(path);
    let base = existing_dir(parent);
    let rel = path.strip_prefix(base).unwrap_or(path);
    (base, rel.to_path_buf())
}

/// `path` with a trailing `/`, which gives Git check-ignore's pathname
/// directory semantics on every platform.
fn with_trailing_slash(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push("/");
    PathBuf::from(os)
}

/// Interpret `git check-ignore`'s exit status and stderr.
fn ignored_result(status: Option<i32>, stderr: &[u8]) -> Result<bool, GitFailure> {
    match status {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(failure_from_stderr(stderr)),
    }
}

/// The Git failure a non-zero, non-`1` exit reports: the first stderr line,
/// without Git's `fatal: ` label.
fn failure_from_stderr(stderr: &[u8]) -> GitFailure {
    let stderr = String::from_utf8_lossy(stderr);
    let reason = stderr
        .lines()
        .next()
        .unwrap_or("git check-ignore failed")
        .trim()
        .trim_start_matches("fatal: ")
        .to_owned();
    GitFailure::Other(reason)
}

#[cfg(test)]
#[path = "visible_git_tests.rs"]
mod tests;
