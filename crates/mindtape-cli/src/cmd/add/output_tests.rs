#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use serde_json::json;

use super::{Format, Handle, Outcome, render};

fn handle() -> Handle {
    Handle {
        hash: "ksr".to_owned(),
        shown: "tasks/foo.typ".to_owned(),
    }
}

fn path() -> PathBuf {
    PathBuf::from("tasks/foo.typ")
}

#[test]
fn text_format_wrote() {
    assert_eq!(
        render(&Outcome::Wrote(path()), &handle(), Format::Text),
        "wrote tasks/foo.typ (ksr)"
    );
}

#[test]
fn text_format_skipped() {
    assert_eq!(
        render(&Outcome::Skipped(path()), &handle(), Format::Text),
        "skipped, already exists at tasks/foo.typ (ksr)"
    );
}

#[test]
fn text_format_suffixed() {
    assert_eq!(
        render(&Outcome::Suffixed(path()), &handle(), Format::Text),
        "suffixed, wrote tasks/foo.typ (ksr)"
    );
}

#[test]
fn path_format_is_the_bare_path_for_every_outcome() {
    for outcome in [
        Outcome::Wrote(path()),
        Outcome::Skipped(path()),
        Outcome::Suffixed(path()),
    ] {
        assert_eq!(render(&outcome, &handle(), Format::Path), "tasks/foo.typ");
    }
}

#[test]
fn path0_format_is_nul_terminated_not_newline_terminated() {
    let out = render(&Outcome::Wrote(path()), &handle(), Format::Path0);
    assert_eq!(out, "tasks/foo.typ\0");
    assert!(!out.ends_with('\n'));
}

#[test]
fn json_format_reports_outcome_and_path_for_each_kind() {
    let cases = [
        (Outcome::Wrote(path()), "wrote"),
        (Outcome::Skipped(path()), "skipped"),
        (Outcome::Suffixed(path()), "suffixed"),
    ];
    for (outcome, expected_label) in cases {
        let rendered = render(&outcome, &handle(), Format::Json);
        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(
            value.get("outcome"),
            Some(&json!(expected_label)),
            "{rendered}"
        );
        assert_eq!(
            value.get("path"),
            Some(&json!("tasks/foo.typ")),
            "{rendered}"
        );
    }
}

#[test]
fn outcome_label_matches_json_outcome_field() {
    assert_eq!(Outcome::Wrote(path()).label(), "wrote");
    assert_eq!(Outcome::Skipped(path()).label(), "skipped");
    assert_eq!(Outcome::Suffixed(path()).label(), "suffixed");
}

#[test]
fn outcome_path_is_exposed_for_every_kind() {
    assert_eq!(Outcome::Wrote(path()).path(), path());
    assert_eq!(Outcome::Skipped(path()).path(), path());
    assert_eq!(Outcome::Suffixed(path()).path(), path());
}
