#![allow(clippy::unwrap_used)]

use super::{GitCache, GitStatus, PathFilter, StateSet};
use crate::test_support::tempdir;
use crate::test_support::write_file;
use crate::test_support::{git_as_tester as git, git_init};

/// `git status --porcelain -z` output, NUL-separated as git writes it.
fn porcelain(records: &[&str]) -> String {
    records.iter().map(|record| format!("{record}\0")).collect()
}

fn paths(status: &GitStatus, spec: &str) -> Vec<String> {
    let set = StateSet::parse(spec).unwrap();
    status.paths(set).map(str::to_owned).collect()
}

// --- state names -----------------------------------------------------------

#[test]
fn a_state_set_parses_each_name_and_their_join() {
    let staged = StateSet::parse("staged").unwrap();
    assert!(staged.staged && !staged.worktree);
    let worktree = StateSet::parse("worktree").unwrap();
    assert!(!worktree.staged && worktree.worktree);
    let both = StateSet::parse("worktree+staged").unwrap();
    assert!(both.staged && both.worktree);
}

#[test]
fn an_unknown_or_empty_state_is_refused_naming_the_known_ones() {
    let err = StateSet::parse("staged+commited").unwrap_err();
    assert!(err.contains("commited"), "{err}");
    assert!(err.contains("staged") && err.contains("worktree"), "{err}");
    assert!(StateSet::parse("").is_err());
    assert!(StateSet::parse("staged+").is_err());
}

// --- porcelain parsing per state -------------------------------------------

#[test]
fn porcelain_splits_each_path_by_the_state_it_changed_in() {
    let text = porcelain(&[
        "M  src/staged.rs",
        " M src/unstaged.rs",
        "MM src/both.rs",
        "?? src/new.rs",
        "A  src/added.rs",
        " D src/gone.rs",
    ]);
    let status = GitStatus::parse(&text);
    assert_eq!(
        paths(&status, "staged"),
        ["src/staged.rs", "src/both.rs", "src/added.rs"]
    );
    assert_eq!(
        paths(&status, "worktree"),
        [
            "src/unstaged.rs",
            "src/both.rs",
            "src/new.rs",
            "src/gone.rs"
        ],
        "untracked counts as worktree"
    );
    assert_eq!(
        paths(&status, "staged+worktree").len(),
        6,
        "a path changed in both states counts once"
    );
}

#[test]
fn a_rename_counts_its_destination_and_skips_the_source_record() {
    // `-z` writes a rename as `R  <new>\0<old>\0`.
    let text = porcelain(&["R  new/name.rs", "old/name.rs", " M other.rs"]);
    let status = GitStatus::parse(&text);
    assert_eq!(paths(&status, "staged"), ["new/name.rs"]);
    assert_eq!(paths(&status, "worktree"), ["other.rs"]);
}

#[test]
fn a_path_git_would_quote_arrives_raw_under_z() {
    let status = GitStatus::parse(&porcelain(&["?? a weird -> name.md"]));
    assert_eq!(paths(&status, "worktree"), ["a weird -> name.md"]);
}

#[test]
fn a_record_too_short_to_carry_a_path_is_dropped() {
    let status = GitStatus::parse(&porcelain(&["??", " M ok.rs"]));
    assert_eq!(paths(&status, "worktree"), ["ok.rs"]);
}

// --- include and `!` globs -------------------------------------------------

#[test]
fn no_glob_admits_every_path() {
    let filter = PathFilter::new(&[]).unwrap();
    assert!(filter.admits("anything/at/all.md"));
}

#[test]
fn an_include_glob_admits_only_what_it_names_across_directories() {
    let filter = PathFilter::new(&["crates/*/src/**".to_owned()]).unwrap();
    assert!(filter.admits("crates/mindtape/src/filter/mod.rs"));
    assert!(filter.admits("crates/mindtape/src/deep/mod.rs"));
    assert!(!filter.admits("CHANGELOG.md"));
}

#[test]
fn a_bang_glob_excludes_even_what_an_include_admits() {
    let filter =
        PathFilter::new(&["crates/*/src/**".to_owned(), "!**/*_tests*.rs".to_owned()]).unwrap();
    assert!(filter.admits("crates/mindtape/src/filter/mod.rs"));
    assert!(!filter.admits("crates/mindtape/src/filter/links_tests.rs"));
}

#[test]
fn only_bang_globs_admit_everything_else() {
    let filter = PathFilter::new(&["!docs/**".to_owned()]).unwrap();
    assert!(filter.admits("src/lib.rs"));
    assert!(!filter.admits("docs/a.md"));
}

#[test]
fn an_invalid_glob_is_refused_naming_it() {
    let err = PathFilter::new(&["![oops".to_owned()]).unwrap_err();
    assert!(err.contains("[oops"), "{err}");
}

#[test]
fn a_rename_source_is_not_counted_beside_its_destination() {
    let filter = PathFilter::new(&["src/**".to_owned()]).unwrap();
    let paths = ["src/new.rs", "src/old.rs", "src/other.rs"];
    assert_eq!(filter.count(paths, &[]), 3);
    assert_eq!(filter.count(paths, &["src/old.rs".to_owned()]), 2);
}

// --- against a real repository ---------------------------------------------

#[test]
fn reading_a_real_repository_separates_staged_from_worktree() {
    let dir = tempdir();
    let root = dir.path();
    git_init(root);
    write_file(root, "staged.rs", "a\n");
    write_file(root, "loose.rs", "b\n");
    git(root, &["add", "staged.rs"]);

    let status = GitStatus::read(root).unwrap();
    assert_eq!(paths(&status, "staged"), ["staged.rs"]);
    assert_eq!(paths(&status, "worktree"), ["loose.rs"]);
}

#[test]
fn outside_a_repository_the_read_fails_naming_gits_reason() {
    let dir = tempdir();
    let err = GitStatus::read(dir.path()).unwrap_err();
    assert!(err.contains("not a git repository"), "{err}");
}

#[test]
fn the_cache_reads_git_once_and_shares_it_across_clones() {
    let dir = tempdir();
    let root = dir.path();
    git_init(root);
    write_file(root, "one.rs", "a\n");
    let mut cache = GitCache::default();
    cache.set_root(root);
    let clone = cache.clone();
    assert_eq!(cache.status().unwrap().entries().len(), 1);
    write_file(root, "two.rs", "b\n");
    assert_eq!(
        clone.status().unwrap().entries().len(),
        1,
        "the second read is the first one's answer, not a second git run"
    );
}
