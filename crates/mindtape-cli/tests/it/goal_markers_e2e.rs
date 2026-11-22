//! End-to-end tests for the goal DSL: a task body rates a goal with a
//! `#Helps`/`#Hinders`/`#Neutral` marker, and `check-goal-over` reads the
//! body's markers through the reserved `markers` check input
//! (`truth/tasks/2026-11/define-the-goal-dsl-over-metadata-markers.typ`).
//!
//! Every case reads this repo's real goal files and `truth/lib.typ`, not a
//! fixture. A break in the DSL or a registry fails the suite. The rule is the
//! same one the migration leaf switches the live config to.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;
use mindtape::test_support::write_file;

/// The rule that hands a check function the body markers, pointed at the goal
/// DSL's check. `field = "markers"` is the reserved input, not a task field.
const CONFIG: &str = "\
[tasks]
roots = [\"tasks\"]

[[tasks.rules]]
name = \"rate-every-goal\"
check = { typst = \"truth/lib.typ\", function = \"check-goal-over\", field = \"markers\" }
severity = \"error\"
";

/// A task that rates every goal once: two helps, a hinders with a reason, every
/// other goal neutral, and a marker that is not a goal at all. It passes.
const PASSING: &str = "\
== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[because a newcomer reads the block in 30 seconds].
+ #Helps(adopter, to: guide-the-implementer)[because the implementer knows where to start].
+ #Hinders(maintainer, from: keep-changes-safe)[because a hand-merged marker slips past the gate].
+ #Neutral(run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

#metadata((note: \"not a goal\"))
";

/// A body that rates every goal once, for a case that breaks exactly one
/// verdict. One line per verdict, so a case states which line it changes.
const COMPLETE: &str = "\
== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[because a newcomer reads the block in 30 seconds].
+ #Hinders(maintainer, from: keep-changes-safe)[because a hand-merged marker slips past the gate].
+ #Neutral(guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
";

/// Copy one of this repo's real goal files into the throwaway project. The
/// test then reads the source the backlog reads, not a fixture.
fn copy(dir: &Path, name: &str, source: &str) {
    write_file(dir, format!("truth/{name}"), source);
}

/// A throwaway project carrying this repo's goal files and `truth/lib.typ`,
/// with the goal rule pointed at `check-goal-over`.
fn project() -> tempfile::TempDir {
    let dir = support::scaffold_project(CONFIG);
    let truth = dir.path();
    copy(truth, "lib.typ", include_str!("../../../../truth/lib.typ"));
    copy(
        truth,
        "goal-dsl.typ",
        include_str!("../../../../truth/goal-dsl.typ"),
    );
    copy(
        truth,
        "goals.typ",
        include_str!("../../../../truth/goals.typ"),
    );
    copy(
        truth,
        "process-goals.typ",
        include_str!("../../../../truth/process-goals.typ"),
    );
    copy(
        truth,
        "goal-calls.typ",
        include_str!("../../../../truth/goal-calls.typ"),
    );
    dir
}

/// The task source a case files: the prelude, the goal DSL, a `wip` task, then
/// the body under test.
fn task_source(body: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.4.0\": *\n\
         #import \"/truth/goal-calls.typ\": *\n\n\
         #show: task.with(\n  title: \"Subject\",\n  status: wip(2026, 6, 1),\n)\n\n\
         {body}\n"
    )
}

/// Write `body` as the one task in `dir` and run `mt check` over it, returning
/// whether it passed and everything the run printed.
fn check(dir: &Path, body: &str) -> (bool, String) {
    write_file(dir, "tasks/subject.typ", task_source(body));
    support::run_text(dir, &["check"])
}

/// A `Helps` with no `to:` or a `Hinders` with no `from:` is a Typst error
/// rather than a marker with no goal. The verdict has no goal to rate. The
/// load refuses the file before any check runs.
#[test]
fn a_verdict_without_a_goal_fails_the_load() {
    let dir = project();
    let cases = [
        (
            "+ #Helps(adopter)[because a newcomer reads the block].",
            "needs `to: <goal>`",
        ),
        (
            "+ #Hinders(maintainer)[because a hand merge slips past].",
            "needs `from: <goal>`",
        ),
    ];
    for (line, reason) in cases {
        let (ok, text) = check(dir.path(), line);
        assert!(!ok, "a missing goal must fail the load: {text}");
        assert!(text.contains(reason), "{text}");
    }
}

/// Every breach the check reports, one broken verdict at a time. Each case
/// starts from [`COMPLETE`] so exactly one problem is in the body.
#[test]
fn a_broken_verdict_is_a_check_breach() {
    let dir = project();
    let cases = [
        (
            "wrong audience",
            COMPLETE.replace(
                "Hinders(maintainer, from: keep-changes-safe)",
                "Hinders(adopter, from: keep-changes-safe)",
            ),
            "`keep-changes-safe` is maintainer work, rated adopter",
        ),
        (
            "duplicate",
            format!("{COMPLETE}\n+ #Helps(adopter, to: read-tasks-at-a-glance)[again].\n"),
            "`read-tasks-at-a-glance` is rated twice",
        ),
        (
            "missing",
            COMPLETE.replace(", keep-plain-files", ""),
            "`keep-plain-files` is not rated",
        ),
        (
            "unknown",
            format!(
                "{COMPLETE}\n+ #metadata((marker: \"goal\", audience: \"adopter\", \
                 goal: \"keep-some-files\", effect: \"neutral\", reason: none)).\n"
            ),
            "unknown goal `keep-some-files`",
        ),
        (
            "empty reason",
            COMPLETE.replace("[because a newcomer reads the block in 30 seconds]", "[]"),
            "`read-tasks-at-a-glance` says helps but not why",
        ),
        (
            "no helps",
            COMPLETE.replace(
                "#Helps(adopter, to: read-tasks-at-a-glance)\
                 [because a newcomer reads the block in 30 seconds]",
                "#Neutral(read-tasks-at-a-glance)",
            ),
            "helps no goal",
        ),
    ];
    for (case, body, fragment) in cases {
        let (ok, text) = check(dir.path(), &body);
        assert!(!ok, "{case} must breach: {text}");
        assert!(
            text.contains(fragment),
            "{case}: expected {fragment:?}: {text}"
        );
    }
}

/// A task that rates every goal exactly once, helps twice, hinders with a
/// reason and carries a marker that is not a goal passes.
#[test]
fn a_complete_block_passes() {
    let dir = project();
    let (ok, text) = check(dir.path(), PASSING);
    assert!(ok, "a complete block must pass: {text}");
}

/// A body with no goal marker reaches the check as an empty array, not `none`.
/// The check refuses it because the task helps no goal. A body with nothing at
/// all behaves the same.
#[test]
fn a_body_with_no_goal_marker_breaches() {
    let dir = project();
    let cases = [
        ("marker-free body", "== Summary\n\n- nothing to rate.\n"),
        ("dropped body", ""),
    ];
    for (case, body) in cases {
        let (ok, text) = check(dir.path(), body);
        assert!(!ok, "{case} must breach: {text}");
        assert!(
            text.contains("`read-tasks-at-a-glance` is not rated"),
            "{case}: the rule must read an empty marker array: {text}"
        );
        assert!(text.contains("helps no goal"), "{case}: {text}");
    }
}
