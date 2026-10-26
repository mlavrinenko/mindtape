//! The shipped `mindtape:flow` pack: its transition graph refuses a move the
//! legend does not list, and its `stale-wip` gate refuses to start a task
//! nobody has reviewed in a month until `mt review` stamps it.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;

use crate::support;

/// A project under the flow pack holding one `proposed` task last touched in
/// 2020.
fn project() -> tempfile::TempDir {
    let dir =
        support::scaffold_project("extends = [\"mindtape:flow\"]\n[tasks]\nroots = [\"tasks\"]\n");
    write_file(
        dir.path(),
        "tasks/alpha.typ",
        mindtape::typst_task!("(\n  title: \"Alpha\",\n  status: proposed(2020, 1, 1),\n)\n"),
    );
    dir
}

/// `mt <args>` in `dir` must exit 1 and print `needle`.
fn refused(dir: &std::path::Path, args: &[&str], needle: &str) {
    let (code, stderr) = support::run_stderr(dir, args);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains(needle), "{stderr}");
}

#[test]
fn a_stale_task_cannot_start_until_it_is_reviewed() {
    let dir = project();
    refused(
        dir.path(),
        &["wip", "alpha"],
        "re-read the scope against today's code",
    );
    support::run_ok(dir.path(), &["review", "alpha"]);
    support::run_ok(dir.path(), &["wip", "alpha"]);
}

#[test]
fn a_move_the_legend_does_not_list_is_refused() {
    refused(project().path(), &["done", "alpha"], "proposed");
}

#[test]
fn a_project_legend_with_an_unknown_edge_is_refused() {
    let dir = support::scaffold_project(
        "extends = [\"mindtape:flow\"]\n[tasks]\nroots = [\"tasks\"]\n\
         [[tasks.status]]\nvalue = \"open\"\nto = [\"shipped\"]\n",
    );
    refused(dir.path(), &["check"], "unknown status");
}
