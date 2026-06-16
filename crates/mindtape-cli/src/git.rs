//! Minimal git plumbing for `cleanup`: the working-tree guard and the
//! single-commit checkpoint. Shells out to the `git` binary (no libgit2).
//!
//! SWHID minting and resolution live in [`swhid_mint::git`]; this module owns
//! only the orchestration-level commands (`status`, `add`, `commit`,
//! `rev-parse`) that `swhid-mint` deliberately does not. A future shared git
//! crate could absorb both, but the two concerns stay separate for now.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

/// Run `git -C <root> <args>` and return trimmed stdout, failing on non-zero.
fn capture<const N: usize>(root: &Path, args: [&str; N]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .with_context(|| format!("run git {}", args.join(" ")))?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Absolute path to the repository root containing `start`.
///
/// # Errors
/// Fails when `start` is not inside a git work tree.
pub fn toplevel(start: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from(capture(
        start,
        ["rev-parse", "--show-toplevel"],
    )?))
}

/// Whether the working tree at `root` has no uncommitted changes.
///
/// # Errors
/// Fails when `git status` cannot run.
pub fn is_clean(root: &Path) -> Result<bool> {
    Ok(capture(root, ["status", "--porcelain"])?.is_empty())
}

/// Stage everything and create one commit; return the new commit hash.
///
/// # Errors
/// Fails when any of `git add`, `git commit`, or `git rev-parse` fails.
pub fn commit_all(root: &Path, message: &str) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["add", "-A"])
        .output()
        .context("run git add -A")?;
    if !out.status.success() {
        bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["commit", "-q", "-m", message])
        .output()
        .context("run git commit")?;
    if !out.status.success() {
        bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    capture(root, ["rev-parse", "HEAD"])
}
