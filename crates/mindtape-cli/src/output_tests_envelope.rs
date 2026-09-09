#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

//! Envelope-side `output` tests: the document-level wrapper `mt ls --json`
//! puts around its records — `schema_version`, `applied`, `truncated` — and
//! the bare array every caller without that context (`mt eval --json`) keeps.
//! Split out of `output_tests.rs` to keep both files under the size gate;
//! sibling of `output_tests.rs`, which covers the record and non-JSON formats.
//!
//! The task fixture here is deliberately bare: nothing below asserts on a
//! field, only on where the envelope puts the records and what it says
//! alongside them.

use crate::render::TestCtx;
use mindtape::test_support::bare_task;

use super::{Applied, Envelope, OutputFormat, Truncation, render};

/// Render one bare task as `--format json` under `envelope`, sharing the
/// render boilerplate across the tests below (jscpd flags it as a clone once
/// duplicated three times over).
fn json_out(envelope: Option<Envelope<'_>>) -> String {
    render(
        &[bare_task("t1")],
        OutputFormat::Json,
        TestCtx::default().ctx(),
        envelope,
    )
    .unwrap()
}

/// [`json_out`], parsed back into a [`serde_json::Value`].
fn json_value(envelope: Option<Envelope<'_>>) -> serde_json::Value {
    serde_json::from_str(&json_out(envelope)).unwrap()
}

/// The envelope an untruncated `mt ls --json` carries: attribution only.
fn envelope(applied: &Applied) -> Envelope<'_> {
    Envelope {
        applied,
        truncated: None,
    }
}

#[test]
fn json_with_applied_wraps_tasks_in_envelope() {
    // `mt ls --json` (an `applied` present) is the breaking envelope
    // shape (`truth/tasks/conditional-ls-rules.typ`); `mt eval --json`
    // (no `applied`) stays a bare array, as covered below.
    let applied = Applied {
        rule: Some("release".to_owned()),
        filter_from: "inbox".to_owned(),
        filter_value: "priority > 5".to_owned(),
        sort_from: "release".to_owned(),
        sort_value: vec!["priority:desc".to_owned()],
        columns_from: "inbox".to_owned(),
        columns_value: vec!["status".to_owned(), "title".to_owned()],
    };
    let value = json_value(Some(envelope(&applied)));
    assert_eq!(value["tasks"][0]["id"], "t1");
    assert_eq!(value["applied"]["rule"], "release");
    assert_eq!(value["applied"]["facets"]["filter"]["from"], "inbox");
    assert_eq!(
        value["applied"]["facets"]["filter"]["value"],
        "priority > 5"
    );
    assert_eq!(value["applied"]["facets"]["sort"]["from"], "release");
    assert_eq!(
        value["applied"]["facets"]["sort"]["value"][0],
        "priority:desc"
    );
    assert_eq!(value["applied"]["facets"]["columns"]["from"], "inbox");
}

#[test]
fn json_with_applied_carries_schema_version() {
    // The envelope is document-level, so `schema_version` sits alongside
    // `tasks`/`applied`, not on each task record
    // (`truth/tasks/emit-schema-version-in-the-ls-json-envelope.typ`).
    let value = json_value(Some(envelope(&Applied::default())));
    assert_eq!(value["schema_version"], mindtape::SCHEMA_VERSION);
    assert!(value["tasks"][0]["schema_version"].is_null());
}

#[test]
fn json_without_applied_stays_a_bare_array_with_no_schema_version() {
    let out = json_out(None);
    assert!(!out.contains("schema_version"));
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(value.is_array());
}

#[test]
fn json_with_no_rule_matched_reports_null_rule() {
    let applied = Applied {
        rule: None,
        filter_from: "match-all".to_owned(),
        ..Applied::default()
    };
    let value = json_value(Some(envelope(&applied)));
    assert!(value["applied"]["rule"].is_null());
    assert_eq!(value["applied"]["facets"]["filter"]["from"], "match-all");
}

// --- truncation (`truth/tasks/ls-limit-flag.typ`) --------------------------

#[test]
fn json_reports_a_truncated_listing_in_the_envelope() {
    // `--json` is structured, so the truncation goes IN the document rather
    // than after it: an agent that asked for a bounded listing reads whether
    // it saw everything from the same parse, not from a footer it would have
    // to strip back off.
    let applied = Applied::default();
    let value = json_value(Some(Envelope {
        applied: &applied,
        truncated: Truncation::new(Some(10), 47),
    }));
    assert_eq!(value["truncated"]["shown"], 10);
    assert_eq!(value["truncated"]["total"], 47);
}

#[test]
fn json_reports_a_complete_listing_as_a_null_truncation() {
    // Present-and-null, never absent: a consumer reads one key rather than
    // branching on whether the key exists.
    let value = json_value(Some(envelope(&Applied::default())));
    assert!(value["truncated"].is_null());
    assert!(
        value.get("truncated").is_some(),
        "the key is always emitted, so `.truncated` never reads as missing"
    );
}

#[test]
fn json_without_an_envelope_carries_no_truncation_key() {
    // `mt eval --json` has no limit and no envelope; the bare array must not
    // grow a document-level field.
    assert!(!json_out(None).contains("truncated"));
}

#[test]
fn truncation_is_absent_without_a_limit() {
    assert_eq!(Truncation::new(None, 40), None);
}

#[test]
fn a_limit_at_or_above_the_match_count_truncates_nothing() {
    // The boundary is the one worth pinning: `-n 40` over 40 matches shows
    // every row, so claiming a truncation would be a lie the reader acts on.
    assert_eq!(Truncation::new(Some(40), 40), None);
    assert_eq!(Truncation::new(Some(41), 40), None);
}

#[test]
fn a_biting_limit_reports_shown_against_the_match_count() {
    let cut = Truncation::new(Some(10), 47).unwrap();
    assert_eq!((cut.shown, cut.total), (10, 47));
    assert_eq!(cut.footer(), "truncated: showing 10 of 47");
}
