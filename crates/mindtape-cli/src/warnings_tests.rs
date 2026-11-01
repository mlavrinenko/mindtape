#![allow(clippy::unwrap_used)]

use mindtape::{LoadOutcome, LoadWarning};

use super::fold_warnings;
use crate::diag::{Kind, plain};

fn warning(file: &str, line: usize, message: &str, hints: &[&str]) -> LoadWarning {
    LoadWarning {
        file: file.into(),
        line: Some(line),
        column: Some(1),
        message: message.to_owned(),
        hints: hints.iter().map(|hint| (*hint).to_owned()).collect(),
    }
}

fn folded(warnings: &[LoadWarning]) -> String {
    plain(fold_warnings(warnings, false).first().unwrap())
}

#[test]
fn a_repeated_warning_folds_to_one_line_with_its_count_and_files() {
    let warnings = vec![
        warning("tasks/a.typ", 6, "no text within stars", &["hint"]),
        warning("tasks/b.typ", 7, "no text within stars", &["hint"]),
        warning("tasks/a.typ", 8, "no text within stars", &["hint"]),
    ];

    let text = folded(&warnings);
    let a = mindtape::display_path("tasks/a.typ");
    let b = mindtape::display_path("tasks/b.typ");
    assert_eq!(
        text,
        format!("warning: no text within stars: 3 times in 2 files: {a}, {b}\nhint: hint\n")
    );
}

#[test]
fn a_repeated_warning_in_one_file_still_names_the_count() {
    let warnings = vec![
        warning("tasks/a.typ", 6, "no text within stars", &["hint"]),
        warning("tasks/a.typ", 8, "no text within stars", &["hint"]),
    ];

    assert_eq!(
        folded(&warnings),
        format!(
            "warning: no text within stars: 2 times in {}\nhint: hint\n",
            mindtape::display_path("tasks/a.typ")
        )
    );
}

#[test]
fn a_repeated_warning_lists_every_site_when_verbose() {
    let warnings = vec![
        warning("tasks/a.typ", 6, "no text within stars", &["hint"]),
        warning("tasks/b.typ", 7, "no text within stars", &["hint"]),
    ];

    let diags = fold_warnings(&warnings, true);

    assert_eq!(diags.len(), 2);
    assert!(
        plain(diags.first().unwrap())
            .starts_with("warning: tasks/a.typ:6:1: no text within stars\n")
    );
    assert!(
        plain(diags.get(1).unwrap())
            .starts_with("warning: tasks/b.typ:7:1: no text within stars\n")
    );
}

#[test]
fn a_single_warning_keeps_its_place_without_a_count() {
    let warnings = vec![warning("tasks/a.typ", 6, "no text within stars", &["hint"])];

    let diags = fold_warnings(&warnings, false);

    assert_eq!(diags.len(), 1);
    assert_eq!(
        plain(diags.first().unwrap()),
        "warning: tasks/a.typ:6:1: no text within stars\nhint: hint\n"
    );
}

#[test]
fn a_repeated_warning_keeps_each_distinct_hint_once() {
    let warnings = vec![
        warning("tasks/a.typ", 6, "same", &["first"]),
        warning("tasks/b.typ", 7, "same", &["second", "first"]),
    ];

    let text = folded(&warnings);

    assert_eq!(text.matches("hint: first\n").count(), 1, "{text}");
    assert_eq!(text.matches("hint: second\n").count(), 1, "{text}");
}

#[test]
fn load_problems_folds_warnings_for_every_reporting_command() {
    let outcome = LoadOutcome {
        warnings: vec![
            warning("tasks/a.typ", 6, "no text within stars", &["hint"]),
            warning("tasks/a.typ", 8, "no text within stars", &["hint"]),
        ],
        ..LoadOutcome::default()
    };

    let diags = crate::util::load_problems(&outcome, Kind::Warning);

    assert_eq!(diags.len(), 1);
    assert!(
        plain(diags.first().unwrap())
            .starts_with("warning: no text within stars: 2 times in tasks/a.typ\n"),
        "{diags:?}"
    );
}
