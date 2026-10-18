//! Coverage for the error and fall-through arms of the `function` dispatcher
//! and its helpers — the paths a happy-path filter never walks.
//!
//! `function` is a flat match over filter-function names, so its cyclomatic
//! complexity is dispatch breadth. Leaving the rare arms unexercised is what
//! pushed it over the CRAP threshold (`just crap::check`); these tests walk them.

#![allow(clippy::unwrap_used)]

use super::{hit, sample};
use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::parse_config;

fn err(expr: &str, task: &Task) -> FilterError {
    Filter::parse(expr)
        .unwrap()
        .matches(task, &Config::default())
        .unwrap_err()
}

#[test]
fn miss_propagates_an_unknown_field_error() {
    // `miss(...)` negates `present`, so its error path is separate from `has`.
    let got = err("miss(bogus)", &sample());
    assert!(matches!(got, FilterError::UnknownField(_)), "{got:?}");
}

#[test]
fn default_haystack_matches_the_id_when_the_title_does_not() {
    // Complements `default_haystack_matches_title_or_id` in the parent module
    // (which hits via `title`): the fold must also reach `id`, not stop after
    // the first field misses.
    let mut task = sample();
    task.id = "mod-001".to_owned();
    task.title = "Nothing To See".to_owned();
    assert!(hit("contains(\"mod-0\")", &task));
    assert!(!hit("contains(\"absent\")", &task));
}

#[test]
fn stale_with_a_non_integer_literal_errors() {
    // A quoted non-number reaches the parse, unlike the bare identifier that
    // `single_literal` rejects first.
    let got = err("stale(\"soon\")", &sample());
    assert!(
        matches!(&got, FilterError::Call { problem, .. } if problem.contains("whole number")),
        "{got:?}"
    );
}

#[test]
fn unknown_function_errors() {
    let got = err("bogus()", &sample());
    assert!(
        matches!(&got, FilterError::UnknownFunction { name, .. } if name == "bogus"),
        "{got:?}"
    );
}

#[test]
fn numeric_field_falls_back_to_string_compare() {
    // A non-numeric literal cannot be parsed as a number, so the numeric field
    // is formatted and compared as text ("8" sorts before "eight").
    let task = sample(); // priority 8
    assert!(!hit("priority == \"eight\"", &task));
    assert!(hit("priority != \"eight\"", &task));
    assert!(hit("priority < \"eight\"", &task));
}

#[test]
fn in_view_with_an_empty_filter_matches_everything() {
    let cfg = parse_config("[views.all]\nfilter = ''\n").unwrap();
    let task = sample();
    assert!(Filter::parse("@all").unwrap().matches(&task, &cfg).unwrap());
}

#[test]
fn task_without_reviewed_or_status_date_is_stale() {
    // No date to measure against at all — treated as stale.
    let mut task = sample();
    task.reviewed = None;
    task.status = None;
    assert!(hit("stale(30)", &task));
}

#[test]
fn contains_reads_numeric_fields_and_rejects_missing_ones() {
    let task = sample(); // priority 8, start None
    assert!(hit("contains(\"8\", priority)", &task));
    assert!(!hit("contains(\"9\", priority)", &task));
    assert!(!hit("contains(\"2026\", start)", &task));
}

#[test]
fn path_field_contains_matches_nested_dir_not_flat_sibling() {
    // `sample()` is flat (tasks/add-auth.typ) — no "sub/" segment, so a
    // subdirectory task must match `contains("sub/", path)` and its flat
    // sibling must not.
    let mut nested = sample();
    nested.file = "tasks/sub/add-auth.typ".into();
    assert!(hit("contains(\"sub/\", path)", &nested));

    let flat = sample();
    assert!(!hit("contains(\"sub/\", path)", &flat));
}

#[test]
fn path_field_equality_matches_the_full_root_relative_path() {
    let task = sample(); // file: tasks/add-auth.typ
    assert!(hit("path == \"tasks/add-auth.typ\"", &task));
    assert!(!hit("path == \"tasks/other.typ\"", &task));
    assert!(hit("path != \"tasks/other.typ\"", &task));
}

