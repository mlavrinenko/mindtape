#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{CommitRefs, Read, cache_path, parse_log, resolve};
use crate::config::parse_config;
use crate::test_support::bare_task;

/// One commit of `git log --format=%x1e%B%x1f --name-status -z` output.
fn commit(message: &str, records: &[&str]) -> String {
    let files: String = records.iter().map(|record| format!("{record}\0")).collect();
    format!("\x1e{message}\n\x1f\0\n{files}")
}

#[test]
fn a_trailer_value_indexes_every_path_its_commit_changed_by_stem() {
    let log = [
        commit(
            "feat: a\n\nRefs: truth/tasks/one.typ",
            &["M", "src/a.rs", "A", "src/b.rs"],
        ),
        commit("fix: a\n\nRefs: two\nRefs: one", &["M", "src/a.rs"]),
        commit("chore: no trailer", &["M", "src/c.rs"]),
    ]
    .concat();
    let refs = parse_log(&log, "Refs");
    assert_eq!(refs.touched["src/a.rs"], ["one", "two"]);
    assert_eq!(refs.touched["src/b.rs"], ["one"]);
    assert!(
        !refs.touched.contains_key("src/c.rs"),
        "a commit naming no task indexes nothing"
    );
}

#[test]
fn a_rename_indexes_both_sides_and_records_typ_renames() {
    let log = [
        commit(
            "refactor: move\n\nRefs: one",
            &["R100", "src/old.rs", "src/new.rs"],
        ),
        commit(
            "task: rename",
            &["R098", "tasks/a.typ", "tasks/b.typ", "M", "tasks/c.typ"],
        ),
    ]
    .concat();
    let refs = parse_log(&log, "Refs");
    assert_eq!(refs.touched["src/old.rs"], ["one"]);
    assert_eq!(refs.touched["src/new.rs"], ["one"]);
    assert_eq!(
        refs.renames,
        [("tasks/a.typ".to_owned(), "tasks/b.typ".to_owned())]
    );
}

#[test]
fn an_empty_log_is_an_empty_index() {
    assert_eq!(parse_log("", "Refs"), CommitRefs::default());
}

#[test]
fn a_rename_resolves_only_when_both_sides_are_task_files() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    let touched = [("src/x.rs".to_owned(), vec!["old".to_owned()])]
        .into_iter()
        .collect();
    let read = Read {
        refs: CommitRefs {
            touched,
            renames: vec![("old.typ".to_owned(), "tasks/new.typ".to_owned())],
        },
        root_prefix: String::new(),
    };
    let tasks = [bare_task("new")];
    let resolved = resolve(&read, "src/x.rs", &config, Some(&tasks));
    assert!(
        resolved.stems.is_empty(),
        "a rename with a side outside the task roots resolves nothing: {:?}",
        resolved.stems
    );
    assert_eq!(resolved.retired, ["old"]);
}

#[test]
fn cache_path_needs_both_a_root_and_an_enabled_cache() {
    if crate::loader::cache::disabled() {
        assert!(
            cache_path(std::path::Path::new("/tmp/project")).is_none(),
            "with the cache off, even a rooted project gets no cache path"
        );
    } else {
        assert!(cache_path(std::path::Path::new("/tmp/project")).is_some());
        assert!(
            cache_path(std::path::Path::new("")).is_none(),
            "an empty root has no place to put the cache"
        );
    }
}
