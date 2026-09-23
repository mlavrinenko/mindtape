//! The body-rule half of the candidate seam: a header or content rule
//! `mt check` already enforces is run over the candidate's own source and
//! refuses a candidate that breaks it before anything is written. Split out of
//! `candidate_tests.rs`, which was near the linecop cap, along the
//! core-vs-body-rule seam; the shared `marker`/`setup` helpers stay in the
//! parent test module and are reached through `super::tests`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::tests::{marker, setup};
use super::{BodyRules, Candidate, check_candidate};
use crate::content_gate::compile_content_rules;
use crate::gate::compile_rules;
use crate::loader::load_project;
use crate::parse_config;

/// [`marker`], plus literal body text appended after the metadata call. The
/// header and content gates parse headings straight out of the raw source
/// text rather than the harvested marker, so a candidate exercising them
/// needs an actual `==` heading in its source.
fn marker_with_body(status: &str, body: &str) -> String {
    format!("{}{body}", marker(status))
}

/// The Problem this task exists to fix
/// (`truth/tasks/validate-body-rules-before-mt-add-writes-a-task.typ`): a
/// content rule `mt check` already enforces — this repo's own "every TO-BE
/// bullet opens with 🟢" — is broken by one candidate and satisfied by
/// another, so the candidate seam is proven to constrain bullets that violate
/// the rule rather than the section's mere presence. The refused candidate
/// gets the same field and message `mt check` would report, and writes
/// nothing.
#[test]
fn refuses_a_candidate_whose_to_be_bullet_breaks_the_content_rule() {
    let dir = setup();
    let config = parse_config(
        r#"[[tasks.body.content.rules]]
path = ["TO-BE"]
assert = { must-not-match = "(?m)^\\s*- [^🟢]" }
message = "open every TO-BE bullet with 🟢"
"#,
    )
    .unwrap();
    let content = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let body_rules = BodyRules {
        headers: &[],
        content: &content,
    };
    let rel = Path::new("tasks/build.typ");

    let bad = marker_with_body("wip", "\n== TO-BE\n\n- ships the feature\n");
    let violations = check_candidate(
        dir.path(),
        &config,
        &[],
        Candidate { rel, source: &bad },
        body_rules,
    )
    .unwrap();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].field, "body.content.rules[0]");
    assert_eq!(violations[0].message, "open every TO-BE bullet with 🟢");
    assert!(!dir.path().join(rel).exists());

    let good = marker_with_body("wip", "\n== TO-BE\n\n- 🟢 ships the feature\n");
    let violations = check_candidate(
        dir.path(),
        &config,
        &[],
        Candidate { rel, source: &good },
        body_rules,
    )
    .unwrap();
    assert!(violations.is_empty(), "{violations:?}");
}

/// A header-structure rule `mt check` already enforces (a required section) is
/// broken by a candidate carrying no such section, and the candidate seam
/// refuses it naming the field `mt check` would.
#[test]
fn refuses_a_candidate_missing_a_required_section() {
    let dir = setup();
    let config = parse_config(
        r#"[[tasks.body.headers.rules]]
path = ["Summary"]
action = "require"
message = "every task needs a Summary section"
"#,
    )
    .unwrap();
    let headers = compile_rules(&config.tasks.body.headers.rules).unwrap();
    let body_rules = BodyRules {
        headers: &headers,
        content: &[],
    };
    let source = marker_with_body("wip", "\n== Scope\n\nno summary here.\n");

    let violations = check_candidate(
        dir.path(),
        &config,
        &[],
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: &source,
        },
        body_rules,
    )
    .unwrap();

    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].field, "body.headers.rules[0]");
    assert_eq!(violations[0].message, "every task needs a Summary section");
}

/// A content-rule breach already on disk is the existing task's problem, not
/// this candidate's: body rules run over the candidate's own source only, the
/// same property [`filters_out_violations_of_the_existing_tasks`] pins for
/// [`crate::validate::validate`].
///
/// [`filters_out_violations_of_the_existing_tasks`]: super::tests::filters_out_violations_of_the_existing_tasks
#[test]
fn does_not_leak_a_content_violation_from_an_existing_task() {
    let dir = setup();
    let config = parse_config(
        r#"[[tasks.body.content.rules]]
path = ["Summary"]
assert = "non-empty"
"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("tasks/old.typ"),
        marker_with_body("wip", "\n== Summary\n\n"),
    )
    .unwrap();
    let existing = load_project(dir.path(), &config).tasks;
    assert_eq!(existing.len(), 1);

    let content = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let body_rules = BodyRules {
        headers: &[],
        content: &content,
    };
    let source = marker_with_body("wip", "\n== Summary\n\nfine.\n");

    let violations = check_candidate(
        dir.path(),
        &config,
        &existing,
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: &source,
        },
        body_rules,
    )
    .unwrap();

    assert!(violations.is_empty(), "{violations:?}");
}
