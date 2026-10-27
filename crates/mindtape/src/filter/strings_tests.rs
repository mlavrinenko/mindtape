#![allow(clippy::unwrap_used)]

use crate::config::Config;
use crate::filter::{DictMiss, Filter, FilterError};
use crate::model::Task;
use crate::parse_config;

/// A task with a two-word title and no assignee — the missing-field arm every
/// one of these functions has to answer.
fn titled(title: &str) -> Task {
    Task {
        id: "add-auth".to_owned(),
        title: title.to_owned(),
        ..Task::default()
    }
}

fn hit(expr: &str, task: &Task, config: &Config) -> bool {
    Filter::parse(expr).unwrap().matches(task, config).unwrap()
}

fn err(expr: &str, task: &Task, config: &Config) -> FilterError {
    Filter::parse(expr)
        .unwrap()
        .matches(task, config)
        .unwrap_err()
}

/// A `fold` dict (the default) declared inline, plus an `exact` twin over the
/// same entry, so one config answers both halves of the case cross product.
fn cased_config() -> Config {
    parse_config(
        "[dicts.verbs]\nvalues = [\"add\", \"wire\"]\n\n\
         [dicts.strict]\nvalues = [\"add\"]\ncase = \"exact\"\n",
    )
    .unwrap()
}

// --- word() ----------------------------------------------------------------

#[test]
fn word_is_one_based() {
    let cfg = Config::default();
    let task = titled("Add auth to the gateway");
    assert!(hit("matches(word(title, 1), \"^Add$\")", &task, &cfg));
    assert!(hit("matches(word(title, 2), \"^auth$\")", &task, &cfg));
}

#[test]
fn word_trims_surrounding_punctuation() {
    let cfg = Config::default();
    assert!(hit(
        "matches(word(title, 1), \"^Fix$\")",
        &titled("Fix: the slug"),
        &cfg
    ));
    assert!(hit(
        "matches(word(title, 1), \"^Add$\")",
        &titled("**Add** auth"),
        &cfg
    ));
}

/// The rule the task artifact spells out: `word()` does not read Typst markup,
/// so a title opening with a code span yields the bare token inside the ticks —
/// not a verb, so a verb rule fires and the author exempts it with `matches()`.
#[test]
fn a_backticked_first_word_yields_the_span_not_a_verb() {
    let cfg = Config::default();
    let task = titled("`mt ls` should print the path");
    assert!(hit("matches(word(title, 1), \"^mt$\")", &task, &cfg));
    assert!(!hit(
        "in_dict(\"mindtape-verbs\", word(title, 1))",
        &task,
        &cfg
    ));
    assert!(hit("matches(title, \"^`\")", &task, &cfg));
}

#[test]
fn a_word_index_past_the_end_is_a_non_match() {
    let cfg = Config::default();
    let task = titled("Add auth");
    assert!(!hit(
        "in_dict(\"mindtape-verbs\", word(title, 9))",
        &task,
        &cfg
    ));
    assert!(!hit("matches(word(title, 9), \"\")", &task, &cfg));
}

#[test]
fn a_missing_field_is_a_non_match_not_an_error() {
    let cfg = Config::default();
    let task = titled("Add auth"); // no assignee
    assert!(!hit(
        "in_dict(\"mindtape-verbs\", word(assignee, 1))",
        &task,
        &cfg
    ));
    assert!(!hit("matches(assignee, \"\")", &task, &cfg));
}

/// A token with no alphanumeric character trims to nothing, which reads the
/// same as no word at all.
#[test]
fn an_all_punctuation_word_resolves_to_nothing() {
    let cfg = Config::default();
    assert!(!hit(
        "matches(word(title, 1), \"\")",
        &titled("— dashed"),
        &cfg
    ));
}

#[test]
fn word_nests_inside_word() {
    let cfg = Config::default();
    // The first word of a one-word string is that string.
    let task = titled("Add auth");
    assert!(hit(
        "matches(word(word(title, 2), 1), \"^auth$\")",
        &task,
        &cfg
    ));
}

