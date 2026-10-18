//! `.just/scripts/wt-carry-lcov.sh` copies the worktree's coverage lcov to the
//! primary checkout when the worktrunk `pre-remove` hook fires, so `just crap::check`
//! there scores a landed merge without a second instrumented build. Each test
//! builds a primary checkout and a worktree of it, runs the script from the
//! worktree root as the hook does, and reads what reached the primary.
//!
//! Unix only: the script needs `bash`, `git` and `awk`, which the dev shell
//! provides and the Windows job, which has no dev shell, does not.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support::checkout;
use mindtape::test_support::tempdir;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime};

const SCRIPT: &str = ".just/scripts/wt-carry-lcov.sh";
const STALE_SCRIPT: &str = ".just/scripts/_lcov-stale-reason.sh";
const LCOV: &str = "target/coverage/lcov.info";

struct Pair {
    dir: tempfile::TempDir,
}

impl Pair {
    /// A primary checkout on `main` and a worktree on `feature` at the same
    /// commit, both holding the two scripts and one source file.
    fn new() -> Self {
        let pair = Self { dir: tempdir() };
        let primary = pair.primary();
        std::fs::create_dir_all(&primary).unwrap();
        git(&primary, &["init", "-q", "-b", "main"]);
        git(&primary, &["config", "user.email", "t@example.com"]);
        git(&primary, &["config", "user.name", "t"]);
        git(&primary, &["config", "commit.gpgsign", "false"]);
        for path in [SCRIPT, STALE_SCRIPT] {
            write(
                &primary.join(path),
                &std::fs::read_to_string(checkout().join(path)).unwrap(),
            );
        }
        write(&primary.join("crates/a/src/lib.rs"), "pub fn a() {}\n");
        commit(&primary, "base");
        let worktree = pair.worktree();
        git(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "feature",
                worktree.to_str().unwrap(),
            ],
        );
        pair
    }

    fn primary(&self) -> PathBuf {
        self.dir.path().join("repo")
    }

    fn worktree(&self) -> PathBuf {
        self.dir.path().join("repo-feature")
    }

    /// Write the worktree's lcov, as `cargo llvm-cov` does: absolute `SF:`
    /// paths into the worktree, written after the sources.
    fn instrument(&self) {
        let wt = self.worktree();
        let lcov = format!(
            "TN:\nSF:{}/crates/a/src/lib.rs\nDA:1,1\nend_of_record\n",
            wt.display()
        );
        write(&wt.join(LCOV), &lcov);
        age(&wt.join("crates/a/src/lib.rs"), 3600);
    }

    /// Run the script as the `pre-remove` hook does: from the worktree root,
    /// with the primary checkout as its argument.
    fn carry(&self, primary: &Path) -> Output {
        crate::support::shell("bash", &self.worktree())
            .arg(SCRIPT)
            .arg(primary)
            .output()
            .unwrap()
    }

    /// Run the script with an `mmz` first on `PATH` that answers
    /// `--is-fresh -- just cover` with `exit_code`.
    fn carry_with_mmz(&self, exit_code: i32) -> Output {
        let bin = self.dir.path().join("bin");
        write(&bin.join("mmz"), &format!("#!/bin/sh\nexit {exit_code}\n"));
        std::fs::set_permissions(
            bin.join("mmz"),
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .unwrap();
        let path = format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        crate::support::shell("bash", &self.worktree())
            .arg(SCRIPT)
            .arg(self.primary())
            .env("PATH", path)
            .output()
            .unwrap()
    }

    /// A worktree whose lcov is older than its source by mtime, as after a
    /// rebase, and the `mmz` verdict on `just cover`, run through the hook.
    fn rebased_carry(&self, mmz_exit: i32) -> Output {
        self.instrument();
        age(&self.worktree().join(LCOV), 7200);
        self.carry_with_mmz(mmz_exit)
    }

    fn carried(&self) -> Option<String> {
        std::fs::read_to_string(self.primary().join(LCOV)).ok()
    }
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

/// Set `path`'s mtime to `secs` seconds in the past.
fn age(path: &Path, secs: u64) {
    let file = std::fs::File::options().write(true).open(path).unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(secs))
        .unwrap();
}

#[test]
fn a_landed_merge_carries_the_lcov_with_its_paths_rewritten() {
    let pair = Pair::new();
    pair.instrument();
    let out = pair.carry(&pair.primary());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let carried = pair.carried().expect("the lcov reached the primary");
    let primary = pair.primary();
    assert!(
        carried.contains(&format!("SF:{}/crates/a/src/lib.rs", primary.display())),
        "SF path was not rewritten:\n{carried}"
    );
    assert!(!carried.contains(pair.worktree().to_str().unwrap()));
}

#[test]
fn a_branch_that_did_not_land_carries_nothing() {
    let pair = Pair::new();
    pair.instrument();
    let wt = pair.worktree();
    write(&wt.join("crates/a/src/other.rs"), "pub fn b() {}\n");
    commit(&wt, "work");
    let out = pair.carry(&pair.primary());
    assert!(out.status.success());
    assert!(
        pair.carried().is_none(),
        "records of an unlanded tree moved"
    );
}

#[test]
fn a_rebased_branch_whose_cover_record_is_fresh_still_carries() {
    // A rebase rewrites the files it replays, so the lcov is older than a
    // source by mtime while mmz, which hashes content, calls `just cover` fresh.
    let pair = Pair::new();
    let out = pair.rebased_carry(0);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(pair.carried().is_some(), "a content-fresh lcov was skipped");
}

#[test]
fn a_skipped_carry_says_why() {
    let pair = Pair::new();
    let out = pair.rebased_carry(1);
    assert!(out.status.success());
    assert!(pair.carried().is_none());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("wt-carry-lcov: skipped") && err.contains("predates"),
        "{err}"
    );
}

#[test]
fn a_newer_lcov_in_the_primary_is_kept() {
    let pair = Pair::new();
    pair.instrument();
    age(&pair.worktree().join(LCOV), 600);
    write(&pair.primary().join(LCOV), "primary's own\n");
    let out = pair.carry(&pair.primary());
    assert!(out.status.success());
    assert_eq!(pair.carried().as_deref(), Some("primary's own\n"));
}

#[test]
fn a_worktree_without_an_lcov_is_a_no_op() {
    let pair = Pair::new();
    let out = pair.carry(&pair.primary());
    assert!(out.status.success());
    assert!(pair.carried().is_none());
}

#[test]
fn a_failed_carry_says_so_and_never_fails_the_hook() {
    let pair = Pair::new();
    pair.instrument();
    // `target` is a file in the primary, so `target/coverage` cannot be made.
    write(&pair.primary().join("target"), "");
    let out = pair.carry(&pair.primary());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("could not carry"));
}

#[test]
fn a_primary_that_is_no_checkout_is_a_no_op() {
    let pair = Pair::new();
    pair.instrument();
    let missing = pair.dir.path().join("no-such-checkout");
    let out = pair.carry(&missing);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!missing.exists());
}
