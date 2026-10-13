#![allow(clippy::unwrap_used)]

use crate::test_support::tempdir;
use std::collections::BTreeMap;

use super::{Filter, FilterError};
use crate::config::Config;
use crate::model::numeric::{InputValue, NumValue};
use crate::model::{Status, Task};
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
        waits_on: vec![],
        answers: std::collections::BTreeMap::new(),
        tags: vec!["backend".to_owned()],
        derived_tags: Vec::new(),
        depth: 0,
        topo: 0,
        blocks: 0,
        children: 0,
        children_done: 0,
        due: Some("2026-07-01".to_owned()),
        start: None,
        reviewed: None,
        recur: None,
        extras: BTreeMap::new(),
        shape_errors: Vec::new(),
        body: String::new(),
        spans: crate::model::task_spans::TaskCallSpans::default(),
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

/// A legend with `wip` at position 1, so `status.index` has a position to
/// resolve (the sample task's `wip`).
fn legend() -> Config {
    parse_config(
        "[[tasks.status]]\nvalue = \"proposed\"\n[[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap()
}

fn hit_with(expr: &str, task: &Task, cfg: &Config) -> bool {
    Filter::parse(expr).unwrap().matches(task, cfg).unwrap()
}

/// [`Filter::parse`] then [`Filter::matches`] under `config` — the helper the
/// view tests and `contains_tests` share.
fn hit_views(expr: &str, task: &Task, config: &Config) -> bool {
    Filter::parse(expr).unwrap().matches(task, config).unwrap()
}

#[test]
fn status_properties_are_filterable() {
    let cfg = legend();
    let task = sample(); // wip, stamped 2026-06-14, no note
    assert!(hit_with("status.name == \"wip\"", &task, &cfg));
    assert!(!hit_with("status.name == \"done\"", &task, &cfg));
    assert!(hit_with("status.date == \"2026-06-14\"", &task, &cfg));
    assert!(hit_with("status.date < \"2026-07-01\"", &task, &cfg));
    assert!(!hit_with("status.date > \"2026-07-01\"", &task, &cfg));
    assert!(hit_with("status.index == 1", &task, &cfg));
    assert!(hit_with("status.index >= 1", &task, &cfg));
    assert!(!hit_with("status.index < 1", &task, &cfg));
    // Bare `status` keeps comparing the lifecycle name.
    assert!(hit_with("status == \"wip\"", &task, &cfg));
    // A part with no authored value follows the missing-field rule.
    assert!(hit_with("status.note != \"x\"", &task, &cfg));
    assert!(!hit_with("status.note == \"x\"", &task, &cfg));
}

#[test]
fn status_property_paths_reach_the_filter_helpers() {
    let cfg = legend();
    let mut task = sample();
    task.status.as_mut().unwrap().note = Some("waiting on review".to_owned());
    assert!(hit_with("has(status.note)", &task, &cfg));
    assert!(hit_with("contains(\"review\", status.note)", &task, &cfg));
    assert!(hit_with("matches(status.note, \"^wait\")", &task, &cfg));
    task.status.as_mut().unwrap().note = None;
    assert!(hit_with("miss(status.note)", &task, &cfg));
    assert!(!hit_with("has(status.note)", &task, &cfg));
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
fn the_retired_underscore_link_names_are_unknown_fields() {
    // `has(depends_on)` etc. used to be sugar for `links("depends-on")`; a
    // caller now reaches for `links("kind")` instead, and the underscore
    // spelling fails as any other unknown field would.
    let mut task = sample();
    task.links = vec![crate::test_support::dep_edge("dep.typ")];
    let err = Filter::parse("has(depends_on)")
        .unwrap()
        .matches(&task, &Config::default())
        .unwrap_err();
    assert!(
        matches!(&err, super::FilterError::UnknownField(name) if name == "depends_on"),
        "{err:?}"
    );
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
fn empty_filter_matches_all() {
    let task = sample();
    assert!(hit("", &task));
    assert!(hit("   ", &task));
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
    use crate::graph::ready::Readiness;

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

// --- Boundary regression tests ---

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
        matches!(&err, super::FilterError::Invalid(msg) if msg.contains("incomplete")),
        "{err:?}"
    );
}

// The rare arms of the `function` dispatcher live in a child module so they
// share `sample()` / `hit()` without duplicating them (and keep this file
// under the linecop limit).
#[path = "dispatch_tests.rs"]
mod dispatch;

// The `assignee` / `role` registry-backed filter tests likewise.
#[path = "actors_tests.rs"]
mod actors;

// `contains()`'s unified needle-first grammar (default haystack, explicit
// fields, the retired `search()` / field-first orders) likewise.
#[path = "contains_tests.rs"]
mod contains;

// Presence (`has`/`miss`), staleness (`stale`), and the `contains()`
// default-haystack fold — the tests for what moved into `filter::predicates`
// (file-size gate) — likewise.
#[path = "predicates_tests.rs"]
mod predicates;

// `in_view()`/`@name` predicate evaluation, the `view_refs` collector, and
// the cyclic view guard likewise.
#[path = "in_view_tests.rs"]
mod in_view;

// Reading an `x-<ns>` extension slot's value likewise.
#[path = "extras_tests.rs"]
mod extras;

#[test]
fn a_rule_reading_changed_loads_outside_a_repository() {
    let dir = tempdir();
    let mut config = parse_config(
        "[tasks.estimate]\nunits = [\"files\"]\n\n\
         [[tasks.rules]]\nname = \"budget\"\nassert = 'changed(\"worktree\") <= estimate.files'\n",
    )
    .unwrap();
    config.git.set_root(dir.path());
    config.validate().unwrap();
}

#[test]
fn reads_set_for_a_count_call_or_a_set_field_only() {
    let reads = |expr: &str| Filter::parse(expr).unwrap().reads_set();
    assert!(reads("count(!terminal) <= 4"));
    assert!(reads("title == \"x\" && topo > 1"));
    for field in ["ready", "blocked", "blocks", "children", "progress"] {
        assert!(reads(&format!("{field} == 1")), "{field}");
    }
    assert!(!reads("kind != \"fix\" && priority > 2"));
    assert!(!reads(""));
}

#[test]
fn an_operator_or_value_the_language_lacks_is_named_as_written() {
    for (expr, expected) in [
        ("\"wip\"", "'\"wip\"' on its own isn't a condition"),
        ("title + 1", "'+' isn't an operator the filter language has"),
        ("title = 1", "'=' isn't an operator the filter language has"),
    ] {
        let err = Filter::parse(expr)
            .unwrap()
            .matches(&sample(), &Config::default())
            .unwrap_err();
        assert!(
            matches!(&err, FilterError::Unsupported(reason) if reason == expected),
            "{expr}: {err:?}"
        );
    }
}