#[test]
fn word_reads_a_string_literal_too() {
    let cfg = Config::default();
    assert!(hit(
        "in_dict(\"mindtape-verbs\", word(\"add the thing\", 1))",
        &titled("Add auth"),
        &cfg
    ));
}

// --- in_dict() -------------------------------------------------------------

#[test]
fn in_dict_reads_the_first_word_of_the_title() {
    let cfg = cased_config();
    assert!(hit(
        "in_dict(\"verbs\", word(title, 1))",
        &titled("Add auth to the gateway"),
        &cfg
    ));
    assert!(!hit(
        "in_dict(\"verbs\", word(title, 1))",
        &titled("Gateway auth"),
        &cfg
    ));
}

/// `fold` (the default) matches a capitalised title word against a lowercase
/// entry; `exact` over the same entry does not.
#[test]
fn a_fold_dict_matches_a_capitalised_word_and_an_exact_one_does_not() {
    let cfg = cased_config();
    let task = titled("Add auth");
    assert!(hit("in_dict(\"verbs\", word(title, 1))", &task, &cfg));
    assert!(!hit("in_dict(\"strict\", word(title, 1))", &task, &cfg));
    assert!(hit(
        "in_dict(\"strict\", word(title, 1))",
        &titled("add auth"),
        &cfg
    ));
}

#[test]
fn the_builtins_resolve_without_a_dicts_section() {
    let cfg = Config::default();
    assert!(hit(
        "in_dict(\"mindtape-verbs\", word(title, 1))",
        &titled("Add auth"),
        &cfg
    ));
    assert!(hit(
        "in_dict(\"mindtape-dev-verbs\", word(title, 1))",
        &titled("Refactor the walker"),
        &cfg
    ));
    assert!(!hit(
        "in_dict(\"mindtape-verbs\", word(title, 1))",
        &titled("Refactor the walker"),
        &cfg
    ));
}

#[test]
fn in_dict_takes_a_bare_field_and_a_literal() {
    let cfg = cased_config();
    let task = titled("Add auth");
    assert!(hit("in_dict(\"verbs\", \"wire\")", &task, &cfg));
    assert!(!hit("in_dict(\"verbs\", id)", &task, &cfg));
}

#[test]
fn an_unknown_dict_name_errors_and_names_the_known_ones() {
    let got = err(
        "in_dict(\"verb\", word(title, 1))",
        &titled("Add auth"),
        &cased_config(),
    );
    let FilterError::UnknownDict { name, unknown } = &got else {
        panic!("{got:?}");
    };
    assert_eq!(name, "verb");
    assert_eq!(unknown.nearest.as_deref(), Some("verbs"));
    assert!(unknown.names.iter().any(|known| known == "mindtape-verbs"));
}

// --- matches() -------------------------------------------------------------

#[test]
fn matches_runs_a_regex_over_a_string_field() {
    let cfg = Config::default();
    assert!(hit("matches(title, \"^Add \")", &titled("Add auth"), &cfg));
    assert!(!hit("matches(title, \"^Fix \")", &titled("Add auth"), &cfg));
    assert!(hit("matches(id, \"auth$\")", &titled("Add auth"), &cfg));
}

/// `matches()` reads the text-only fields `contains()` reads: the prose body,
/// every link note joined, and the tags, authored and derived, joined.
#[test]
fn matches_reads_the_text_only_fields() {
    let cfg = Config::default();
    let mut task = titled("Add auth");
    task.body = "== Summary\n\n- TODO: say what is wrong\n".to_owned();
    task.links = vec![crate::test_support::noted_edge(
        "other.typ",
        "related",
        "blocked on review",
    )];
    task.tags = vec!["backend".to_owned()];
    task.derived_tags = vec!["auth".to_owned()];
    assert!(hit("matches(body, \"(?m)^- TODO:\")", &task, &cfg));
    assert!(hit("matches(note, \"on review$\")", &task, &cfg));
    assert!(hit("matches(tags, \"^backend auth$\")", &task, &cfg));

    task.body = "== Summary\n\nThe login drops a TODO: marker.\n".to_owned();
    assert!(!hit("matches(body, \"(?m)^- TODO:\")", &task, &cfg));
}

