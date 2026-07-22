#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{ViewSpec, validate_views};

fn view(filter: &str) -> ViewSpec {
    ViewSpec {
        filter: filter.to_owned(),
        sort: Vec::new(),
        columns: Vec::new(),
        description: None,
    }
}

fn map(pairs: Vec<(&str, ViewSpec)>) -> BTreeMap<String, ViewSpec> {
    pairs
        .into_iter()
        .map(|(name, spec)| (name.to_owned(), spec))
        .collect()
}

#[test]
fn valid_views_pass() {
    let views = map(vec![
        ("broken", view("blocked && stale(30)")),
        (
            "summer",
            ViewSpec {
                filter: "has_tag(\"summer\")".to_owned(),
                sort: vec!["priority:desc".to_owned(), "due:asc".to_owned()],
                columns: vec!["status".to_owned(), "title".to_owned()],
                description: Some("Summer push".to_owned()),
            },
        ),
    ]);
    assert!(validate_views(&views).is_empty());
}

#[test]
fn empty_filter_is_match_all_and_valid() {
    let views = map(vec![("all", view(""))]);
    assert!(validate_views(&views).is_empty());
}

#[test]
fn unknown_column_reported() {
    let views = map(vec![(
        "x",
        ViewSpec {
            columns: vec!["bogus".to_owned()],
            ..view("ready")
        },
    )]);
    let problems = validate_views(&views);
    assert!(
        problems.iter().any(|p| p.contains("unknown column")),
        "{problems:?}"
    );
}

#[test]
fn bad_sort_spec_reported() {
    let views = map(vec![(
        "x",
        ViewSpec {
            sort: vec!["nope:asc".to_owned()],
            ..view("ready")
        },
    )]);
    assert!(!validate_views(&views).is_empty());
}

#[test]
fn bad_filter_reported() {
    // Unbalanced parenthesis is a hard parse error.
    let views = map(vec![("x", view("(status"))]);
    let problems = validate_views(&views);
    assert!(
        problems.iter().any(|p| p.contains("x.filter")),
        "{problems:?}"
    );
}

#[test]
fn unknown_view_reference_reported() {
    let views = map(vec![("x", view("@ghost"))]);
    let problems = validate_views(&views);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("unknown view reference")),
        "{problems:?}"
    );
}

#[test]
fn composition_reference_resolves() {
    let views = map(vec![
        ("a", view("ready")),
        ("b", view("@a && priority > 5")),
    ]);
    assert!(validate_views(&views).is_empty());
}

#[test]
fn self_cycle_reported() {
    let views = map(vec![("a", view("@a"))]);
    let problems = validate_views(&views);
    assert!(problems.iter().any(|p| p.contains("cycle")), "{problems:?}");
}

#[test]
fn mutual_cycle_reported() {
    let views = map(vec![("a", view("@b")), ("b", view("@a"))]);
    let problems = validate_views(&views);
    assert!(problems.iter().any(|p| p.contains("cycle")), "{problems:?}");
}

#[test]
fn cycle_exact_message_distinguishes_position_predicate() {
    // Guards against mutant: replace == with != in dfs .
    // That swap changes which slice of the DFS stack is extracted as the cycle
    // body. For a 2-node cycle a↔b the real code produces "a → b → a" while
    // the mutant produces "b → a" (first non-matching position).
    let views = map(vec![("a", view("@b")), ("b", view("@a"))]);
    let problems = validate_views(&views);
    let msg = problems.iter().find(|p| p.contains("cycle")).unwrap();
    assert_eq!(msg, "views: reference cycle a → b → a");
}

#[test]
fn three_node_cycle_exact_message() {
    // Same mutant guard for longer cycles.
    let views = map(vec![
        ("a", view("@b")),
        ("b", view("@c")),
        ("c", view("@a")),
    ]);
    let problems = validate_views(&views);
    let msg = problems.iter().find(|p| p.contains("cycle")).unwrap();
    assert_eq!(msg, "views: reference cycle a → b → c → a");
}
