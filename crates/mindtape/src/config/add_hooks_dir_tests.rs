//! The date macros a static `dir` takes: which names it accepts, the names
//! [`DirTemplate::parse`] reports, and each one's expansion from a given date.
#![allow(clippy::unwrap_used)]

use super::{DirTemplate, date_macro_value, is_date_macro, validate_dir};

/// Assert that `dir` loads as a hook's destination.
fn accepts(dir: &str) {
    if let Err(err) = validate_dir(dir) {
        panic!("expected {dir:?} to load: {err}");
    }
}

/// Assert that `dir` draws a refusal whose error contains `needle`.
fn refuses(dir: &str, needle: &str) {
    match validate_dir(dir) {
        Ok(()) => panic!("expected {dir:?} to be refused"),
        Err(err) => assert!(err.contains(needle), "{err}"),
    }
}

/// A static `dir` takes `[tasks.naming] stem`'s date macros, and only those
/// names: a near miss is still an unknown macro.
#[test]
fn a_dir_accepts_the_date_macros_and_refuses_a_near_miss() {
    for good in [
        "tasks/{date}",
        "tasks/{date:YYYY}-{date:MM}",
        "tasks/{yy}/{mm}/{dd}",
    ] {
        accepts(good);
    }
    for bad in ["datex", "dates:YYYY", "YYYY"] {
        refuses(
            &format!("tasks/{{{bad}}}"),
            &format!("unknown dir macro: {{{bad}}}"),
        );
    }
}

/// `{date:YYYY}` is one macro whose name holds its format, not a `date` macro
/// followed by literal text.
#[test]
fn a_dir_template_reports_each_date_macro_name() {
    let template = DirTemplate::parse("tasks/{date:YYYY}-{date:MM}/{yy}").unwrap();
    assert_eq!(
        template.macros().collect::<Vec<_>>(),
        ["date:YYYY", "date:MM", "yy"]
    );
}

#[test]
fn date_macros_expand_from_the_given_date() {
    let date = (2026, 11, 3);
    for (name, expected) in [
        ("date", "2026-11-03"),
        ("date:YYYY-MM", "2026-11"),
        ("date:DD.MM.YYYY", "03.11.2026"),
        ("yy", "26"),
        ("mm", "11"),
        ("dd", "03"),
    ] {
        assert_eq!(
            date_macro_value(name, date).as_deref(),
            Some(expected),
            "{name}"
        );
    }
    for unknown in ["nope", "datex"] {
        assert_eq!(date_macro_value(unknown, date), None, "{unknown}");
        assert!(!is_date_macro(unknown), "{unknown}");
    }
    assert!(is_date_macro("date:YYYY"));
}
