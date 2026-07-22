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
        estimate: BTreeMap::from([("hours".to_owned(), 8.0)]),
        links: vec![noted_edge("other.typ", "related", "context")],
        tags: vec!["backend".to_owned()],
        derived_tags: Vec::new(),
        due: Some("2026-07-01".to_owned()),
        start: None,
        reviewed: None,
        recur: None,
        extras: BTreeMap::new(),
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
fn estimate_is_presence_only() {
    let task = sample();
    assert!(hit("has(estimate)", &task));
    assert!(!hit("miss(estimate)", &task));
    // The opaque estimate dict has no scalar; comparison is rejected.
    assert!(
        Filter::parse("estimate > 4")
            .unwrap()
            .matches(&task, &Config::default())
            .is_err()
    );
}

#[test]
fn date_string_comparison() {
    let task = sample();
    assert!(hit("due < \"2026-08-01\"", &task));
    assert!(!hit("due > \"2026-08-01\"", &task));
}

#[test]
fn presence_functions() {
    let task = sample();
    assert!(hit("has(due)", &task));
    assert!(hit("miss(start)", &task));
    assert!(!hit("has(start)", &task));
    assert!(hit("has(tags)", &task));
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
    assert!(hit("contains(title, \"auth\")", &task)); // case-insensitive
    assert!(hit("search(\"AUTH\")", &task));
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
fn stale_without_reviewed_uses_status_date() {
    let mut task = sample();
    // Use a clearly old status date so the test is not time-sensitive.
    if let Some(ref mut s) = task.status {
        s.date = Some("2020-01-01".to_owned());
    }
    assert!(hit("stale(1)", &task));
}

#[test]
fn stale_with_recent_reviewed_is_not_stale() {
    let mut task = sample();
    task.reviewed = Some(crate::date::today_iso());
    assert!(!hit("stale(1)", &task));
}

#[test]
fn stale_with_old_reviewed_is_stale() {
    let mut task = sample();
    task.reviewed = Some("2020-01-01".to_owned());
    assert!(hit("stale(1)", &task));
}

#[test]
fn stale_bad_arg_errors() {
    let task = sample();
    assert!(
        Filter::parse("stale(not_a_number)")
            .unwrap()
            .matches(&task, &Config::default())
            .is_err()
    );
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

#[test]
fn search_matches_title_or_id() {
    // Guards against `||` → `&&` in the search function.
    let mut task = sample();
    task.id = "mod-001".to_owned();
    task.title = "Auth Module".to_owned();
    // "auth" in title, not in id — `||` gives true, `&&` gives false.
    assert!(hit("search(\"auth\")", &task));
}

#[test]
fn has_tags_distinguishes_each_negation() {
    // Guards against deletion of `!` in `present` for tags.
    // The expression is `!task.tags.is_empty() || !task.derived_tags.is_empty()`.
    let mut task = sample();
    // Non-empty tags, empty derived → true (catches first `!` deleted)
    task.tags = vec!["backend".to_owned()];
    task.derived_tags = Vec::new();
    assert!(hit("has(tags)", &task));
    // Empty tags, non-empty derived → true (catches second `!` deleted)
    task.tags = Vec::new();
    task.derived_tags = vec!["frontend".to_owned()];
    assert!(hit("has(tags)", &task));
    // Both empty → false (catches either `!` deleted)
    task.tags = Vec::new();
    task.derived_tags = Vec::new();
    assert!(!hit("has(tags)", &task));
}

#[test]
fn stale_with_reviewed_at_exact_threshold_is_not_stale() {
    // Guards against `<` → `<=` in the reviewed branch of `is_stale`.
    // When reviewed equals days_ago_iso(days), real `<` returns false but
    // mutant `<=` returns true.
    let mut task = sample();
    task.reviewed = Some(crate::date::days_ago_iso(1));
    assert!(!hit("stale(1)", &task));
}

#[test]
fn stale_with_status_at_exact_threshold_is_not_stale() {
    // Guards against `<` → `<=` in the status-date branch of `is_stale`.
    let mut task = sample();
    task.reviewed = None;
    if let Some(ref mut s) = task.status {
        s.date = Some(crate::date::days_ago_iso(1));
    }
    assert!(!hit("stale(1)", &task));
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
fn contains_does_not_match_absent_substring() {
    // Guards against `contains_ci` returning `true` unconditionally.
    let task = sample();
    assert!(!hit("contains(title, \"xyz\")", &task));
}

// The rare arms of the `function` dispatcher live in a child module so they
// share `sample()` / `hit()` without duplicating them (and keep this file
// under the linecop limit).
#[path = "filter_dispatch_tests.rs"]
mod dispatch;
