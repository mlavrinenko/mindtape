#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::Filter;
use crate::config::Config;
use crate::model::{Status, Task};
use crate::numeric::{InputValue, NumValue};
use crate::parse_config;
use crate::test_support::noted_edge;

fn sample() -> Task {
    Task {
        id: "add-auth".to_owned(),
        file: "tasks/add-auth.typ".into(),
        title: "Add Auth".to_owned(),
        status: Some(Status {
            name: "wip".to_owned(),
            date: Some("2026-06-14".to_owned()),
            note: None,
        }),
        priority: Some(NumValue::Num(8.0)),
        difficulty: Some(NumValue::Num(6.0)),
        kind: Some("feature".to_owned()),
        assignee: None,
        estimate: BTreeMap::from([("hours".to_owned(), 8.0)]),
        links: vec![noted_edge("other.typ", "related", "context")],
        tags: vec!["backend".to_owned()],
        derived_tags: Vec::new(),
        depth: 0,
        due: Some("2026-07-01".to_owned()),
        start: None,
        reviewed: None,
        recur: None,
        extras: BTreeMap::new(),
        shape_errors: Vec::new(),
        body: String::new(),
    }
}

fn hit(expr: &str, task: &Task) -> bool {
    Filter::parse(expr)
        .unwrap()
        .matches(task, &Config::default())
        .unwrap()
}

#[test]
fn equality_and_inequality() {
    let task = sample();
    assert!(hit("status == \"wip\"", &task));
    assert!(!hit("status == \"done\"", &task));
    assert!(hit("status != \"done\"", &task));
    assert!(hit("priority == 8", &task));
}

#[test]
fn numeric_fields_compare_as_numbers() {
    let task = sample();
    assert!(hit("priority > 5", &task));
    assert!(!hit("priority < 8", &task));
    assert!(hit("priority >= 8", &task));
    assert!(hit("difficulty == 6", &task));
    assert!(hit("difficulty <= 6", &task));
}

#[test]
fn date_string_comparison() {
    let task = sample();
    assert!(hit("due < \"2026-08-01\"", &task));
    assert!(!hit("due > \"2026-08-01\"", &task));
}

#[test]
fn links_predicate_checks_kind_presence() {
    let task = sample();
    assert!(hit("links(\"related\")", &task));
    assert!(!hit("links(\"depends-on\")", &task));
}

#[test]
fn legacy_field_names_sugar_to_links_kind() {
    let mut task = sample();
    task.links = vec![
        crate::test_support::dep_edge("dep.typ"),
        crate::test_support::link_edge("old.typ", "supersedes"),
        crate::test_support::link_edge("twin.typ", "duplicates"),
    ];
    assert!(hit("has(depends_on)", &task));
    assert!(hit("has(supersedes)", &task));
    assert!(hit("has(duplicates)", &task));
    assert!(!hit("miss(depends_on)", &task));
}

#[test]
fn tag_and_text_functions() {
    let task = sample();
    assert!(hit("has_tag(\"backend\")", &task));
    assert!(!hit("has_tag(\"frontend\")", &task));
    assert!(hit("contains(\"auth\", title)", &task)); // case-insensitive
    assert!(hit("contains(\"AUTH\")", &task)); // default haystack
}

#[test]
fn boolean_combinators() {
    let task = sample();
    assert!(hit("priority >= 7 && status != \"done\"", &task));
    assert!(hit("status == \"done\" || has_tag(\"backend\")", &task));
    assert!(hit("!has(start)", &task));
}

#[test]
fn missing_field_inequality_is_true() {
    let mut task = sample();
    task.status = None;
    assert!(hit("status != \"done\"", &task));
    assert!(!hit("status == \"done\"", &task));
}

#[test]
fn unknown_field_errors() {
    let task = sample();
    let result = Filter::parse("bogus == \"x\"")
        .unwrap()
        .matches(&task, &Config::default());
    assert!(result.is_err());
}

#[test]
fn parse_error_surfaces() {
    assert!(Filter::parse("(status == \"x\"").is_err());
}

#[test]
fn reviewed_field_comparable() {
    let mut task = sample();
    task.reviewed = Some("2026-01-01".to_owned());
    assert!(hit("reviewed < \"2026-06-01\"", &task));
    assert!(!hit("reviewed > \"2026-06-01\"", &task));
    assert!(hit("has(reviewed)", &task));
    assert!(!hit("miss(reviewed)", &task));
}