/// `file` must fail loudly rather than silently: a reader who copies a path
/// out of the `file` COLUMN and writes `file` into a filter used to get an
/// empty listing, which is also what a correct filter over an empty result
/// set looks like
/// (`truth/tasks/the-file-filter-field-and-the-file-column-name-different-paths.typ`).
/// It is simply not a filter field — `path` is — so it fails as any other
/// unknown field would, with no pointer back to the replacement.
#[test]
fn the_file_field_is_an_unknown_field() {
    let got = err("file == \"tasks/add-auth.typ\"", &sample());
    assert!(matches!(&got, FilterError::UnknownField(name) if name == "file"));
}

#[test]
fn the_file_field_is_an_unknown_field_inside_contains() {
    let got = err("contains(\"sub/\", file)", &sample());
    assert!(
        matches!(&got, FilterError::UnknownField(name) if name == "file"),
        "{got:?}"
    );
}

#[test]
fn depth_field_dispatches_as_a_number() {
    // depth is always numeric (a root task is 0), so it compares numerically and
    // its value walks the same dispatch as priority/difficulty.
    let mut task = sample(); // depth 0
    assert!(hit("depth == 0", &task));
    assert!(hit("depth < 1", &task));
    assert!(!hit("depth >= 1", &task));
    task.depth = 3;
    assert!(hit("depth > 2", &task));
    assert!(hit("depth != 0", &task));
}

/// Each listed word reaches a dispatch arm: called with no arguments it fails,
/// if at all, on its arguments or its type, never as an unknown function.
#[test]
fn every_listed_word_is_a_function_the_dispatcher_knows() {
    for entry in crate::filter::functions() {
        let word = entry.name;
        let result = Filter::parse(&format!("{word}()"))
            .unwrap()
            .matches(&sample(), &Config::default());
        if let Err(got) = result {
            assert!(
                !got.to_string().contains("unknown function"),
                "{word}: {got}"
            );
        }
    }
}

#[test]
fn names_term_covers_language_words_fields_and_config_names() {
    let config =
        parse_config("[views.triage]\nfilter = \"ready\"\n\n[dicts.verbs]\nvalues = [\"fix\"]\n")
            .unwrap();
    for word in ["ready", "has", "due", "triage", "verbs", "done"] {
        assert!(Filter::names_term(word, &config), "{word}");
    }
    for word in ["k3x", "alpha", "@triage"] {
        assert!(!Filter::names_term(word, &config), "{word}");
    }
}

/// A word with arity zero is a bare predicate; every other registry word is
/// refused bare as a non-boolean field, and an unregistered one is unknown.
#[test]
fn bare_words_follow_the_registry_arity() {
    for entry in crate::filter::functions() {
        let result = Filter::parse(entry.name)
            .unwrap()
            .matches(&sample(), &Config::default());
        let bare_predicate = entry.arity.max == Some(0);
        let refused = result
            .as_ref()
            .is_err_and(|got| got.to_string().contains("not a condition"));
        assert_eq!(refused, !bare_predicate, "{}: {result:?}", entry.name);
    }
    let unknown = Filter::parse("nope()")
        .unwrap()
        .matches(&sample(), &Config::default());
    assert!(unknown.is_err_and(|got| got.to_string().contains("unknown function")));
}

/// The registry carries every name the language documents, including the
/// two-argument `has` and the bare `exists`.
#[test]
fn the_registry_names_each_word_once() {
    let names: Vec<&str> = crate::filter::functions().iter().map(|f| f.name).collect();
    for word in ["has", "exists", "ready", "stale", "in_dict"] {
        assert!(names.contains(&word), "{word}");
    }
    let unique: std::collections::BTreeSet<_> = names.iter().collect();
    assert_eq!(unique.len(), names.len());
}
