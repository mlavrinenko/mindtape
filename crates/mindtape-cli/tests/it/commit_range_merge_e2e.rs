//! End-to-end tests for merge commits in `mt check --commits`: skipped by
//! default, read against their first parent with `--check-merges`, and still
//! the tip state whose tree the `scope = "tip"` rules read.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::commit_range_e2e::{FINISH_AT_THE_TIP, check_range, check_range_with, tip_breach};
use crate::support;
use support::{commit, commit_repo, git, stage_file};

/// A repository whose `HEAD` is a `--no-ff` merge of `side` into its first
/// parent, with `side` and `main` as the branch commit messages. Both branches
/// add a file, so the merge changes both against either parent.
fn merged_with(dir: &Path, side: &str, main: &str) {
    git(dir, &["checkout", "-q", "-b", "side"]);
    stage_file(dir, "src/b.rs");
    commit(dir, side);
    git(dir, &["checkout", "-q", "-"]);
    stage_file(dir, "src/a.rs");
    commit(dir, main);
    git(
        dir,
        &[
            "-c",
            "commit.gpgsign=false",
            "merge",
            "-q",
            "--no-ff",
            "-m",
            "merge side",
            "side",
        ],
    );
}

/// [`merged_with`] with plain `feat: b` and `feat: a` messages.
fn merged(dir: &Path) {
    merged_with(dir, "feat: b", "feat: a");
}

/// A rule that fails every merge, so the skip is visible.
const MERGES_FAIL: &str = "[[commits.rules]]\n\
     name = \"merges-fail\"\n\
     when = 'matches(subject, \"^merge\")'\n\
     assert = 'matches(subject, \"^never\")'\n\
     severity = \"error\"\n";

/// A rule a merge passes only when read against its first parent.
const MERGE_FIRST_PARENT: &str = "[[commits.rules]]\n\
     name = \"merge-leaves-a-alone\"\n\
     when = 'matches(subject, \"^merge\")'\n\
     assert = '!matches(files, \"^src/a\")'\n\
     severity = \"error\"\n";

/// A repository with `rules` and a merge at `HEAD`.
fn merged_repo(rules: &str) -> tempfile::TempDir {
    let dir = commit_repo(rules);
    merged(dir.path());
    dir
}

#[test]
fn a_merge_is_skipped_unless_check_merges_asks_for_it() {
    let dir = merged_repo(MERGES_FAIL);
    let (ok, text) = check_range(dir.path(), "HEAD~1..HEAD");
    assert!(ok, "a merge joins commits checked on their own: {text}");
    assert!(text.contains("1 merge skipped"), "{text}");
    let (ok, text) = check_range_with(dir.path(), "HEAD~1..HEAD", &["--check-merges"]);
    assert!(!ok, "the opt-in must judge the merge: {text}");
    assert!(text.contains("merges-fail"), "{text}");
}

#[test]
fn check_merges_reads_a_merge_against_its_first_parent_only() {
    let dir = commit_repo(MERGE_FIRST_PARENT);
    merged(dir.path());
    let (ok, text) = check_range_with(dir.path(), "HEAD~1..HEAD", &["--check-merges"]);
    assert!(
        ok,
        "against its first parent the merge adds only src/b: {text}"
    );
}

#[test]
fn a_skipped_merge_still_supplies_the_tip_state() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    merged_with(
        dir,
        "feat: side\n\nRefs: parser",
        "feat: main\n\nRefs: parser",
    );
    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(!ok, "the task is still wip at the merge: {text}");
    assert!(
        text.contains(&tip_breach(dir, "HEAD")),
        "the tip rule names the merge, whose tree it read: {text}"
    );
}
