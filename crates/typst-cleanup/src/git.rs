//! Repository plumbing for cleanup: locate the work-tree root and guard against
//! a dirty working tree, via the `gix` (gitoxide) crate. Cleanup does not commit
//! — the consumer reviews the diff and commits.

use std::path::{Path, PathBuf};

use gix::bstr::BString;

use crate::Error;

/// Absolute path to the repository root (work tree) containing `start`.
///
/// # Errors
/// [`Error::Git`] when `start` is not inside a git work tree (e.g. a bare
/// repository, or no repository at all).
pub fn toplevel(start: &Path) -> Result<PathBuf, Error> {
    let repo = gix::discover(start).map_err(|err| Error::Git(err.to_string()))?;
    repo.workdir()
        .map(Path::to_path_buf)
        .ok_or_else(|| Error::Git("repository has no work tree".to_owned()))
}

/// Whether the working tree at `root` has no uncommitted changes — tracked
/// modifications, staged changes, or untracked files (matching the breadth of
/// `git status --porcelain`).
///
/// # Errors
/// [`Error::Git`] when the repository cannot be opened or the status walk fails.
pub fn is_clean(root: &Path) -> Result<bool, Error> {
    let repo = gix::discover(root).map_err(|err| Error::Git(err.to_string()))?;
    let mut iter = repo
        .status(gix::progress::Discard)
        .map_err(|err| Error::Git(err.to_string()))?
        .untracked_files(gix::status::UntrackedFiles::Files)
        .into_iter(Vec::<BString>::new())
        .map_err(|err| Error::Git(err.to_string()))?;
    match iter.next() {
        Some(item) => {
            item.map_err(|err| Error::Git(err.to_string()))?;
            Ok(false)
        }
        None => Ok(true),
    }
}
