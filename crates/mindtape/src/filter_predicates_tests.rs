//! Coverage for `has`/`miss` presence, `stale(N)`, and the `contains()`
//! default-haystack fold — the arms that moved into `filter_predicates`
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
