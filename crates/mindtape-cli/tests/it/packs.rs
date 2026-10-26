//! `mt packs`, against a bare git repository under the test's own directory,
//! which the project declares by a path relative to its root. No test
//! touches the network.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::config::pack_sources::hash;
use mindtape::test_support::{git_as_tester, git_stdout, tempdir_with_config, write_file};

use crate::support;

const RULE: &str = "[[tasks.rules]]\nname = \"acme-rule\"\nassert = 'kind == \"bug\"'\n";

/// The project config: `acme:commits` from `upstream/src.git` at `reference`.
fn config(reference: &str) -> String {
    format!(
        "extends = [\"acme:commits\"]\n\n[pack-sources.acme]\n\
         git = \"upstream/src.git\"\nref = \"{reference}\"\n"
    )
}

/// The source repository: a work tree `upstream/work` and its bare clone
/// `upstream/src.git`, which the project fetches from.
struct Upstream<'a> {
    root: &'a Path,
}

impl Upstream<'_> {
    fn work(&self) -> std::path::PathBuf {
        self.root.join("upstream/work")
    }

    /// Commit `commits.toml` holding `text`, tag the commit `tag`, push it
    /// to `src.git` (made on the first call), and return the commit.
    fn publish(&self, text: &str, tag: &str) -> String {
        let work = self.work();
        let first = !work.exists();
        write_file(&work, "commits.toml", text);
        if first {
            git_as_tester(&work, &["init", "--quiet"]);
        }
        git_as_tester(&work, &["add", "."]);
        git_as_tester(&work, &["commit", "--quiet", "-m", tag]);
        git_as_tester(&work, &["tag", "--force", tag]);
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
fn tree_hash(text: &str) -> String {
    let staged = tempfile::tempdir().unwrap();
    write_file(staged.path(), "commits.toml", text);
    hash(staged.path()).unwrap()
}

/// Write the lock: `acme` from `upstream/src.git` at `reference`, pinned to
/// `commit` and `sha256`.
fn lock(root: &Path, reference: &str, commit: &str, sha256: &str) {
    let text = format!(
        "[acme]\ngit = \"upstream/src.git\"\nref = \"{reference}\"\n\
         commit = \"{commit}\"\nsha256 = \"{sha256}\"\n"
    );
    write_file(root, ".mindtape/packs.lock", text);
}

/// A project that declares and locks `acme` at `v1`, its source published.
fn locked_project() -> (tempfile::TempDir, String, String) {
    let dir = tempdir_with_config(&config("v1"));
    let commit = Upstream { root: dir.path() }.publish(RULE, "v1");
    let sha = tree_hash(RULE);
    lock(dir.path(), "v1", &commit, &sha);
    (dir, commit, sha)
}

/// `mt <args>` in `cwd`, isolated under `root`: the exit code, stdout and
/// stderr.
fn mt_in(root: &Path, cwd: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = support::mt(root)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    support::parts_of(&out)
}

#[test]
fn install_fetches_a_locked_source_the_load_then_reads() {
    let (dir, commit, sha) = locked_project();
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();

    let (code, stdout, stderr) = mt_in(dir.path(), &sub, &["packs", "install"]);

    assert_eq!(code, Some(0), "{stderr}");
    let short = &commit[..7];
    assert!(
        stdout.starts_with(&format!("fetched acme {short} into ")),
        "{stdout}"
    );
    let tree = dir
        .path()
        .join(support::CACHE_HOME)
        .join("mindtape/packs")
        .join(&sha);
    assert!(tree.join("commits.toml").is_file(), "{stdout}");
    let (code, stdout, stderr) = mt_in(dir.path(), dir.path(), &["config", "show", "--origin"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        stdout.contains("rule tasks.rules.acme-rule: pack acme:commits\n"),
        "{stdout}"
    );
}

#[test]
fn a_second_install_counts_the_sources_already_cached() {
    let (dir, _, _) = locked_project();
    let (code, _, stderr) = mt_in(dir.path(), dir.path(), &["packs", "install"]);
    assert_eq!(code, Some(0), "{stderr}");

    let (code, stdout, stderr) = mt_in(dir.path(), dir.path(), &["packs", "install"]);

    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(stdout, "1 source up to date\n");
}

#[test]
fn install_json_names_each_source_and_whether_it_fetched() {
    let (dir, commit, sha) = locked_project();

    let (code, stdout, stderr) = mt_in(dir.path(), dir.path(), &["packs", "install", "--json"]);

    assert_eq!(code, Some(0), "{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let expected = serde_json::json!([
        {"source": "acme", "commit": commit, "sha256": sha, "fetched": true}
    ]);
    assert_eq!(json, expected, "{stdout}");
}

/// How a refusal case breaks a [`locked_project`], given its root, commit
/// and tree hash.
type Break = fn(&Path, &str, &str);

#[test]
fn install_refuses_a_lock_it_cannot_restore_and_caches_nothing() {
    let cases: [(&str, Break, &[&str]); 3] = [
        (
            "a lock the sources disagree with",
            |root, _, _| write_file(root, ".mindtape/config.toml", config("v2")),
            &["lock is stale: 'acme'", "mt packs update"],
        ),
        (
            "a tree of another hash",
            |root, commit, _| lock(root, "v1", commit, &tree_hash("other = 1\n")),
            &["fetch pack source: 'acme'", "hash mismatch"],
        ),
        (
            "a commit its ref does not reach",
            |root, _, sha| lock(root, "v1", &"0".repeat(40), sha),
            &["is not on ref 'v1'", "hint: run `mt packs update`"],
        ),
    ];
    for (case, break_it, expected) in cases {
        let (dir, commit, sha) = locked_project();
        break_it(dir.path(), &commit, &sha);

        let (code, _, stderr) = mt_in(dir.path(), dir.path(), &["packs", "install"]);

        assert_eq!(code, Some(1), "{case}: {stderr}");
        for text in expected {
            assert!(stderr.contains(text), "{case}: {stderr}");
        }
        let packs = dir.path().join(support::CACHE_HOME).join("mindtape/packs");
        let cached = std::fs::read_dir(&packs).map_or(0, Iterator::count);
        assert_eq!(cached, 0, "{case}: {stderr}");
    }
}
