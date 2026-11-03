//! Which commit a `commit-msg` hook's index becomes the child of, and whether
//! it is a merge.
//!
//! A new commit follows `HEAD`. `git commit --amend` replaces `HEAD`, so the
//! commit it makes follows `HEAD^` and the index is judged against that.
//!
//! The hook cannot be told which one it is in. git exports no variable for
//! it: `GIT_REFLOG_ACTION` is unset in a `commit-msg` hook, and
//! `prepare-commit-msg` gets the same `commit HEAD` arguments for
//! `--amend` as for `-C HEAD`, a plain new commit. The one reliable witness
//! is the `git commit` process that runs the hook, which is an ancestor of
//! this one and has `--amend` on its command line. [`amending`] walks up the
//! process tree to the nearest `git commit` and looks for that flag. `--amend`
//! has no short form, so the match is exact, and `git rebase` rewords through
//! `git commit --amend`, so it is judged the same way. Where the process
//! tree cannot be read (Windows, no `ps`) or the command was a git alias that
//! hides its arguments, the answer is no and the commit is judged against
//! `HEAD`, as before.

use std::path::Path;
use std::process::Command;

use crate::commit::{CommitMsgError, git, git_optional};

/// How far up the process tree to look for the `git commit` that runs the hook.
const MAX_DEPTH: usize = 12;

/// The revision the index is diffed against: `HEAD`, or while amending the
/// parent of `HEAD`, or the empty tree when the commit it follows is a root or
/// the repository has no commit yet.
pub(crate) fn base_rev(root: &Path) -> Result<String, CommitMsgError> {
    if !amending() {
        return match git_optional(root, &["rev-parse", "--verify", "--quiet", "HEAD"])? {
            Some(_) => Ok("HEAD".to_owned()),
            None => empty_tree(root),
        };
    }
    match git(root, &["rev-parse", "--verify", "--quiet", "HEAD^"]) {
        Ok(parent) => Ok(parent.trim().to_owned()),
        Err(CommitMsgError::Git { .. }) => empty_tree(root),
        Err(other) => Err(other),
    }
}

/// The empty tree's object id, the parent a root commit is read against.
fn empty_tree(root: &Path) -> Result<String, CommitMsgError> {
    Ok(git(root, &["hash-object", "-t", "tree", "--stdin"])?
        .trim()
        .to_owned())
}

/// Whether the commit a `commit-msg` check is judging is a merge: a merge is
/// in progress (`MERGE_HEAD`), or `git commit --amend` replaces a commit with
/// more than one parent. An amend has no `MERGE_HEAD`, since the merge it
/// replaces is already committed.
///
/// # Errors
/// [`CommitMsgError::NotGit`] when `root` is not inside a git working tree;
/// [`CommitMsgError::Git`] / [`CommitMsgError::Spawn`] when git fails.
pub fn writing_a_merge(root: &Path) -> Result<bool, CommitMsgError> {
    let merge_head = git_optional(root, &["rev-parse", "-q", "--verify", "MERGE_HEAD"]).map_err(
        |err| match err {
            CommitMsgError::Git { .. } => CommitMsgError::NotGit(crate::display_path(root)),
            other => other,
        },
    )?;
    if merge_head.is_some() {
        return Ok(true);
    }
    if !amending() {
        return Ok(false);
    }
    match git_optional(root, &["rev-list", "--parents", "-n", "1", "HEAD"]) {
        Ok(Some(line)) => Ok(line.split_whitespace().count() > 2),
        Ok(None) | Err(CommitMsgError::Git { .. }) => Ok(false),
        Err(other) => Err(other),
    }
}

/// Whether the `git commit` running this hook was given `--amend`.
pub(crate) fn amending() -> bool {
    let mut chain = Vec::new();
    let mut pid = std::process::id();
    for _ in 0..MAX_DEPTH {
        let Some((parent, argv)) = process(pid) else {
            break;
        };
        chain.push(argv);
        pid = parent;
    }
    is_amend(&chain)
}

/// Whether the nearest `git commit` in `chain` (argument lists, nearest
/// process first) carries `--amend`.
pub(crate) fn is_amend(chain: &[Vec<String>]) -> bool {
    chain
        .iter()
        .find(|argv| is_git_commit(argv))
        .is_some_and(|argv| argv.iter().any(|arg| arg == "--amend"))
}

fn is_git_commit(argv: &[String]) -> bool {
    let is_git = argv.first().is_some_and(|program| {
        let name = program.rsplit(['/', '\\']).next().unwrap_or_default();
        name == "git" || name == "git.exe"
    });
    is_git && argv.iter().skip(1).any(|arg| arg == "commit")
}

/// The parent pid and the arguments of process `pid`, from `/proc` where it
/// exists and from `ps` elsewhere.
fn process(pid: u32) -> Option<(u32, Vec<String>)> {
    from_proc(pid).or_else(|| from_ps(pid))
}

fn from_proc(pid: u32) -> Option<(u32, Vec<String>)> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // `pid (comm) state ppid ...`; comm may itself hold spaces and parens.
    let after_comm = stat.get(stat.rfind(')')? + 1..)?;
    let parent = after_comm.split_whitespace().nth(1)?.parse().ok()?;
    let raw = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let argv = raw
        .split(|byte| *byte == 0)
        .filter(|arg| !arg.is_empty())
        .map(|arg| String::from_utf8_lossy(arg).into_owned())
        .collect();
    Some((parent, argv))
}

fn from_ps(pid: u32) -> Option<(u32, Vec<String>)> {
    let ps = |field: &str| {
        let out = Command::new("ps")
            .args(["-o", field, "-p", &pid.to_string()])
            .output()
            .ok()
            .filter(|out| out.status.success())?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    };
    let parent = ps("ppid=")?.parse().ok()?;
    let argv = ps("args=")?
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect();
    Some((parent, argv))
}

#[cfg(test)]
#[path = "amend_tests.rs"]
mod tests;
