//! The body-rule half of the `mt check` tests: `[[tasks.body.headers.rules]]`
//! and `[tasks.body.content.rules]` applied to a real project.
//!
//! Split out of `cmd/check/tests.rs` to stay under the line budget. Shared
//! fixtures come from `super::tests`.
#![allow(clippy::unwrap_used)]

use mindtape::parse_config;
use mindtape::test_support::TASK_SRC as TASK;

use super::tests::make_project_with_config;
use crate::test_util::metadata_task_with;

/// `TASK` with a `Summary` section holding one line of prose.
fn task_with_summary() -> String {
    metadata_task_with("\n= Summary\n\nSome content here.\n")
}

/// `TASK` with an empty `Summary`, a filled `Scope`, and a `Watch for` whose
/// prose states a fact instead of asking.
fn task_with_empty_summary() -> String {
    metadata_task_with(
        "\n= Summary\n\n= Scope\n\nReal scope prose.\n\n= Watch for\n\nNo question asked\n",
    )
}

/// Run `mt check` over one task file `src` under the config `config_toml`.
fn check(config_toml: &str, src: &str) -> crate::diag::Result<()> {
    let config = parse_config(config_toml).unwrap();
    let (_dir, project) = make_project_with_config(&[("a.typ", src)], config);
    super::run_with(&project, &super::CheckArgs::default())
}

/// Assert that `mt check` accepts `src` under `config_toml`.
fn assert_ok(config_toml: &str, src: &str) {
    assert!(check(config_toml, src).is_ok());
}

/// Assert that `mt check` refuses `src` under `config_toml`.
fn assert_err(config_toml: &str, src: &str) {
    assert!(check(config_toml, src).is_err());
}

#[test]
fn header_require_rule_passes_when_heading_present() {
    assert_ok(
        "[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n",
        &task_with_summary(),
    );
}

#[test]
fn header_require_rule_fails_when_heading_absent() {
    // TASK has no headings at all
    assert_err(
        "[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n",
        TASK,
    );
}

// `[tasks.body.content]` section-content rules

#[test]
fn content_non_empty_rule_passes_with_prose() {
    assert_ok(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\n",
        &task_with_summary(),
    );
}

#[test]
fn content_non_empty_rule_fails_on_empty_section() {
    assert_err(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\n",
        &task_with_empty_summary(),
    );
}

#[test]
fn content_words_rule_fails_when_short() {
    assert_err(
        "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { words = { min = 100 } }\n",
        &task_with_empty_summary(),
    );
}

#[test]
fn content_must_match_rule_fails_without_question() {
    assert_err(
        "[[tasks.body.content.rules]]\npath = [\"Watch for\"]\nassert = { must-match = \"\\\\?\" }\n",
        &task_with_empty_summary(),
    );
}

#[test]
fn invalid_content_rule_regex_fails_with_config_error() {
    assert_err(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = { must-match = \"[bad\" }\n",
        TASK,
    );
}
