//! [`Filter::reads_path`] and the shared [`path_dependent_fields`] list: the
//! fields only a task file's location can answer. Split out of `tests.rs` for
//! the line cap.

#![allow(clippy::unwrap_used)]

use crate::filter::{Filter, path_dependent_fields};

#[test]
fn path_dependent_fields_names_path_and_dir() {
    assert_eq!(path_dependent_fields(), ["path", "dir"]);
}

#[test]
fn reads_path_sees_a_filter_that_names_path_or_dir() {
    for expr in [
        "path == \"tasks/fixes/a.typ\"",
        "dir == \"tasks/fixes\"",
        "kind == \"fix\" && matches(path, \"^tasks/fixes/\")",
    ] {
        assert!(Filter::parse(expr).unwrap().reads_path(), "{expr}");
    }
}

#[test]
fn reads_path_ignores_a_filter_about_the_task_s_own_fields() {
    for expr in ["kind == \"fix\"", "title == \"Fix\"", ""] {
        assert!(!Filter::parse(expr).unwrap().reads_path(), "{expr}");
    }
}

#[test]
fn reads_path_in_follows_a_view_that_reads_dir() {
    let config = crate::parse_config("[views.fixes]\nfilter = 'dir == \"tasks/fixes\"'\n").unwrap();
    let filter = Filter::parse("@fixes").unwrap();
    assert!(!filter.reads_path());
    assert!(filter.reads_path_in(&config));
}

#[test]
fn reads_path_in_survives_a_view_cycle() {
    let config =
        crate::parse_config("[views.a]\nfilter = '@b'\n[views.b]\nfilter = '@a'\n").unwrap();
    assert!(!Filter::parse("@a").unwrap().reads_path_in(&config));
}
