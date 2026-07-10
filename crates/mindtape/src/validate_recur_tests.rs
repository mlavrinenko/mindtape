#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::check_all;
use crate::config::Config;
use crate::recur::{PeriodUnit, Recur, RecurConfig, Rule};
use crate::test_support::bare_task;
use crate::validate::Violation;

fn weekly_recur() -> Recur {
    Recur {
        rule: Rule::Period {
            unit: PeriodUnit::Weeks,
            value: 1,
            weekdays: vec![],
        },
        anchor: None,
        until: None,
        count: None,
        materialize: None,
    }
}

#[test]
fn task_without_recur_is_untouched() {
    let task = bare_task("t");
    let config = Config::default();
    let mut out: Vec<Violation> = Vec::new();
    check_all(&task, &config, &mut out);
    assert!(out.is_empty());
}

#[test]
fn task_with_valid_recur_is_clean() {
    let mut task = bare_task("t");
    task.recur = Some(weekly_recur());
    let config = Config::default();
    let mut out: Vec<Violation> = Vec::new();
    check_all(&task, &config, &mut out);
    assert!(out.is_empty());
}

#[test]
fn task_with_recur_disabled_by_config_is_flagged() {
    let mut task = bare_task("t");
    task.recur = Some(weekly_recur());
    let mut config = Config::default();
    config.tasks.recur = RecurConfig {
        enabled: false,
        ..RecurConfig::default()
    };
    let mut out: Vec<Violation> = Vec::new();
    check_all(&task, &config, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].field, "recur");
    assert!(out[0].message.contains("disabled"));
}
