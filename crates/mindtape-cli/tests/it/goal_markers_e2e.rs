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
use std::process::Command;

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

/// A warning rule that echoes the reasons the harvest recorded, so a test can
/// see the marker's value instead of inferring it from a breach.
const INSPECT: &str = "\
#let check-reason(markers) = {
  let reasons = markers
    .filter(marker => type(marker) == dictionary)
    .map(marker => marker.at(\"value\", default: none))
    .filter(value => type(value) == dictionary and value.at(\"marker\", default: none) == \"goal\")
    .map(value => value.at(\"reason\", default: none))
  (\"harvested reasons: \" + repr(reasons),)
}
";

/// The inspect rule, appended after the goal rule in [`CONFIG`].
const INSPECT_RULE: &str = "\
[[tasks.rules]]
name = \"inspect-reason\"
check = { typst = \"truth/inspect.typ\", function = \"check-reason\", field = \"markers\" }
severity = \"warn\"
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

/// The task source a case files: the prelude, the goal DSL, a `wip` task, then
/// the body under test.
fn task_source(body: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.5.0\": *\n\
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
    let dir = support::goal_project(CONFIG);
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
    let dir = support::goal_project(CONFIG);
    let without_guide_neutral = COMPLETE.replace("#Neutral(guide-the-implementer, ", "#Neutral(");
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
            "unknown effect",
            format!(
                "{without_guide_neutral}\n+ #metadata((marker: \"goal\", audience: \"adopter\", \
                 goal: \"guide-the-implementer\", effect: \"sideways\", \
                 reason: \"because the implementer knows where to start\")).\n"
            ),
            "`guide-the-implementer` has unknown effect `sideways`",
        ),
        (
            "missing effect",
            format!(
                "{without_guide_neutral}\n+ #metadata((marker: \"goal\", audience: \"adopter\", \
                 goal: \"guide-the-implementer\", \
                 reason: \"because the implementer knows where to start\")).\n"
            ),
            "`guide-the-implementer` has no effect",
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

/// The wrong-audience message quotes both audiences and leaves the decision
/// to the author, without a copy-paste replacement. A mechanical swap would
/// hide a bad verdict instead of fixing the framing.
#[test]
fn a_wrong_audience_verdict_guides_without_a_mechanical_fix() {
    let dir = support::goal_project(CONFIG);
    let cases = [
        (
            "Hinders",
            COMPLETE.replace(
                "Hinders(maintainer, from: keep-changes-safe)",
                "Hinders(adopter, from: keep-changes-safe)",
            ),
        ),
        (
            "Neutral",
            COMPLETE.replace(
                "+ #Hinders(maintainer, from: keep-changes-safe)[because a hand-merged marker slips past the gate].",
                "+ #metadata((marker: \"goal\", audience: \"adopter\", goal: \"keep-changes-safe\", effect: \"neutral\", reason: none)).",
            ),
        ),
    ];
    for (case, body) in cases {
        let (ok, text) = check(dir.path(), &body);
        assert!(!ok, "{case}: a wrong audience must breach: {text}");
        assert!(
            text.contains("`keep-changes-safe` is maintainer work, rated adopter"),
            "{case}: the message quotes both audiences: {text}"
        );
        assert!(
            text.contains("reconsider the goal, the audience or the task's framing"),
            "{case}: the message asks for a decision: {text}"
        );
        assert!(
            !text.contains("#Helps"),
            "{case}: the message gives no mechanical replacement: {text}"
        );
    }
}

/// The helper renders the reason, and the harvested marker still carries it.
/// A warning rule echoes the marker's reason instead of a breach.
#[test]
fn the_harvested_marker_carries_the_reason() {
    let dir = support::goal_project(CONFIG);
    write_file(dir.path(), "truth/inspect.typ", INSPECT);
    let config = dir.path().join(".mindtape/config.toml");
    let base = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("{base}\n{INSPECT_RULE}")).unwrap();
    write_file(dir.path(), "tasks/subject.typ", task_source(PASSING));
    let (ok, text) = support::run_text(dir.path(), &["check"]);
    assert!(ok, "a complete block passes with a warning: {text}");
    assert!(text.contains("harvested reasons:"), "{text}");
    assert!(
        text.contains("because a newcomer reads the block in 30 seconds"),
        "the marker still carries the reason: {text}"
    );
}

/// The reason is visible in the compiled task, not only in the marker. The
/// typst in this repo's shell lacks the HTML feature. The test queries the
/// labelled reason: the same compile, read as text instead of outlined paths.
#[test]
fn a_goal_reason_renders_in_the_task_card() {
    let dir = support::goal_project(CONFIG);
    let help =
        "+ #Helps(adopter, to: read-tasks-at-a-glance)[a visible goal reason <goal-reason>].\n";
    write_file(dir.path(), "tasks/subject.typ", task_source(help));
    let output = Command::new("typst")
        .current_dir(dir.path())
        .arg("query")
        .arg("--package-path")
        .arg(support::package_root(dir.path()))
        .arg("--root")
        .arg(dir.path())
        .arg("tasks/subject.typ")
        .arg("<goal-reason>")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "typst query failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let queried: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let shown: Vec<&str> = queried
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry.get("text").and_then(serde_json::Value::as_str))
        .collect();
    assert_eq!(
        shown,
        ["a visible goal reason"],
        "the compiled card shows the reason"
    );
}

/// An empty reason is a breach: an empty block, a whitespace-only block, or a
/// single word. Each case starts from [`COMPLETE`] so the reason is the one
/// problem.
#[test]
fn a_thin_reason_is_a_check_breach() {
    let dir = support::goal_project(CONFIG);
    let cases = [
        ("empty", "[]"),
        ("blank", "[ ]"),
        ("whitespace", "[   ]"),
        ("one word", "[why]"),
    ];
    for (case, reason) in cases {
        let (ok, text) = check(
            dir.path(),
            &COMPLETE.replace("[because a newcomer reads the block in 30 seconds]", reason),
        );
        assert!(!ok, "{case} reason must breach: {text}");
        assert!(
            text.contains("`read-tasks-at-a-glance` says helps but not why"),
            "{case}: {text}"
        );
        assert!(
            text.contains("write the reason in a sentence"),
            "{case}: the message says what is missing: {text}"
        );
    }
}

/// The check refuses a duplicate goal name in either registry before the
/// merge. The merged `known` would keep one entry, so a task could pass with
/// one goal unrated. Each case appends to a temp copy of this repo's real
/// registry.
#[test]
fn a_duplicate_goal_name_is_a_check_breach() {
    let dir = support::goal_project(CONFIG);
    let cases = [
        (
            "within the product registry",
            "truth/goals.typ",
            "keep-plain-files",
            "\n#let goals = goals + (\n  keep-plain-files-again: goal(\n    to: \"keep plain files\",\n    from: \"keeping plain files again\",\n  ),\n)\n",
        ),
        (
            "across the two registries",
            "truth/process-goals.typ",
            "read-tasks-at-a-glance",
            "\n#let goals = goals + (\n  read-tasks-at-a-glance: goal(\n    to: \"read tasks at a glance\",\n    from: \"reading tasks at a glance\",\n  ),\n)\n",
        ),
    ];
    for (case, registry, name, duplicate) in cases {
        let path = dir.path().join(registry);
        let original = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("{original}{duplicate}")).unwrap();
        let (ok, text) = check(dir.path(), COMPLETE);
        assert!(!ok, "{case}: a duplicate must breach: {text}");
        assert!(
            text.contains(&format!(
                "`{name}` is declared twice; rename one `to` or split the registries"
            )),
            "{case}: the breach names the duplicate: {text}"
        );
        assert!(
            !text.contains("ok: 1 task"),
            "{case}: no false pass: {text}"
        );
        std::fs::write(&path, original).unwrap();
    }
}

/// A task that rates every goal exactly once, helps twice, hinders with a
/// reason and carries a marker that is not a goal passes.
#[test]
fn a_complete_block_passes() {
    let dir = support::goal_project(CONFIG);
    let (ok, text) = check(dir.path(), PASSING);
    assert!(ok, "a complete block must pass: {text}");
}

/// A body with no goal marker reaches the check as an empty array, not `none`.
/// The check refuses it because the task helps no goal. A body with nothing at
/// all behaves the same.
#[test]
fn a_body_with_no_goal_marker_breaches() {
    let dir = support::goal_project(CONFIG);
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
