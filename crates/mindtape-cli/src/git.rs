//! Minimal git plumbing for `cleanup`: locate the repository root and guard
//! against a dirty working tree, via the `gix` (gitoxide) crate — a real git
//! object model, no subprocess.
//!
//! SWHID minting and resolution live in [`swhid_mint::git`]; this module owns
//! only the orchestration-level queries (repository discovery, worktree status)
//! that `swhid-mint` deliberately does not. `cleanup` does not commit — the user
//! reviews the diff and commits.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gix::bstr::BString;

/// Absolute path to the repository root (work tree) containing `start`.
///
/// # Errors
/// Fails when `start` is not inside a git work tree (e.g. a bare repository).
pub fn toplevel(start: &Path) -> Result<PathBuf> {
    let repo = gix::discover(start).context("discover git repository")?;
    repo.workdir()
        .map(Path::to_path_buf)
        .context("repository has no work tree")
}

/// Whether the working tree at `root` has no uncommitted changes — tracked
/// modifications, staged changes, or untracked files (matching the breadth of
/// `git status --porcelain`).
///
/// # Errors
/// Fails when the repository cannot be opened or the status walk fails.
pub fn is_clean(root: &Path) -> Result<bool> {
    let repo = gix::discover(root).context("discover git repository")?;
    let mut iter = repo
        .status(gix::progress::Discard)
        .context("start status")?
        .untracked_files(gix::status::UntrackedFiles::Files)
        .into_iter(Vec::<BString>::new())
        .context("iterate status")?;
    match iter.next() {
        Some(item) => {
            item.context("status item")?;
            Ok(false)
        }
        None => Ok(true),
    }
}
