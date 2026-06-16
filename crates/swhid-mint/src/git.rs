//! Git backend: derive and resolve SWHIDs against a local git repository by
//! shelling out to the `git` binary — the single `Command::new("git")` site.
//!
//! For content/directory/revision the SWHID hash is `sha1_git`, identical to
//! git's own object SHA-1, so we read it straight from git and prefix the kind:
//! no hashing here, no libgit2, no extra dependency.

use std::path::Path;
use std::process::{Command, Output};

use crate::{Kind, Swhid};

/// Why a git-backed SWHID operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The `git` invocation failed (non-zero exit, or could not be spawned).
    Git(String),
    /// No commit touches the path — nothing to pin a revision id to.
    NotTracked(String),
    /// The working-tree blob differs from the blob recorded at the commit, so a
    /// minted id would not recover the current bytes.
    Dirty(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git(msg) => write!(f, "git: {msg}"),
            Self::NotTracked(path) => write!(f, "{path}: no commit touches this path"),
            Self::Dirty(path) => write!(
                f,
                "{path}: working tree differs from the recorded blob; commit first"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// Mint a revision-scoped SWHID for `path` (repository-relative): its last-touch
/// commit, qualified by path. Verifies the working-tree blob still equals the
/// blob recorded at that commit, so the id recovers the same bytes via
/// `git show <commit>:<path>`.
///
/// # Errors
/// [`Error::NotTracked`] when no commit touches the path, [`Error::Dirty`] when
/// the working tree diverges from the recorded blob, [`Error::Git`] on any git
/// failure.
pub fn mint_rev_path(root: &Path, path: &str) -> Result<Swhid, Error> {
    let commit = capture(root, &["log", "-1", "--format=%H", "--", path])?;
    if commit.is_empty() {
        return Err(Error::NotTracked(path.to_owned()));
    }
    let working = capture(root, &["hash-object", path])?;
    let recorded = ls_tree_blob(root, &commit, path)?;
    if working != recorded {
        return Err(Error::Dirty(path.to_owned()));
    }
    Ok(Swhid {
        kind: Kind::Revision,
        hash: commit,
        path: Some(path.to_owned()),
    })
}

/// Mint a content (`cnt`) SWHID for `path` from its working-tree blob.
///
/// # Errors
/// [`Error::Git`] when `git hash-object` fails.
pub fn mint_content(root: &Path, path: &str) -> Result<Swhid, Error> {
    let hash = capture(root, &["hash-object", path])?;
    Ok(Swhid {
        kind: Kind::Content,
        hash,
        path: None,
    })
}

/// True when `id` still resolves in `root`: its object exists, and when a `path`
/// qualifier is present, the path exists at that revision.
///
/// # Errors
/// [`Error::Git`] only when git cannot be invoked; a clean "does not resolve" is
/// `Ok(false)`.
pub fn resolves(root: &Path, id: &Swhid) -> Result<bool, Error> {
    let target = match &id.path {
        Some(path) => format!("{}:{}", id.hash, path),
        None => id.hash.clone(),
    };
    succeeds(root, &["cat-file", "-e", &target])
}

fn run(root: &Path, args: &[&str]) -> Result<Output, Error> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|err| Error::Git(format!("cannot run git: {err}")))
}

fn capture(root: &Path, args: &[&str]) -> Result<String, Error> {
    let out = run(root, args)?;
    if !out.status.success() {
        return Err(Error::Git(
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn succeeds(root: &Path, args: &[&str]) -> Result<bool, Error> {
    Ok(run(root, args)?.status.success())
}

fn ls_tree_blob(root: &Path, commit: &str, path: &str) -> Result<String, Error> {
    // Line shape: "<mode> blob <sha>\t<path>".
    let line = capture(root, &["ls-tree", commit, path])?;
    line.split_ascii_whitespace()
        .nth(2)
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::Git(format!("ls-tree gave no blob for {path} at {commit}")))
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
