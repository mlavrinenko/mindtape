//! Tests for `Applied`'s footer, supersession, and `bare_view_ref`'s `@name`
//! parsing. Cascade resolution (`resolve`/`resolve_bare`) lives in
//! `cmd/ls/rules_tests.rs`; shares its `args`/`project`/`project_with_config`
//! fixtures.
#![allow(clippy::unwrap_used)]

use crate::cmd::ls::rules::tests::{args, project, project_with_config, resolve};

use super::{Applied, MATCH_ALL, footer};

/// An `Applied` whose facets came from `filter`, `sort` and `columns`, with
/// `rule` matched.
fn applied(rule: Option<&str>, filter: &str, sort: &str, columns: &str) -> Applied {
    Applied {
        rule: rule.map(str::to_owned),
        filter_from: filter.to_owned(),
        sort_from: sort.to_owned(),
        columns_from: columns.to_owned(),
        ..Applied::default()
    }
}

#[test]
fn footer_names_the_view_and_counts_the_project() {
    let from_view = applied(None, "default", "default", "default");
    assert_eq!(footer(&from_view, 3, 12), "default view · 3 of 12 tasks");
    assert_eq!(footer(&from_view, 1, 1), "default view · 1 of 1 task");
    // A rule that set the filter is named once, not again for its sort.
    let from_rule = applied(Some("release"), "release", "release", "release");
    assert_eq!(footer(&from_rule, 0, 2), "release rule · 0 of 2 tasks");
}

#[test]
fn footer_names_a_sort_rule_after_the_view() {
    let shared = applied(Some("release"), "inbox", "release", "inbox");
    assert_eq!(
        footer(&shared, 3, 12),
        "inbox view · release sort · 3 of 12 tasks"
    );
    let sort_only = applied(Some("release"), MATCH_ALL, "release", MATCH_ALL);
    assert_eq!(footer(&sort_only, 0, 2), "release sort · 0 of 2 tasks");
}

#[test]
fn footer_says_your_filter_whatever_else_the_user_typed() {
    let typed = |sort: &str, columns: &str| footer(&applied(None, "manual", sort, columns), 2, 2);
    assert_eq!(typed(MATCH_ALL, MATCH_ALL), "your filter · 2 of 2 tasks");
    assert_eq!(typed("manual", MATCH_ALL), "your filter · 2 of 2 tasks");
    assert_eq!(typed("manual", "manual"), "your filter · 2 of 2 tasks");
}

#[test]
fn footer_leaves_out_a_view_that_only_sorts_or_picks_columns() {
    let narrowed = applied(None, "manual", "default", "default");
    assert_eq!(footer(&narrowed, 2, 5), "your filter · 2 of 5 tasks");
}

#[test]
fn footer_names_a_rule_that_only_picks_columns() {
    let columns_only = applied(Some("wide"), MATCH_ALL, MATCH_ALL, "wide");
    assert_eq!(footer(&columns_only, 2, 2), "wide rule · 2 of 2 tasks");
}

#[test]
fn footer_is_the_count_alone_when_nothing_shaped_the_listing() {
    assert_eq!(footer(&Applied::match_all(), 2, 2), "2 of 2 tasks");
}

#[test]
fn footer_leaves_an_overridden_rule_to_the_json() {
    let applied = Applied {
        rule_superseded: true,
        ..applied(Some("release"), "manual", MATCH_ALL, MATCH_ALL)
    };
    assert_eq!(footer(&applied, 1, 4), "your filter · 1 of 4 tasks");
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
    let (_dir, project) = project_with_config(mindtape::Config::default());
    let (bundle, applied) = resolve(&project, &args(Some("@"))).unwrap();
    assert_eq!(bundle.filter, "@");
    assert_eq!(applied.filter_from, "manual");
}

#[test]
fn view_ref_with_underscore_is_valid() {
    // Kills mutant at second || → && in bare_view_ref.
    // "_" is a valid view-name character. Mutant: c.is_ascii_alphanumeric() || (c=='_' && c=='-')
    // → only alphanumeric passes, so "@foo_bar" is misrecognised as manual.
    let (_dir, project) = project(
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
    let (_dir, project) = project(
        r#"
[views.my-view]
filter = "priority > 5"
"#,
    );
    let (bundle, _applied) = resolve(&project, &args(Some("@my-view"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
}
