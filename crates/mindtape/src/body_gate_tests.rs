//! [`check_body_rules`]'s own tests: it runs both gates, in header-then-content
//! order, and neither gate's rule set is required for the other to run —
//! [`gate_tests`](crate::gate_tests) and
//! [`content_gate_tests`](crate::content_gate_tests) pin each gate's own
//! behaviour; this pins only that the combinator calls both.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{BodyRules, check_body_rules};
use crate::config::parse_config;
use crate::content_gate::compile_content_rules;
use crate::gate::compile_rules;
use crate::test_support::bare_task;

#[test]
fn reports_no_violations_with_no_rules_configured() {
    let config = parse_config("").unwrap();
    let task = bare_task("x");

    let violations = check_body_rules(
        &task,
        "== Summary\n\nfine.\n",
        BodyRules::default(),
        &config,
    );

    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn reports_a_header_violation() {
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    let headers = compile_rules(&config.tasks.body.headers.rules).unwrap();
    let rules = BodyRules {
        headers: &headers,
        content: &[],
    };
    let task = bare_task("x");

    let violations = check_body_rules(&task, "== Scope\n\nno summary here.\n", rules, &config);

    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].field, "body.headers.rules[0]");
}

#[test]
fn reports_a_content_violation() {
    let config = parse_config(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\n",
    )
    .unwrap();
    let content = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let rules = BodyRules {
        headers: &[],
        content: &content,
    };
    let task = bare_task("x");

    let violations = check_body_rules(&task, "== Summary\n\n", rules, &config);

    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].field, "body.content.rules[0]");
}

#[test]
fn reports_violations_from_both_gates_together() {
    let config = parse_config(
        "[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n\n\
         [[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = \"non-empty\"\n",
    )
    .unwrap();
    let headers = compile_rules(&config.tasks.body.headers.rules).unwrap();
    let content = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let rules = BodyRules {
        headers: &headers,
        content: &content,
    };
    let task = bare_task("x");

    let violations = check_body_rules(&task, "== Scope\n\n", rules, &config);

    assert_eq!(violations.len(), 2, "{violations:?}");
    assert_eq!(violations[0].field, "body.headers.rules[0]");
    assert_eq!(violations[1].field, "body.content.rules[0]");
}
