//! The body-rule half of the `mt check` tests: `[[tasks.body.headers.rules]]`
//! and `[tasks.body.content.rules]` applied to a real project.
//!
//! Split out of `check_tests.rs` to stay under the line budget; shared
//! fixtures come from `super::tests`.
#![allow(clippy::unwrap_used)]

use mindtape::parse_config;
use mindtape::test_support::TASK_SRC as TASK;

use super::tests::make_project_with_config;

const TASK_WITH_SUMMARY: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))

= Summary

Some content here.
";

#[test]
fn header_require_rule_passes_when_heading_present() {
    let dir = tempfile::tempdir().unwrap();
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK_WITH_SUMMARY)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn header_require_rule_fails_when_heading_absent() {
    let dir = tempfile::tempdir().unwrap();
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    // TASK has no headings at all
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

// --- `[tasks.body.content]` section-content rules -------------------------

const TASK_WITH_EMPTY_SUMMARY: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))

= Summary

= Scope

Real scope prose.

= Watch for

No question asked
";

#[test]
fn content_non_empty_rule_passes_with_prose() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\n",
    )
    .unwrap();
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK_WITH_SUMMARY)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}

#[test]
fn content_non_empty_rule_fails_on_empty_section() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\n",
    )
    .unwrap();
    let project =
        make_project_with_config(dir.path(), &[("a.typ", TASK_WITH_EMPTY_SUMMARY)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

#[test]
fn content_min_words_rule_fails_when_short() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { min-words = 100 }\n",
    )
    .unwrap();
    let project =
        make_project_with_config(dir.path(), &[("a.typ", TASK_WITH_EMPTY_SUMMARY)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

#[test]
fn content_must_match_rule_fails_without_question() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.body.content.rules]]\npath = [\"Watch for\"]\nassert = { must-match = \"\\\\?\" }\n",
    )
    .unwrap();
    let project =
        make_project_with_config(dir.path(), &[("a.typ", TASK_WITH_EMPTY_SUMMARY)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}

#[test]
fn invalid_content_rule_regex_fails_with_config_error() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = { must-match = \"[bad\" }\n",
    )
    .unwrap();
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK)], config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_err());
}