#[test]
fn an_invalid_regex_errors_naming_the_pattern() {
    let got = err(
        "matches(title, \"[\")",
        &titled("Add auth"),
        &Config::default(),
    );
    let rendered = got.to_string();
    assert!(rendered.contains("bad regex"), "{rendered}");
    assert!(rendered.contains("'['"), "{rendered}");
}

// --- error discipline ------------------------------------------------------

#[test]
fn a_numeric_field_in_string_position_errors() {
    let cfg = Config::default();
    let task = titled("Add auth");
    // `depth` is the field that is always numeric and never absent, so it
    // reaches the numeric arm rather than the missing one.
    for expr in [
        "in_dict(\"mindtape-verbs\", word(depth, 1))",
        "matches(depth, \"0\")",
    ] {
        let got = err(expr, &task, &cfg);
        assert!(
            matches!(&got, FilterError::Invalid(msg) if msg.contains("is a number")),
            "{expr}: {got:?}"
        );
    }
}

#[test]
fn an_unknown_field_in_string_position_errors() {
    let got = err(
        "in_dict(\"mindtape-verbs\", word(bogus, 1))",
        &titled("Add auth"),
        &Config::default(),
    );
    assert!(matches!(got, FilterError::UnknownField(_)), "{got:?}");
}

#[test]
fn a_wrong_argument_count_errors_per_function() {
    let cfg = cased_config();
    let task = titled("Add auth");
    for (expr, needle) in [
        ("in_dict(\"verbs\")", "in_dict()"),
        ("matches(title)", "matches()"),
        ("in_dict(\"verbs\", word(title))", "word()"),
        ("in_dict(\"verbs\", word(title, 1, 2))", "word()"),
    ] {
        let got = err(expr, &task, &cfg);
        assert!(
            matches!(&got, FilterError::Call { problem, .. } if problem.contains(needle)),
            "{expr}: {got:?}"
        );
    }
}

#[test]
fn a_word_index_below_one_errors() {
    let got = err(
        "in_dict(\"mindtape-verbs\", word(title, 0))",
        &titled("Add auth"),
        &Config::default(),
    );
    assert!(
        matches!(&got, FilterError::Call { problem, .. } if problem.contains("counts from 1")),
        "{got:?}"
    );
}

#[test]
fn a_non_literal_dict_name_errors() {
    let got = err(
        "in_dict(title, word(title, 1))",
        &titled("Add auth"),
        &Config::default(),
    );
    assert!(
        matches!(&got, FilterError::Call { problem, .. } if problem.contains("quoted string")),
        "{got:?}"
    );
}

#[test]
fn word_in_boolean_position_errors_naming_where_it_belongs() {
    let got = err("word(title, 1)", &titled("Add auth"), &Config::default());
    assert!(
        matches!(&got, FilterError::Call { example, .. } if example.contains("in_dict")),
        "{got:?}"
    );
}

#[test]
fn a_boolean_call_in_string_position_errors() {
    let got = err(
        "in_dict(\"mindtape-verbs\", has_tag(\"x\"))",
        &titled("Add auth"),
        &Config::default(),
    );
    assert!(
        matches!(&got, FilterError::Call { problem, .. } if problem.contains("has_tag")),
        "{got:?}"
    );
}

/// An operator or a negated field where a string belongs is an error, not a
/// value: `title + id` is named in the message, and `-title` is no literal.
#[test]
fn an_operator_in_string_position_errors() {
    let cfg = Config::default();
    let plus = err("matches(title + id, \"x\")", &titled("Add auth"), &cfg);
    assert!(
        matches!(&plus, FilterError::Invalid(text)
            if text.contains("expected a string, a field or word(...)") && text.contains("'+'")),
        "{plus:?}"
    );
    let neg = err("matches(-title, \"x\")", &titled("Add auth"), &cfg);
    assert!(matches!(neg, FilterError::Invalid(_)), "{neg:?}");
}

