//! Which commit a `commit-msg` hook's index becomes the child of.
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

use crate::commit::{CommitMsgError, git};

/// How far up the process tree to look for the `git commit` that runs the hook.
const MAX_DEPTH: usize = 12;

/// The revision the index is diffed against: `HEAD`, or while amending the
/// parent of `HEAD`, or the empty tree when the amended commit is a root.
pub(crate) fn base_rev(root: &Path) -> Result<String, CommitMsgError> {
    if !amending() {
        return Ok("HEAD".to_owned());
    }
    match git(root, &["rev-parse", "--verify", "--quiet", "HEAD^"]) {
        Ok(parent) => Ok(parent.trim().to_owned()),
        Err(CommitMsgError::Git { .. }) => {
            Ok(git(root, &["hash-object", "-t", "tree", "--stdin"])?
                .trim()
                .to_owned())
        }
        Err(other) => Err(other),
    }
}

/// Whether the `git commit` running this hook was given `--amend`.
fn amending() -> bool {
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
