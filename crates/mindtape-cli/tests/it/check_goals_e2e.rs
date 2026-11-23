//! End-to-end tests for the live goal-block rules: the project's allowlist, the
//! one `== Goals` section after `Dogfood`, and the content rules that keep it
//! written and short. The refusal covers a reopened task with `x-goals`
//! (`truth/tasks/2026-11/migrate-the-open-backlog-and-switch-the-goal-rule.typ`).
//!
//! The marker DSL itself is `goal_markers_e2e`. This file reads this repo's
//! real goal files and points the live rules at them, so a registry rename or a
//! rule that stops reading the body fails here.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;
use mindtape::test_support::write_file;

/// The rules the project declares over an open task's goal block.
const CONFIG: &str = "\
[tasks]
roots = [\"truth/tasks\"]

[[tasks.rules]]
name = \"give-every-goal-verdict\"
when = \"!terminal\"
check = { typst = \"truth/lib.typ\", function = \"check-goal-over\", field = \"markers\" }
severity = \"error\"

[[tasks.body.headers.rules]]
name = \"allow-only-declared-sections\"
action = \"allowlist\"
allowed = [
  { match = \"^(AS-IS|Environment|TO-BE|Source of truth|Changes|Alternatives|Rollback|Out of scope|Dogfood|Goals)$\" },
]
when = \"!terminal\"
message = \"limit an open task to AS-IS, TO-BE, Changes, Out of scope, Dogfood and Goals, plus its kind's own section\"

[[tasks.body.headers.rules]]
name = \"keep-one-goals-section\"
path = [\"Goals\"]
action = \"unique\"
when = \"!terminal\"
message = \"rate every goal under one `== Goals` heading\"

[[tasks.body.headers.rules]]
name = \"put-goals-after-dogfood\"
action = \"order\"
before = [\"Dogfood\"]
after = [\"Goals\"]
when = \"!terminal\"
message = \"move Goals after Dogfood\"

[[tasks.body.content.rules]]
name = \"write-goals-section\"
path = [\"Goals\"]
assert = \"non-empty\"
when = \"!terminal\"
message = \"write the goal block under `Goals`; an empty section rates nothing\"

[[tasks.body.content.rules]]
name = \"cap-goals-at-160-words\"
path = [\"Goals\"]
assert = { words = { max = 160, count = \"text\" } }
when = \"!terminal\"
message = \"cut the goal reasons to under 160 words\"
";

/// A complete block that rates every goal once.
const COMPLETE: &str = "\
== Dogfood

Not applicable.

== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[a reader sees the block].
+ #Neutral(guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
";

/// The old `x-goals` dict a reopened task carries. The live goal check reads
/// the field, so the task must move its goals into `Goals`.
const LEGACY: &str = "\
  x-goals: (
    product: (
      readable-tasks: helps[the task reads as it did when it closed],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
";

/// The task source a case files: generation 2's prelude, a `wip` task, an
/// optional header block, then the body under test.
fn task_source(header: &str, body: &str) -> String {
    format!(
        "#import \"/truth/prelude/2/lib.typ\": *\n\n\
         #show: task.with(\n  title: \"Subject\",\n  status: wip(2026, 6, 1),\n\
         {header})\n\n\
         {body}\n"
    )
}

/// Write `body` as the one task in `dir` and run `mt check`, returning whether
/// it passed and everything the run printed.
fn check(dir: &Path, header: &str, body: &str) -> (bool, String) {
    write_file(
        dir,
        "truth/tasks/2026-11/subject.typ",
        task_source(header, body),
    );
    support::run_text(dir, &["check"])
}

/// The text of a breach a body must raise on `rule`.
fn breach(dir: &Path, header: &str, body: &str, rule: &str) -> String {
    let (ok, text) = check(dir, header, body);
    assert!(!ok, "the body must breach {rule}: {text}");
    assert!(text.contains(rule), "the breach names {rule}: {text}");
    text
}

/// A heading outside the project's allowlist breaches, naming the rule.
#[test]
fn an_undeclared_section_breaches() {
    let dir = support::goal_project(CONFIG);
    let body = format!("== Summary\n\nA section the allowlist lacks.\n\n{COMPLETE}");
    breach(dir.path(), "", &body, "allow-only-declared-sections");
}

/// A second `== Goals` heading breaches the unique rule.
#[test]
fn a_second_goals_section_breaches() {
    let dir = support::goal_project(CONFIG);
    let body = format!("{COMPLETE}\n== Goals\n\nAn extra block.\n");
    breach(dir.path(), "", &body, "keep-one-goals-section");
}

/// Goals before Dogfood breaches the order rule.
#[test]
fn goals_before_dogfood_breach() {
    let dir = support::goal_project(CONFIG);
    let body = "\
== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[a reader sees the block].
+ #Neutral(guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

== Dogfood

Not applicable.
";
    breach(dir.path(), "", body, "put-goals-after-dogfood");
}

/// An empty Goals section breaches the content rule.
#[test]
fn an_empty_goals_section_breaches() {
    let dir = support::goal_project(CONFIG);
    let body = "== Dogfood\n\nNot applicable.\n\n== Goals\n";
    breach(dir.path(), "", body, "write-goals-section");
}

/// A block over the word cap breaches even when it rates every goal.
#[test]
fn a_long_goals_section_breaches() {
    let dir = support::goal_project(CONFIG);
    let long = "word ".repeat(200);
    let body = format!(
        "== Dogfood\n\nNot applicable.\n\n== Goals\n\n\
         + #Helps(adopter, to: read-tasks-at-a-glance)[{long}].\n\
         + #Neutral(guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).\n"
    );
    breach(dir.path(), "", &body, "cap-goals-at-160-words");
}

/// The goal rules refuse a reopened task that still carries `x-goals` and no
/// `Goals`, and the run names them rather than a warning.
#[test]
fn a_reopened_legacy_task_is_refused() {
    let dir = support::goal_project(CONFIG);
    let body = "== Dogfood\n\nNot applicable.\n";
    let (ok, text) = check(dir.path(), LEGACY, body);
    assert!(!ok, "a reopened task with x-goals must fail: {text}");
    assert!(
        text.contains("give-every-goal-verdict"),
        "the refusal names the goal rule: {text}"
    );
    assert!(
        text.contains("keep-one-goals-section"),
        "the refusal names the missing Goals section: {text}"
    );
    assert!(
        !text.contains("warn-on-x-goals-on-open-tasks"),
        "there is no warning rule: {text}"
    );
}
