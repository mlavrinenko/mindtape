//! Tests for `[add]`: the `verify-when` key and its load-time check.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{AddConfig, validate_add};

/// An `[add]` with `verify-when` set to `expr`.
fn with_verify(expr: &str) -> AddConfig {
    AddConfig {
        verify_when: Some(expr.to_owned()),
        ..AddConfig::default()
    }
}

#[test]
fn an_absent_verify_when_is_no_problem() {
    assert!(validate_add(&AddConfig::default()).is_empty());
}

#[test]
fn a_verify_when_that_parses_is_no_problem() {
    assert!(validate_add(&with_verify("!tty()")).is_empty());
    assert!(validate_add(&with_verify("tty() && env(\"CI\") == \"\"")).is_empty());
}

#[test]
fn a_verify_when_that_does_not_parse_names_the_key() {
    let problems = validate_add(&with_verify("tty("));

    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].starts_with("add.verify-when: "),
        "{}",
        problems[0]
    );
}

/// The condition grammar has no fields, so a filter expression carried over
/// from `[[on.add]]` fails at load rather than at the first `mt add`.
#[test]
fn a_task_field_in_verify_when_is_refused() {
    let problems = validate_add(&with_verify("status == \"wip\""));

    assert!(
        problems[0].contains("can't read the task field"),
        "{}",
        problems[0]
    );
}

#[test]
fn the_toml_key_is_kebab_case() {
    let config = crate::config::parse_config("[add]\nverify-when = \"!tty()\"\n").unwrap();

    assert_eq!(config.add.verify_when.as_deref(), Some("!tty()"));
}

#[test]
fn config_validate_reports_a_bad_verify_when() {
    let config = crate::config::parse_config("[add]\nverify-when = \"nope()\"\n").unwrap();

    let err = config.validate().unwrap_err();

    assert!(err.to_string().contains("add.verify-when"), "{err}");
}
