//! Priority validation: the framework-vs-manual dispatch, the
//! active/known/valid-inputs checks a framework invocation must clear, and the
//! strict-mode policy a manual value must clear instead — exercising
//! `check_priority` directly, the same shape the `validate()` seam drives.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::check_priority;
use crate::config::{Config, parse_config};
use crate::model::Task;
use crate::model::numeric::{InputValue, NumValue};
use crate::test_support::{bare_task, ice_priority};
use crate::validate::Violation;

/// The violations `check_priority` reports for a task (`required` off: every
/// test here exercises a present priority value).
fn violations(task: &Task, config: &Config) -> Vec<Violation> {
    check_priority(task, &config.tasks.priority, false)
        .into_iter()
        .collect()
}

#[test]
fn framework_value_with_no_configured_framework_flags() {
    let mut item = bare_task("a");
    item.priority = Some(ice_priority(5.0, 0.5, 5.0));
    let problems = violations(&item, &Config::default());
    assert!(
        problems
            .iter()
            .any(|v| v.field == "priority"
                && v.message.contains("no priority framework is configured")),
        "a framework value under a config with no active framework should be flagged, got: {problems:?}"
    );
}

#[test]
fn active_framework_missing_from_registry_flags() {
    let config = parse_config("[tasks.priority]\nframework = \"ghost\"\n").unwrap();
    let mut item = bare_task("a");
    item.priority = Some(NumValue::Framework {
        name: "ghost".to_owned(),
        inputs: BTreeMap::new(),
    });
    let problems = violations(&item, &config);
    assert!(
        problems
            .iter()
            .any(|v| v.field == "priority" && v.message.contains("unknown framework `ghost`")),
        "an active framework absent from the registry should be flagged, got: {problems:?}"
    );
}

#[test]
fn categorical_score_matching_no_category_flags() {
    // A categorical framework with a numeric input escapes the exhaustive
    // coverage check, so a gap is loadable; a score landing in the gap must
    // still be flagged rather than silently banded.
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n\
         [tasks.priority]\nmin = 0\nmax = 9\nframework = \"custom\"\n\
         [tasks.priority.frameworks.custom]\n\
         inputs.impact = { type = \"number\", min = 1, max = 10 }\n\
         category = [{ when = \"impact >= 8\", score = 8 }]\n",
    )
    .unwrap();
    let mut item = bare_task("a");
    item.priority = Some(NumValue::Framework {
        name: "custom".to_owned(),
        inputs: BTreeMap::from([("impact".to_owned(), InputValue::Num(2.0))]),
    });
    let problems = violations(&item, &config);
    assert!(
        problems
            .iter()
            .any(|v| v.field == "priority" && v.message.contains("no category matched")),
        "a categorical score in an uncovered gap should be flagged, got: {problems:?}"
    );
}
