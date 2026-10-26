//! Resolving a `ref`: the pick from `git ls-remote` lines, and a run against
//! a local bare repository.
#![allow(clippy::unwrap_used)]

use std::ffi::OsStr;

use super::{pick, resolve, resolve_with};
use crate::config::pack_sources::FetchError;
use crate::test_support::{git_as_tester, git_stdout, tempdir, write_file};

const TAG: &str = "1111111111111111111111111111111111111111";
const PEELED: &str = "2222222222222222222222222222222222222222";
const BRANCH: &str = "3333333333333333333333333333333333333333";
const NESTED: &str = "4444444444444444444444444444444444444444";

/// `ls-remote` lines with `v1` as an annotated tag, a branch, and a branch
/// whose name only ends in `v1`.
fn listing() -> String {
    format!(
        "{NESTED}\trefs/heads/x/v1\n{BRANCH}\trefs/heads/v1\n\
         {TAG}\trefs/tags/v1\n{PEELED}\trefs/tags/v1^{{}}\n{BRANCH}\tHEAD\n"
    )
}

#[test]
fn a_ref_resolves_as_git_fetch_reads_it_peeling_an_annotated_tag() {
    let cases = [
        ("v1", PEELED),
        ("refs/tags/v1", PEELED),
        ("tags/v1", PEELED),
        ("refs/heads/v1", BRANCH),
        ("heads/x/v1", NESTED),
        ("HEAD", BRANCH),
    ];
    let picked: Vec<_> = cases
        .iter()
        .map(|(reference, _)| (*reference, pick(&listing(), reference)))
        .collect();
    let expected: Vec<_> = cases
        .iter()
        .map(|(reference, id)| (*reference, Some((*id).to_owned())))
        .collect();
    assert_eq!(picked, expected);
}

#[test]
fn a_lightweight_tag_or_branch_resolves_to_its_own_line() {
    let lines = format!("{BRANCH}\trefs/heads/main\n{TAG}\trefs/tags/v2\n");
    assert_eq!(pick(&lines, "main").as_deref(), Some(BRANCH));
    assert_eq!(pick(&lines, "v2").as_deref(), Some(TAG));
    assert_eq!(pick(&lines, "v3"), None);
    assert_eq!(pick(&lines, "ain"), None);
}

#[test]
fn a_commit_ref_is_its_own_commit_and_runs_no_git() {
    let missing = OsStr::new("mt-no-such-git");
    assert_eq!(resolve_with(missing, "unused", TAG).unwrap(), TAG);
    let error = resolve_with(missing, "unused", "v1").unwrap_err();
    assert!(matches!(error, FetchError::GitNotFound), "{error}");
}

#[test]
fn a_local_repository_answers_with_the_commit_its_tag_names() {
    let dir = tempdir();
    let work = dir.path().join("work");
    write_file(&work, "commits.toml", "a = 1\n");
    git_as_tester(&work, &["init", "--quiet"]);
    git_as_tester(&work, &["add", "."]);
    git_as_tester(&work, &["commit", "--quiet", "-m", "packs"]);
    git_as_tester(&work, &["tag", "-a", "-m", "v1", "v1"]);
    let commit = git_stdout(&work, &["rev-parse", "HEAD"]);
    let url = work.display().to_string();

    assert_eq!(resolve(&url, "v1").unwrap(), commit);
    let error = resolve(&url, "v9").unwrap_err();
    assert_eq!(error.to_string(), "no ref 'v9' in the repository");
    assert!(error.hint().is_some_and(|hint| hint.contains("`ref`")));
}
