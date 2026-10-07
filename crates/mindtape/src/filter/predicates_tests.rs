//! Coverage for `has`/`miss` presence, `stale(N)`, and the `contains()`
//! default-haystack fold — the arms that moved into `filter::predicates`
//! (file-size gate). Lives in a child module so it shares `sample()` /
//! `hit()` with the parent without duplicating them.
#![allow(clippy::unwrap_used)]

use super::{hit, sample};
use crate::config::Config;
use crate::filter::Filter;

#[test]
fn estimate_is_presence_only() {
    let task = sample();
    assert!(hit("has(estimate)", &task));
    assert!(!hit("miss(estimate)", &task));
    // The opaque estimate dict has no scalar; comparison is rejected.
    assert!(
        Filter::parse("estimate > 4")
            .unwrap()
            .matches(&task, &Config::default())
            .is_err()
    );
}

#[test]
fn presence_functions() {
    let task = sample();
    assert!(hit("has(due)", &task));
    assert!(hit("miss(start)", &task));
    assert!(!hit("has(start)", &task));
    assert!(hit("has(tags)", &task));
}

#[test]
fn stale_without_reviewed_uses_status_date() {
    let mut task = sample();
    // Use a clearly old status date so the test is not time-sensitive.
    if let Some(ref mut s) = task.status {
        s.date = Some("2020-01-01".to_owned());
    }
    assert!(hit("stale(1)", &task));
}

#[test]
fn stale_with_recent_reviewed_is_not_stale() {
    let mut task = sample();
    task.reviewed = Some(crate::date::today_iso());
    assert!(!hit("stale(1)", &task));
}

#[test]
fn stale_with_old_reviewed_is_stale() {
    let mut task = sample();
    task.reviewed = Some("2020-01-01".to_owned());
    assert!(hit("stale(1)", &task));
}

#[test]
fn stale_bad_arg_errors() {
    let task = sample();
    assert!(
        Filter::parse("stale(not_a_number)")
            .unwrap()
            .matches(&task, &Config::default())
            .is_err()
    );
}

#[test]
fn default_haystack_matches_title_or_id() {
    // Guards against `||` → `&&` in `contains_call`'s default-haystack fold.
    let mut task = sample();
    task.id = "mod-001".to_owned();
    task.title = "Auth Module".to_owned();
    // "auth" in title, not in id — `||` gives true, `&&` gives false.
    assert!(hit("contains(\"auth\")", &task));
}

#[test]
fn has_tags_distinguishes_each_negation() {
    // Guards against deletion of `!` in `present` for tags.
    // The expression is `!task.tags.is_empty() || !task.derived_tags.is_empty()`.
    let mut task = sample();
    // Non-empty tags, empty derived → true (catches first `!` deleted)
    task.tags = vec!["backend".to_owned()];
    task.derived_tags = Vec::new();
    assert!(hit("has(tags)", &task));
    // Empty tags, non-empty derived → true (catches second `!` deleted)
    task.tags = Vec::new();
    task.derived_tags = vec!["frontend".to_owned()];
    assert!(hit("has(tags)", &task));
    // Both empty → false (catches either `!` deleted)
    task.tags = Vec::new();
    task.derived_tags = Vec::new();
    assert!(!hit("has(tags)", &task));
}

#[test]
fn stale_with_reviewed_at_exact_threshold_is_not_stale() {
    // Guards against `<` → `<=` in the reviewed branch of `is_stale`.
    // When reviewed equals days_ago_iso(days), real `<` returns false but
    // mutant `<=` returns true.
    let mut task = sample();
    task.reviewed = Some(crate::date::days_ago_iso(1));
    assert!(!hit("stale(1)", &task));
}

#[test]
fn stale_with_status_at_exact_threshold_is_not_stale() {
    // Guards against `<` → `<=` in the status-date branch of `is_stale`.
    let mut task = sample();
    task.reviewed = None;
    if let Some(ref mut s) = task.status {
        s.date = Some(crate::date::days_ago_iso(1));
    }
    assert!(!hit("stale(1)", &task));
}

/// A name follows Typst's identifier rule: a `-` between name characters is
/// part of it, so an `x-<ns>` slot is nameable, as `has`/`miss` presence.
#[test]
fn a_kebab_case_extension_slot_is_one_name() {
    let mut task = sample();
    assert!(!hit("has(x-goals)", &task));
    assert!(hit("miss(x-goals)", &task));
    task.extras
        .insert("x-goals".to_owned(), typst_harvest::HVal::Int(1));
    assert!(hit("has(x-goals)", &task));
}

fn with_unit(unit: &str, value: f64) -> crate::model::Task {
    let mut task = sample();
    task.estimate.insert(unit.to_owned(), value);
    task
}

#[test]
fn an_estimate_unit_compares_as_a_number() {
    let task = with_unit("src-files", 5.0);
    assert!(hit("estimate.src-files > 3", &task));
    assert!(!hit("estimate.src-files > 5", &task));
    assert!(hit("estimate.hours == 8", &task));
}

/// An absent unit is an absent numeric field: unequal to every literal,
/// ordered against none.
#[test]
fn an_absent_estimate_unit_is_missing() {
    let task = sample();
    assert!(!hit("estimate.src-files > 0", &task));
    assert!(!hit("estimate.src-files <= 0", &task));
    assert!(hit("estimate.src-files != 0", &task));
    assert!(!hit("has(estimate.src-files)", &task));
    assert!(hit("has(estimate.src-files)", &with_unit("src-files", 1.0)));
    assert!(hit("miss(estimate.src-files)", &task));
}

/// A unit outside a declared `[tasks.estimate] units` is an unknown field;
/// with no allow-list, any unit name reads.
#[test]
fn an_undeclared_estimate_unit_is_an_unknown_field() {
    let config = crate::parse_config("[tasks.estimate]\nunits = [\"hours\"]\n").unwrap();
    let task = sample();
    let err = Filter::parse("estimate.days > 1")
        .unwrap()
        .matches(&task, &config)
        .unwrap_err();
    assert!(err.to_string().contains("estimate.days"), "{err}");
    assert!(
        Filter::parse("estimate.hours > 1")
            .unwrap()
            .matches(&task, &config)
            .unwrap()
    );
}

/// A leading `-` is still a negative number, a `-` inside a string literal
/// is still text, and a `-` after a number is not a name.
#[test]
fn a_hyphen_outside_a_name_is_untouched() {
    let mut task = sample();
    assert!(hit("priority > -1", &task));
    assert!(hit("priority>-1", &task));
    task.title = "a-b - c".to_owned();
    assert!(hit("title == \"a-b - c\"", &task));
    assert!(hit("contains(\"b - c\")", &task));
}

/// An unknown kebab-case name is reported as written, not in the stand-in
/// spelling the parser saw.
#[test]
fn an_unknown_kebab_name_is_reported_as_written() {
    let err = Filter::parse("no-such-field == 1")
        .unwrap()
        .matches(&sample(), &Config::default())
        .unwrap_err();
    assert_eq!(err.to_string(), "unknown field in filter: no-such-field");
}
