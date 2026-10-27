//! `mt import` problems in the error format: the file and place first, the
//! value single-quoted, the next step on a `hint:` line.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::write_file;

fn project() -> tempfile::TempDir {
    support::project_with_tasks(&[("alpha.typ", "Alpha")])
}

#[test]
fn import_of_a_file_that_is_not_json_names_the_place() {
    let dir = project();
    write_file(dir.path(), "bad.json", "not json");

    support::assert_stderr(
        dir.path(),
        &["import", "bad.json"],
        1,
        "error: bad.json:1:2: not JSON: expected ident\n",
    );
}

#[test]
fn import_of_a_missing_file_is_a_failed_read() {
    support::assert_stderr(
        project().path(),
        &["import", "none.json"],
        1,
        &format!("error: can't read: 'none.json': {}\n", support::NOT_FOUND),
    );
}

#[test]
fn import_with_a_malformed_map_is_a_usage_error() {
    let dir = project();
    write_file(dir.path(), "in.json", "[]");

    support::assert_stderr(
        dir.path(),
        &["import", "in.json", "--map", "todo"],
        2,
        "error: --map expects OLD=NEW: 'todo'\nhint: for example: --map todo=proposed\n",
    );
}
