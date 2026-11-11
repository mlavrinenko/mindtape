//! `report_injected`'s no-op paths: a filled source it cannot parse, and no
//! config-spliced fields to report.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;

use mindtape::RequiredSlot;

use super::report_injected;

fn injected() -> Vec<RequiredSlot> {
    vec![RequiredSlot {
        field: "kind".to_owned(),
        config_path: "[tasks.fields.kind]".to_owned(),
        call: "slot(\"kind\")".to_owned(),
        env: None,
        default: None,
    }]
}

#[test]
fn a_filled_source_without_a_task_call_reports_nothing() {
    report_injected(
        "not a task",
        &injected(),
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
}

#[test]
fn no_injected_fields_is_a_no_op() {
    report_injected(
        mindtape::typst_task!(r#"(title: "Do thing")"#),
        &[],
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
}
