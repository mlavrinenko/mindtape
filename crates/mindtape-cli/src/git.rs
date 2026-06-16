//! Minimal git plumbing for `cleanup`: locate the repository root and guard
//! against a dirty working tree. Shells out to the `git` binary (no libgit2).
//!
//! SWHID minting and resolution live in [`swhid_mint::git`]; this module owns
//! only the orchestration-level commands (`rev-parse`, `status`) that
//! `swhid-mint` deliberately does not. `cleanup` does not commit — the user
//! reviews the diff and commits. A future shared git crate could absorb both,
//! but the two concerns stay separate for now.

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