#[test]
fn framework_priority_compares_by_resolved_score() {
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let mut task = sample();
    task.priority = Some(NumValue::Framework {
        name: "ice".to_owned(),
        inputs: BTreeMap::from([
            ("impact".to_owned(), InputValue::Num(10.0)),
            ("confidence".to_owned(), InputValue::Num(0.9)),
            ("ease".to_owned(), InputValue::Num(10.0)),
        ]),
    });
    // ICE score is 8: high comparisons hold, low ones don't.
    assert!(
        Filter::parse("priority >= 7")
            .unwrap()
            .matches(&task, &config)
            .unwrap()
    );
    assert!(
        !Filter::parse("priority < 5")
            .unwrap()
            .matches(&task, &config)
            .unwrap()
    );
}

#[test]
fn ready_and_blocked_predicates() {
    use crate::ready::Readiness;

    let cfg = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap();

    let mut dep = sample();
    dep.id = "dep".to_owned();
    dep.file = "tasks/dep.typ".into();
    dep.status = Some(Status {
        name: "done".to_owned(),
        date: None,
        note: None,
    });

    let mut ready = sample();
    ready.id = "r".to_owned();
    ready.file = "tasks/r.typ".into();
    ready.links = vec![crate::test_support::dep_edge("dep.typ")]; // terminal dep -> ready

    let mut blocked = sample();
    blocked.id = "b".to_owned();
    blocked.file = "tasks/b.typ".into();
    blocked.links = vec![crate::test_support::dep_edge("missing.typ")]; // dangling -> blocked

    let tasks = vec![dep.clone(), ready.clone(), blocked.clone()];
    let readiness = Readiness::compute(&tasks, &cfg);
    let hit = |expr: &str, task: &Task| {
        Filter::parse(expr)
            .unwrap()
            .matches_with(task, &cfg, &readiness)
            .unwrap()
    };

    // Bare and call forms agree.
    assert!(hit("ready", &ready));
    assert!(hit("ready()", &ready));
    assert!(!hit("ready", &blocked));
    assert!(hit("blocked", &blocked));
    assert!(hit("blocked()", &blocked));
    assert!(!hit("blocked", &ready));
    // Terminal task is neither.
    assert!(!hit("ready", &dep));
    assert!(!hit("blocked", &dep));
    // Composes with other predicates (sample priority is 8).
    assert!(hit("ready && priority >= 7", &ready));
    assert!(hit("!ready", &blocked));
}

#[test]
fn terminal_predicate() {
    let cfg = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap();

    let mut done = sample();
    done.status = Some(Status {
        name: "done".to_owned(),
        date: None,
        note: None,
    });
    let wip = sample(); // status "wip" — not terminal

    let hit = |expr: &str, task: &Task| Filter::parse(expr).unwrap().matches(task, &cfg).unwrap();

    // Bare and call forms agree.
    assert!(hit("terminal", &done));
    assert!(hit("terminal()", &done));
    assert!(!hit("terminal", &wip));
    assert!(hit("!terminal", &wip));
    // Composes with comparisons.
    assert!(hit("terminal && priority >= 7", &done));
}

#[test]
fn now_in_comparison() {
    // Use fixed dates far in the past / future so the test never flips.
    let mut past = sample();
    past.start = Some("2020-01-01".to_owned());

    let mut future = sample();
    future.start = Some("2099-12-31".to_owned());

    // start <= now() matches a past date, not a far-future date.
    assert!(hit("start <= now()", &past));
    assert!(!hit("start <= now()", &future));

    // now() also works with other comparison operators.
    assert!(hit("start < now()", &past));
    assert!(hit("start != now()", &future));

    // Intended default-filter shape: no-start tasks are always shown.
    assert!(hit("miss(start) || start <= now()", &past));
    assert!(!hit("miss(start) || start <= now()", &future));
    assert!(hit("miss(start) || start <= now()", &sample())); // start is None
}

// `sample()` is a backend task with priority 8 (so it matches both views).
fn config_with_views() -> Config {
    parse_config(
        "[views.backend]\nfilter = 'has_tag(\"backend\")'\n\
         [views.hot]\nfilter = 'priority > 5'\n",
    )
    .unwrap()
}

fn hit_views(expr: &str, task: &Task, config: &Config) -> bool {
    Filter::parse(expr).unwrap().matches(task, config).unwrap()
}

#[test]
fn empty_filter_matches_all() {
    let task = sample();
    assert!(hit("", &task));
    assert!(hit("   ", &task));
}

#[test]
fn in_view_predicate_matches_named_filter() {
    let task = sample();
    let cfg = config_with_views();
    assert!(hit_views("in_view(\"backend\")", &task, &cfg));
    assert!(hit_views("@backend", &task, &cfg));
    assert!(!hit_views("!@backend", &task, &cfg));
}

