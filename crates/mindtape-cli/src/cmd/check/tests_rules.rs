//! The body-rule half of the `mt check` tests: `[[tasks.body.headers.rules]]`
//! and `[tasks.body.content.rules]` applied to a real project.
//!
//! Split out of `cmd/check/tests.rs` to stay under the line budget; shared
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

/// Run `mt check` over one task file `src` under the config `config_toml`.
fn check(config_toml: &str, src: &str) -> crate::diag::Result<()> {
    let config = parse_config(config_toml).unwrap();
    let (_dir, project) = make_project_with_config(&[("a.typ", src)], config);
    super::run_with(&project, &super::CheckArgs::default())
}

#[test]
fn header_require_rule_passes_when_heading_present() {
    assert!(
        check(
            "[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n",
            TASK_WITH_SUMMARY
        )
        .is_ok()
    );
}

#[test]
fn header_require_rule_fails_when_heading_absent() {
    // TASK has no headings at all
    assert!(
        check(
            "[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n",
            TASK
        )
        .is_err()
    );
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
    assert!(
        check(
            "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\n",
            TASK_WITH_SUMMARY
        )
        .is_ok()
    );
}

#[test]
fn content_non_empty_rule_fails_on_empty_section() {
    assert!(
        check(
            "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\n",
            TASK_WITH_EMPTY_SUMMARY
        )
        .is_err()
    );
}

#[test]
fn content_min_words_rule_fails_when_short() {
    assert!(
        check(
            "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { min-words = 100 }\n",
            TASK_WITH_EMPTY_SUMMARY
        )
        .is_err()
    );
}

#[test]
fn content_must_match_rule_fails_without_question() {
    assert!(check("[[tasks.body.content.rules]]\npath = [\"Watch for\"]\nassert = { must-match = \"\\\\?\" }\n", TASK_WITH_EMPTY_SUMMARY).is_err());
}

#[test]
fn invalid_content_rule_regex_fails_with_config_error() {
    assert!(check("[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = { must-match = \"[bad\" }\n", TASK).is_err());
}
