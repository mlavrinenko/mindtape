//! Tests for `[tasks.assignee]` parsing.
#![allow(clippy::unwrap_used)]

use crate::config::parse_config;
use crate::config_required::Requirement;

use super::AssigneeConfig;

#[test]
fn absent_section_is_unconstrained() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    assert_eq!(
        config.tasks.assignee,
        AssigneeConfig {
            allowed: Vec::new(),
            required: Requirement::No,
        }
    );
}

#[test]
fn empty_section_is_unconstrained() {
    let config = parse_config("[tasks.assignee]\n").unwrap();
    assert_eq!(config.tasks.assignee, AssigneeConfig::default());
}

#[test]
fn allowed_and_required_parse() {
    let config =
        parse_config("[tasks.assignee]\nallowed = [\"ada\", \"lin\"]\nrequired = true\n").unwrap();
    assert_eq!(config.tasks.assignee.allowed, vec!["ada", "lin"]);
    assert_eq!(config.tasks.assignee.required, Requirement::Always);
}

#[test]
fn required_filter_expression_parses() {
    let config = parse_config("[tasks.assignee]\nrequired = \"!terminal\"\n").unwrap();
    assert_eq!(
        config.tasks.assignee.required,
        Requirement::Expr("!terminal".to_owned())
    );
}

#[test]
fn unknown_key_is_rejected() {
    let err = parse_config("[tasks.assignee]\ndefault = \"ada\"\n").unwrap_err();
    assert!(err.to_string().contains("default"), "{}", err.to_string());
}
