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
