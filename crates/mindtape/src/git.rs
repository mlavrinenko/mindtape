//! A minimal, dependency-free reader for the current git branch, used by the
//! condition language's `on_branch(...)` predicate ([`crate::filter::condition`]).
//!
//! Parses `.git/HEAD` directly rather than shelling out or adding a git
//! library: `ref: refs/heads/<name>` names a branch, a raw commit sha is a
//! detached `HEAD`, and a `.git` *file* (`gitdir: <path>`, the worktree/
//! submodule form) redirects to `<path>/HEAD`. Any IO error, or a `HEAD` this
//! module cannot parse, resolves to `None` — `on_branch` treats that the same
//! as "not a git repo".

use std::path::{Path, PathBuf};

/// Resolve the current branch name from `root`'s `.git`. `None` when `root`
/// is not a git working copy, `HEAD` is detached, or anything cannot be read.
#[must_use]
pub fn current_branch(root: &Path) -> Option<String> {
    let head_path = resolve_head_path(root)?;
    let contents = std::fs::read_to_string(head_path).ok()?;
    parse_head(&contents)
}

/// Resolve the path to the `HEAD` file that governs `root`: directly under a
/// `.git` directory, or under the directory a `.git` *file*'s `gitdir:` line
/// points at (the git-worktree / submodule form).
fn resolve_head_path(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git.join("HEAD"));
    }
    if dot_git.is_file() {
        let text = std::fs::read_to_string(&dot_git).ok()?;
        let gitdir = text.trim().strip_prefix("gitdir:")?.trim();
        let gitdir = PathBuf::from(gitdir);
        let gitdir = if gitdir.is_absolute() {
            gitdir
        } else {
            root.join(gitdir)
        };
        return Some(gitdir.join("HEAD"));
    }
    None
}

/// Parse a `HEAD` file's contents into a branch name. `ref: refs/heads/<name>`
/// yields `Some(name)`; anything else (a raw sha — detached `HEAD` — or a
/// malformed line) yields `None`.
fn parse_head(contents: &str) -> Option<String> {
    let line = contents.trim();
    let target = line.strip_prefix("ref:")?.trim();
    let name = target.strip_prefix("refs/heads/")?;
    (!name.is_empty()).then(|| name.to_owned())
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
