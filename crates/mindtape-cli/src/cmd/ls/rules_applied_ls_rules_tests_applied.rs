//! Tests for `Applied`'s human trailer, `is_trivial`, supersession, and
//! `bare_view_ref`'s `@name` parsing. Cascade resolution
//! (`resolve`/`resolve_bare`) lives in `cmd/ls/rules_tests.rs`; shares its
//! `args`/`project`/`project_with_config` fixtures.
#![allow(clippy::unwrap_used)]

use crate::cmd::ls::rules::tests::{args, project, project_with_config, resolve};

use super::{Applied, MATCH_ALL, trailer};

#[test]
fn trailer_groups_by_source_in_first_appearance_order() {
    let applied = Applied {
        rule: Some("release".to_owned()),
        rule_superseded: false,
        filter_from: "inbox".to_owned(),
        filter_value: String::new(),
        sort_from: "release".to_owned(),
        sort_value: vec![],
        columns_from: "inbox".to_owned(),
        columns_value: vec![],
    };
    assert_eq!(
        trailer(&applied).as_deref(),
        Some("applied: inbox (filter, columns) → release (sort)")
    );
}

#[test]
fn trailer_omits_match_all_facets() {
    let applied = Applied {
        rule: Some("release".to_owned()),
        filter_from: "match-all".to_owned(),
        sort_from: "release".to_owned(),
        columns_from: "match-all".to_owned(),
        ..Applied::default()
    };
    assert_eq!(
        trailer(&applied).as_deref(),
        Some("applied: release (sort)")
    );
}

#[test]
fn trailer_is_none_when_trivial() {
    let applied = Applied {
        rule: None,
        filter_from: "match-all".to_owned(),
        sort_from: "match-all".to_owned(),
        columns_from: "match-all".to_owned(),
        ..Applied::default()
    };
    assert!(applied.is_trivial());
    assert_eq!(trailer(&applied).as_deref(), None);
}

#[test]
fn is_trivial_false_when_rule_is_some_but_sort_columns_match_all() {
    // Kills mutant at second && → || in is_trivial.
    // Real:   rule.is_some() && filter_from != match-all && sort_from == match-all && ..
    //         → false
    // Mutant: (rule.is_none() && filter_from==match-all) || (sort_from==match-all && ..)
    //         → true
    let applied = Applied {
        rule: Some("x".to_owned()),
        filter_from: "manual".to_owned(),
        sort_from: MATCH_ALL.to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    assert!(!applied.is_trivial());
}

#[test]
fn is_trivial_false_when_filter_from_not_match_all() {
    // Kills mutant at first && → || in is_trivial.
    // Real:   rule.is_none() && filter_from != match-all && …
    //         → false
    // Mutant: rule.is_none() || (filter_from==match-all && sort_from==match-all && …)
    //         → true
    let applied = Applied {
        rule: None,
        filter_from: "manual".to_owned(),
        sort_from: MATCH_ALL.to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    assert!(!applied.is_trivial());
}

#[test]
fn trailer_names_a_rule_whose_every_facet_was_overridden() {
    // The complaint the split had to answer: a rule that matched and then lost
    // every facet used to read exactly like a rule that never matched.
    let applied = Applied {
        rule: Some("release".to_owned()),
        rule_superseded: true,
        filter_from: "manual".to_owned(),
        sort_from: MATCH_ALL.to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    assert_eq!(
        trailer(&applied).as_deref(),
        Some("applied: manual (filter) · release superseded")
    );
}

#[test]
fn note_supersession_spares_a_rule_that_still_owns_a_facet() {
    let before = [
        "release".to_owned(),
        "release".to_owned(),
        MATCH_ALL.to_owned(),
    ];
    let mut applied = Applied {
        rule: Some("release".to_owned()),
        filter_from: "manual".to_owned(),
        sort_from: "release".to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    applied.note_supersession(&before);
    assert!(!applied.rule_superseded);
}

#[test]
fn note_supersession_spares_a_rule_that_contributed_nothing() {
    // An empty rule was never superseded — nothing of its was taken away, so
    // saying so would name a loss that did not happen.
    let before = [
        MATCH_ALL.to_owned(),
        MATCH_ALL.to_owned(),
        MATCH_ALL.to_owned(),
    ];
    let mut applied = Applied {
        rule: Some("empty".to_owned()),
        filter_from: "manual".to_owned(),
        sort_from: MATCH_ALL.to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    applied.note_supersession(&before);
    assert!(!applied.rule_superseded);
}

#[test]
fn note_supersession_ignores_a_listing_no_rule_matched() {
    let before = [
        "default".to_owned(),
        MATCH_ALL.to_owned(),
        MATCH_ALL.to_owned(),
    ];
    let mut applied = Applied {
        rule: None,
        filter_from: "manual".to_owned(),
        ..Applied::default()
    };
    applied.note_supersession(&before);
    assert!(!applied.rule_superseded);
}

#[test]
fn bare_at_sign_is_manual_filter_not_view_ref() {
    // Kills mutant at && → || in bare_view_ref.
    // "@" has empty name after @, so bare_view_ref returns None in real code.
    // Mutant makes it return Some(""), which then errors looking up view "".
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_config(dir.path(), mindtape::Config::default());
    let (bundle, applied) = resolve(&project, &args(Some("@"))).unwrap();
    assert_eq!(bundle.filter, "@");
    assert_eq!(applied.filter_from, "manual");
}

#[test]
fn view_ref_with_underscore_is_valid() {
    // Kills mutant at second || → && in bare_view_ref.
    // "_" is a valid view-name character. Mutant: c.is_ascii_alphanumeric() || (c=='_' && c=='-')
    // → only alphanumeric passes, so "@foo_bar" is misrecognised as manual.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.foo_bar]
filter = "priority > 5"
"#,
    );
    let (bundle, _applied) = resolve(&project, &args(Some("@foo_bar"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
}

#[test]
fn view_ref_with_hyphen_is_valid() {
    // Kills mutant at == → != in bare_view_ref.
    // "-" is a valid view-name char under ==; under != it is rejected.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.my-view]
filter = "priority > 5"
"#,
    );
    let (bundle, _applied) = resolve(&project, &args(Some("@my-view"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
}