#[test]
fn in_view_composes_with_boolean_operators() {
    let task = sample();
    let cfg = config_with_views();
    assert!(hit_views("@hot && @backend", &task, &cfg));
    assert!(hit_views("@hot || @backend", &task, &cfg));
    // sample is hot (priority 8), so `!@hot` is false.
    assert!(!hit_views("@backend && !@hot", &task, &cfg));
}

#[test]
fn in_view_unknown_view_errors() {
    let task = sample();
    let cfg = config_with_views();
    let err = Filter::parse("@ghost")
        .unwrap()
        .matches(&task, &cfg)
        .unwrap_err();
    assert!(matches!(err, super::FilterError::UnknownView(_)), "{err:?}");
}

#[test]
fn view_refs_collects_referenced_names() {
    let mut refs = Filter::parse("@a && !@b || in_view(\"c\")")
        .unwrap()
        .view_refs();
    refs.sort();
    assert_eq!(refs, vec!["a", "b", "c"]);
}

// --- Boundary regression tests ---

#[test]
fn cyclic_view_hits_depth_limit() {
    // Guards against deeper() using `*` instead of `+` (depth never
    // increments, so the cycle guard never triggers).
    let cfg = parse_config("[views.cycle]\nfilter = '@cycle'\n").unwrap();
    let task = sample();
    let err = Filter::parse("@cycle")
        .unwrap()
        .matches(&task, &cfg)
        .unwrap_err();
    assert!(
        matches!(&err, super::FilterError::Invalid(msg) if msg.contains("nested too deep")),
        "{err:?}"
    );
}

/// The missing-field comparison rule `compare` implements and `[[tasks.limits]]`
/// rests on: an absent field is unequal to every literal and orders against
/// none. `kind != "umbrella"` must therefore count a task carrying no `kind`,
/// so a `kind == "x"` / `kind != "x"` limit pair covers the whole set.
#[test]
fn missing_field_is_unequal_to_every_literal() {
    let mut task = sample();
    task.kind = None;
    assert!(hit("kind != \"umbrella\"", &task), "`!=` holds when absent");
    assert!(!hit("kind == \"umbrella\"", &task));
    assert!(!hit("kind < \"umbrella\"", &task));
    assert!(!hit("kind > \"umbrella\"", &task));
    assert!(!hit("kind <= \"umbrella\"", &task));
    assert!(!hit("kind >= \"umbrella\"", &task));
    // Field-agnostic: the same holds for every optional field, string or
    // numeric (an unresolved priority is `Missing`, not zero).
    task.due = None;
    task.status = None;
    task.priority = None;
    assert!(hit("due != \"2026-01-01\"", &task));
    assert!(hit("status != \"wip\"", &task));
    assert!(hit("priority != 8", &task));
    assert!(!hit("priority >= 0", &task));
}

#[test]
fn binary_op_with_wrong_arity_is_invalid_at_match_time() {
    // `evalexpr`'s parser always builds `&&`/`||` with exactly two operands,
    // so the wrong-arity arm of `binary` is only reachable on a hand-built
    // tree, the same route the match-time errors take that `parse` cannot see.
    // Mutate a parsed tree: drop one of `And`'s operands.
    let mut tree = evalexpr::build_operator_tree("true && true").unwrap();
    let mut and = tree.children_mut().pop().unwrap();
    and.children_mut().pop();
    let filter = Filter { tree: Some(and) };
    let err = filter.matches(&sample(), &Config::default()).unwrap_err();
    assert!(
        matches!(&err, super::FilterError::Invalid(msg) if msg.contains("binary op needs two operands")),
        "{err:?}"
    );
}

// The rare arms of the `function` dispatcher live in a child module so they
// share `sample()` / `hit()` without duplicating them (and keep this file
// under the linecop limit).
#[path = "filter_dispatch_tests.rs"]
mod dispatch;

// The `assignee` / `role` registry-backed filter tests likewise.
#[path = "filter_actors_tests.rs"]
mod actors;

// `contains()`'s unified needle-first grammar (default haystack, explicit
// fields, the retired `search()` / field-first orders) likewise.
#[path = "filter_contains_tests.rs"]
mod contains;

// Presence (`has`/`miss`), staleness (`stale`), and the `contains()`
// default-haystack fold — the tests for what moved into `filter_predicates`
// (file-size gate) — likewise.
#[path = "filter_predicates_tests.rs"]
mod predicates;
