//! `mt packs`, against a bare git repository under the test's own directory,
//! which the project declares by a path relative to its root. No test
//! touches the network.
//!
//! - `install.rs`: `mt packs install`.
//! - `update.rs`: `mt packs update`.
#![allow(clippy::unwrap_used)]

mod install;
mod update;

use std::path::Path;

use mindtape::config::pack_sources::hash;
use mindtape::test_support::{git_as_tester, git_stdout, tempdir_with_config, write_file};

use crate::support;

pub(super) const RULE: &str = "[[tasks.rules]]\nname = \"acme-rule\"\nassert = 'kind == \"bug\"'\n";

/// The project config: `acme:commits` from `upstream/src.git` at `reference`.
pub(super) fn config(reference: &str) -> String {
    format!(
        "extends = [\"acme:commits\"]\n\n[pack-sources.acme]\n\
         git = \"upstream/src.git\"\nref = \"{reference}\"\n"
    )
}

/// The source repository: a work tree `upstream/work` and its bare clone
/// `upstream/src.git`, which the project fetches from.
pub(super) struct Upstream<'a> {
    pub(super) root: &'a Path,
}

impl Upstream<'_> {
    fn work(&self) -> std::path::PathBuf {
        self.root.join("upstream/work")
    }

    /// Commit `commits.toml` holding `text`, give the commit the annotated
    /// tag `tag`, moving it if it exists, push both to `src.git` (made on the
    /// first call), and return the commit.
    pub(super) fn publish(&self, text: &str, tag: &str) -> String {
        let work = self.work();
        let first = !work.exists();
        write_file(&work, "commits.toml", text);
        if first {
            git_as_tester(&work, &["init", "--quiet"]);
        }
        git_as_tester(&work, &["add", "."]);
        git_as_tester(&work, &["commit", "--quiet", "-m", tag]);
        git_as_tester(&work, &["tag", "--force", "-a", "-m", tag, tag]);
        let upstream = self.root.join("upstream");
        if first {
            git_as_tester(
                &upstream,
                &["clone", "--quiet", "--bare", "work", "src.git"],
            );
        } else {
            git_as_tester(
                &work,
                &["push", "--quiet", "--force", "../src.git", "--tags"],
            );
        }
        git_stdout(&work, &["rev-parse", "HEAD"])
    }
}

/// The hash of a tree holding `commits.toml` with `text`.
pub(super) fn tree_hash(text: &str) -> String {
    let staged = tempfile::tempdir().unwrap();
    write_file(staged.path(), "commits.toml", text);
    hash(staged.path()).unwrap()
}

/// Write the lock: `acme` from `upstream/src.git` at `reference`, pinned to
/// `commit` and `sha256`.
pub(super) fn lock(root: &Path, reference: &str, commit: &str, sha256: &str) {
    let text = format!(
        "[acme]\ngit = \"upstream/src.git\"\nref = \"{reference}\"\n\
         commit = \"{commit}\"\nsha256 = \"{sha256}\"\n"
    );
    write_file(root, ".mindtape/packs.lock", text);
}

/// A project that declares and locks `acme` at `v1`, its source published.
pub(super) fn locked_project() -> (tempfile::TempDir, String, String) {
    let dir = tempdir_with_config(&config("v1"));
    let commit = Upstream { root: dir.path() }.publish(RULE, "v1");
    let sha = tree_hash(RULE);
    lock(dir.path(), "v1", &commit, &sha);
    (dir, commit, sha)
}

/// `mt <args>` in `cwd`, isolated under `root`: the exit code, stdout and
/// stderr.
pub(super) fn mt_in(root: &Path, cwd: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = support::mt(root)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    support::parts_of(&out)
}

/// Assert that `root`'s config loads `acme:commits` from the cache.
pub(super) fn assert_resolves_pack(root: &Path) {
    let (code, stdout, stderr) = mt_in(root, root, &["config", "show", "--origin"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        stdout.contains("rule tasks.rules.acme-rule: pack acme:commits\n"),
        "{stdout}"
    );
}
