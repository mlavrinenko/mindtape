//! `values-when`: the filter naming the tasks whose field is held to its
//! allowed values. Tags default to `!terminal`; every other field to all
//! tasks. Shares `validate/tests.rs`'s `config()`/`task()` fixtures.
#![allow(clippy::unwrap_used)]

use typst_harvest::HVal;

use super::tests::task;
use super::validate;
use crate::config::{Config, parse_config};

fn with(extra: &str) -> Config {
    let mut toml = String::from(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    );
    toml.push_str(extra);
    parse_config(&toml).unwrap()
}

fn breaches(
    field: &str,
    status: &str,
    cfg: &Config,
    edit: impl Fn(&mut crate::model::Task),
) -> Vec<String> {
    let mut item = task("a", status);
    item.priority = None;
    item.estimate.clear();
    edit(&mut item);
    validate(&[item], cfg)
        .into_iter()
        .filter(|v| v.field == field)
        .map(|v| v.message)
        .collect()
}

fn delta(item: &mut crate::model::Task) {
    item.extras
        .insert("x-machine".into(), HVal::Str("delta".into()));
}

#[test]
fn an_extension_field_is_checked_on_a_closed_task_by_default() {
    let cfg = with("[tasks.fields.x-machine]\nvalues = [\"alpha\"]\n");
    assert_eq!(
        breaches("x-machine", "done", &cfg, delta),
        ["`delta` is not an allowed value"]
    );
}

#[test]
fn values_when_exempts_the_tasks_its_filter_leaves_out() {
    let cfg = with("[tasks.fields.x-machine]\nvalues = [\"alpha\"]\nvalues-when = \"!terminal\"\n");
    assert!(breaches("x-machine", "done", &cfg, delta).is_empty());
    assert_eq!(breaches("x-machine", "wip", &cfg, delta).len(), 1);
}

#[test]
fn values_when_exempts_a_closed_assignee() {
    let cfg = with("[tasks.assignee]\nallowed = [\"ada\"]\nvalues-when = \"!terminal\"\n");
    let edit = |item: &mut crate::model::Task| item.assignee = Some("bob".to_owned());
    assert!(breaches("assignee", "done", &cfg, edit).is_empty());
    assert_eq!(breaches("assignee", "wip", &cfg, edit).len(), 1);
}

#[test]
fn tags_skip_a_closed_task_unless_values_when_says_otherwise() {
    let edit = |item: &mut crate::model::Task| item.tags = vec!["retired".to_owned()];
    let default = with("[tasks.tags]\nallowed = [\"backend\"]\n");
    assert!(breaches("tags", "done", &default, edit).is_empty());
    let always = with("[tasks.tags]\nallowed = [\"backend\"]\nvalues-when = \"true\"\n");
    assert_eq!(breaches("tags", "done", &always, edit).len(), 1);
}

#[test]
fn an_unreadable_values_when_fails_the_load() {
    let cfg = with("[tasks.fields.x-machine]\nvalues = [\"alpha\"]\nvalues-when = \"nope ==\"\n");
    let problem = cfg.validate().unwrap_err();
    assert!(
        problem.contains("tasks.fields.x-machine.values-when"),
        "{problem}"
    );
}