/// The whole point of the seam: these compose with everything else in the
/// language, including a view reference.
#[test]
fn in_dict_composes_with_the_rest_of_the_language() {
    let cfg = parse_config(
        "[dicts.verbs]\nvalues = [\"add\"]\n\n\
         [views.verbed]\nfilter = 'in_dict(\"verbs\", word(title, 1))'\n",
    )
    .unwrap();
    assert!(hit("@verbed && has(title)", &titled("Add auth"), &cfg));
    assert!(!hit("@verbed", &titled("Gateway auth"), &cfg));
}

// --- dict_miss() -----------------------------------------------------------

/// The miss `expr` reports for a task titled `title`, under [`cased_config`].
fn miss(expr: &str, title: &str) -> Option<DictMiss> {
    Filter::parse(expr)
        .unwrap()
        .dict_miss(&titled(title), &cased_config())
}

fn verbs_miss(word: &str, nearest: &[&str]) -> DictMiss {
    DictMiss {
        dict: "verbs".to_owned(),
        word: word.to_owned(),
        nearest: nearest.iter().map(|entry| (*entry).to_owned()).collect(),
    }
}

const VERB_FIRST: &str = "in_dict(\"verbs\", word(title, 1))";

#[test]
fn a_miss_names_the_dict_the_word_as_written_and_the_nearest_entries() {
    assert_eq!(
        miss(VERB_FIRST, "Ad auth"),
        Some(verbs_miss("Ad", &["add"]))
    );
    assert_eq!(
        miss(VERB_FIRST, "Zzyzx auth"),
        Some(verbs_miss("Zzyzx", &[]))
    );
}

#[test]
fn a_hit_or_a_missing_word_is_no_miss() {
    assert_eq!(miss(VERB_FIRST, "Add auth"), None);
    assert_eq!(miss("in_dict(\"verbs\", word(title, 9))", "Ad auth"), None);
    assert_eq!(miss("", "Ad auth"), None);
}

/// An `exact` dict misses a word that differs in case alone, and ranks the
/// word as written.
#[test]
fn an_exact_miss_keeps_the_case_of_the_word() {
    let found = miss("in_dict(\"strict\", word(title, 1))", "Add auth").unwrap();
    assert_eq!(
        (found.dict.as_str(), found.word.as_str()),
        ("strict", "Add")
    );
    assert_eq!(found.nearest, ["add"]);
}

#[test]
fn and_or_and_parentheses_reach_the_call() {
    let expected = Some(verbs_miss("Ad", &["add"]));
    for expr in [
        "has(title) && in_dict(\"verbs\", word(title, 1))",
        "matches(title, \"^x\") || in_dict(\"verbs\", word(title, 1))",
        "(in_dict(\"verbs\", word(title, 1)))",
    ] {
        assert_eq!(miss(expr, "Ad auth"), expected, "{expr}");
    }
}

/// A call under `!`, inside `count(...)` or behind a view made the
/// expression no falser by missing, so it explains nothing. Neither does a
/// call to another function of the same shape, such as
/// `contains("verbs", title)`.
#[test]
fn a_negated_nested_or_other_call_explains_nothing() {
    for expr in [
        "!in_dict(\"verbs\", word(title, 1))",
        "count(in_dict(\"verbs\", word(title, 1))) > 0",
        "in_view(\"verbed\")",
        "contains(\"verbs\", title)",
    ] {
        assert_eq!(miss(expr, "Ad auth"), None, "{expr}");
    }
}

#[test]
fn the_first_miss_in_reading_order_wins() {
    let found = miss(
        "in_dict(\"strict\", word(title, 1)) || in_dict(\"verbs\", word(title, 1))",
        "Ad auth",
    );
    assert_eq!(found.unwrap().dict, "strict");
}
