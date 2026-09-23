//! Status mapping: the initial-legend default, date stamping, an out-of-legend
//! name reported but kept, and the `status_map` rename applied before the
//! legend check. Split out of `import_map_tests.rs` (file-size gate), which
//! keeps the other per-field mapping clusters.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests::{canonical, config};
use super::{EntryKind, MapOptions, map_canonical};
use crate::canonical::CanonicalStatus;

// --- status ----------------------------------------------------------

#[test]
fn missing_status_defaults_to_initial_legend_value() {
    let c = canonical("x");
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    let status = mapped.task.status.expect("status defaulted");
    assert_eq!(status.name, "proposed");
    assert!(status.date.is_some());
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "status" && e.kind == EntryKind::Defaulted)
    );
}

#[test]
fn status_without_date_stamps_today() {
    let mut c = canonical("x");
    c.status = Some(CanonicalStatus {
        name: "proposed".to_owned(),
        date: None,
        note: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    let status = mapped.task.status.expect("status present");
    assert_eq!(status.name, "proposed");
    assert!(status.date.is_some());
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "status.date" && e.kind == EntryKind::Defaulted)
    );
}

#[test]
fn out_of_legend_status_is_reported_not_silently_accepted() {
    let mut c = canonical("x");
    c.status = Some(CanonicalStatus {
        name: "bogus".to_owned(),
        date: Some("2026-01-01".to_owned()),
        note: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(
        mapped.task.status.as_ref().map(|s| s.name.as_str()),
        Some("bogus")
    );
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "status" && e.kind == EntryKind::Violated)
    );
}

#[test]
fn status_map_option_renames_before_legend_check() {
    let mut c = canonical("x");
    c.status = Some(CanonicalStatus {
        name: "todo".to_owned(),
        date: Some("2026-01-01".to_owned()),
        note: None,
    });
    let mut opts = MapOptions::default();
    opts.status_map
        .insert("todo".to_owned(), "proposed".to_owned());
    let mapped = map_canonical(&c, &config(), &opts);
    assert_eq!(
        mapped.task.status.as_ref().map(|s| s.name.as_str()),
        Some("proposed")
    );
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "status" && e.kind == EntryKind::Mapped)
    );
    assert!(!mapped.report.has_violations());
}
