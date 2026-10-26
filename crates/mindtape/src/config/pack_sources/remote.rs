//! Resolving a source's `ref` to the commit it names now: a commit id is
//! itself, and any other `ref` is looked up with `git ls-remote`, run through
//! the same [`Git`] as a fetch.
//!
//! A name matches the way `git fetch` reads it: as given, then under
//! `refs/`, `refs/tags/` and `refs/heads/`, so a tag wins over a branch of
//! the same name. An annotated tag names a tag object, so its peeled `^{}`
//! line, the commit it points to, wins over the tag's own line.

use std::ffi::OsStr;
use std::path::Path;

use crate::config::pack_sources::FetchError;
use crate::config::pack_sources::fetch::is_commit_id;
use crate::config::pack_sources::git::Git;

/// The commit `reference` of the repository `git` names now. git runs in
/// `cache_root`, as a fetch does.
///
/// # Errors
/// Returns [`FetchError`] when git is missing or fails, or the repository
/// has no such ref.
pub fn resolve(git: &str, reference: &str, cache_root: &Path) -> Result<String, FetchError> {
    resolve_with(OsStr::new("git"), git, reference, cache_root)
}

/// [`resolve`] through `program` in place of `git`.
pub(super) fn resolve_with(
    program: &OsStr,
    git: &str,
    reference: &str,
    cache_root: &Path,
) -> Result<String, FetchError> {
    if is_commit_id(reference) {
        return Ok(reference.to_owned());
    }
    let runner = Git::new(program, cache_root)?;
    // A pattern matches a ref's tail, and a peeled line's tail is `^{}`.
    let peeled = format!("{reference}^{{}}");
    let args = ["ls-remote", "--end-of-options", git, reference, &peeled].map(OsStr::new);
    let listing = runner.run(None, &args, "ls-remote")?;
    pick(&String::from_utf8_lossy(&listing), reference).ok_or_else(|| FetchError::UnknownRef {
        reference: reference.to_owned(),
    })
}

/// The commit `reference` names in `listing`, the `<id>\t<ref>` lines of
/// `git ls-remote`.
pub(super) fn pick(listing: &str, reference: &str) -> Option<String> {
    let lines: Vec<(&str, &str)> = listing
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .collect();
    let find = |name: &str| {
        lines
            .iter()
            .find(|(_, listed)| *listed == name)
            .map(|(id, _)| (*id).to_owned())
    };
    ["", "refs/", "refs/tags/", "refs/heads/"]
        .iter()
        .map(|prefix| format!("{prefix}{reference}"))
        .find_map(|name| find(&format!("{name}^{{}}")).or_else(|| find(&name)))
}

#[cfg(test)]
#[path = "remote_tests.rs"]
mod tests;
