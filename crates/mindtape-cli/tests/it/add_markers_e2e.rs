//! End-to-end tests for a body marker a `mt add` files, read back by the same
//! `check` rule on the load path. The candidate door harvests the rendered
//! source once and builds the task from both the `mindtape.task` marker and
//! every other marker the body emits
//! (`truth/tasks/2026-11/carry-body-markers-through-the-candidate-door.typ`).
//! A scripted run, one without a terminal, is the case the door must accept:
//! the rule reads the list the body emits, not an empty one.
//!
//! Its own binary rather than more of `add_e2e.rs`, along the subject seam the
//! other `add_*_e2e.rs` modules follow.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::write_file;

/// A template whose body emits the one marker the project's rule reads.
const MARKED_TEMPLATE: &str = mindtape::typst_task!(
    r#"(title: slot("title", example: "Rate a goal"))

== Goals

+ #metadata((goal: "implementable-tasks", verdict: "helps")) <goals>
"#
);

/// The same template with no body marker, so the candidate's rule has nothing
/// to read.
const BARE_TEMPLATE: &str =
    mindtape::typst_task!("(title: slot(\"title\", example: \"Rate a goal\"))\n");

/// The rule that hands a check function the body markers, and the function
/// itself, which refuses a body with none. `verify-when = "!tty()"` makes
/// every run without a terminal check, which is the scripted case.
const CONFIG: &str = "\
[tasks]
roots = [\"tasks\"]

[add]
verify-when = \"!tty()\"

[[tasks.rules]]
name = \"goal-marker\"
when = \"!terminal\"
check = { typst = \"check.typ\", function = \"check-markers\", field = \"markers\" }
message = \"rate a goal in the body\"
severity = \"error\"
";

const CHECK_MARKERS: &str = "\
#let check-markers(markers) = {
  if markers.len() != 1 { return (\"one marker expected: \" + repr(markers),) }
  if markers.at(0).label != \"goals\" { return (\"label: \" + repr(markers),) }
  ()
}
";

/// The project: [`CONFIG`], `source` as the default template, and the rule's
/// Typst module.
fn project(source: &str) -> tempfile::TempDir {
    let dir = support::project_with_template_and_config(CONFIG, source);
    write_file(dir.path(), "check.typ", CHECK_MARKERS);
    dir
}

/// A scripted `mt add` files a marker-bearing task, and `mt check` reads the
/// same markers off the written file. The add succeeds only if the candidate
/// saw its own body markers. The check passes only if the load sees them too.
#[test]
fn a_scripted_add_files_a_marker_bearing_task_that_check_accepts() {
    let dir = project(MARKED_TEMPLATE);

    let (ok, text) = support::run_text(
        dir.path(),
        &["add", "--name", "rated", "--set", "title=Rated"],
    );
    assert!(
        ok,
        "the candidate's rule must read its body markers: {text}"
    );
    support::run_ok(dir.path(), &["check"]);
    let filed = std::fs::read_to_string(dir.path().join("tasks/rated.typ")).unwrap();
    assert!(
        filed.contains("<goals>"),
        "the body marker must land: {filed}"
    );
}

/// The same scripted run refuses a body without the marker, so the acceptance
/// above is the candidate's rule reading a real list, not a rule that never
/// ran.
#[test]
fn a_scripted_add_refuses_a_body_with_no_marker() {
    let dir = project(BARE_TEMPLATE);

    let (ok, text) = support::run_text(
        dir.path(),
        &[
            "add",
            "-t",
            "task",
            "--name",
            "unrated",
            "--set",
            "title=Unrated",
        ],
    );
    assert!(!ok, "a body with no marker must be refused: {text}");
    assert!(text.contains("rules.goal-marker"), "{text}");
    assert!(
        !dir.path().join("tasks/unrated.typ").exists(),
        "a refused add writes nothing"
    );
}
