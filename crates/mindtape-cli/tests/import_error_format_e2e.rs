//! `mt import` problems in the error format: the file and place first, the
//! value single-quoted, the next step on a `hint:` line.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

use support::run_stderr as run;

fn project() -> tempfile::TempDir {
    support::project_with_tasks(&[("alpha.typ", "Alpha")])
}

#[test]
fn import_of_a_file_that_is_not_json_names_the_place() {
    let dir = project();
    std::fs::write(dir.path().join("bad.json"), "not json").unwrap();

    let (code, stderr) = run(dir.path(), &["import", "bad.json"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(stderr, "error: bad.json:1:2: not JSON: expected ident\n");
}

#[test]
fn import_of_a_missing_file_is_a_failed_read() {
    let (code, stderr) = run(project().path(), &["import", "none.json"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        stderr,
        "error: can't read: 'none.json': no such file or directory\n"
    );
}

#[test]
fn import_with_a_malformed_map_is_a_usage_error() {
    let dir = project();
    std::fs::write(dir.path().join("in.json"), "[]").unwrap();

    let (code, stderr) = run(dir.path(), &["import", "in.json", "--map", "todo"]);

    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        stderr,
        "error: --map expects OLD=NEW: 'todo'\nhint: for example: --map todo=proposed\n"
    );
}
