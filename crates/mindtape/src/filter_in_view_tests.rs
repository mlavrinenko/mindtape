//! The `in_view("name")` / `@name` predicate as it evaluates: matching a named
//! view's filter, composing with boolean operators, the `UnknownView` error,
//! the `view_refs` collector, and the cyclic-view depth guard. Split out of
//! `filter_tests.rs`, which was near the linecop cap; the shared `sample()` and
//! `hit_views()` fixtures stay in the parent test module and are reached
//! through `super`.
#![allow(clippy::unwrap_used)]

use super::{hit_views, sample};
use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::parse_config;

// `sample()` is a backend task with priority 8 (so it matches both views).
fn config_with_views() -> Config {
    parse_config(
        "[views.backend]\nfilter = 'has_tag(\"backend\")'\n\
         [views.hot]\nfilter = 'priority > 5'\n",
    )
    .unwrap()
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
    assert!(matches!(err, FilterError::UnknownView(_)), "{err:?}");
}

#[test]
fn view_refs_collects_referenced_names() {
    let mut refs = Filter::parse("@a && !@b || in_view(\"c\")")
        .unwrap()
        .view_refs();
    refs.sort();
    assert_eq!(refs, vec!["a", "b", "c"]);
}

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
        matches!(&err, FilterError::Invalid(msg) if msg.contains("nested too deep")),
        "{err:?}"
    );
}
