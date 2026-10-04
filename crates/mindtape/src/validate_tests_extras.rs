//! `[tasks.fields.x-<ns>]`: an extension slot held to its declared `values`
//! and `required` rule, and the load-time refusal of a `[tasks.fields]` key
//! that names nothing the table constrains. Shares `validate_tests.rs`'s
//! `config()`/`task()` fixtures.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use typst_harvest::HVal;

use super::tests::{config, task};
use super::validate;
use crate::config::{Config, FieldConfig, Requirement, parse_config};

fn machine_config(values: &[&str]) -> Config {
    let mut cfg = config();
    cfg.tasks.fields.insert(
        "x-machine".into(),
        FieldConfig {
            values: values.iter().map(|&v| v.to_owned()).collect(),
            default: None,
            required: Requirement::Expr("!terminal".to_owned()),
        },
    );
    cfg
}

fn machine_breaches(machine: Option<HVal>, status: &str, cfg: &Config) -> Vec<String> {
    let mut item = task("a", status);
    if let Some(value) = machine {
        item.extras.insert("x-machine".into(), value);
    }
    validate(&[item], cfg)
        .into_iter()
        .filter(|v| v.field == "x-machine")
        .map(|v| v.message)
        .collect()
}

#[test]
fn missing_required_extra_flags() {
    let breaches = machine_breaches(None, "wip", &machine_config(&["alpha"]));
    assert_eq!(breaches, ["required field is missing"]);
}

#[test]
fn missing_extra_passes_where_required_does_not_apply() {
    assert!(machine_breaches(None, "done", &machine_config(&["alpha"])).is_empty());
}

#[test]
fn extra_outside_values_flags() {
    let value = Some(HVal::Str("desktop".into()));
    let breaches = machine_breaches(value, "wip", &machine_config(&["alpha"]));
    assert_eq!(breaches, ["`desktop` is not an allowed value"]);
}

#[test]
fn extra_inside_values_passes() {
    let value = Some(HVal::Str("alpha".into()));
    assert!(machine_breaches(value, "wip", &machine_config(&["alpha"])).is_empty());
}

#[test]
fn non_string_extra_against_values_flags() {
    let value = Some(HVal::Dict(BTreeMap::new()));
    let breaches = machine_breaches(value, "wip", &machine_config(&["alpha"]));
    assert_eq!(breaches, ["a field with allowed values takes a string"]);
}

#[test]
fn required_extra_without_values_takes_any_shape() {
    let value = Some(HVal::Dict(BTreeMap::new()));
    assert!(machine_breaches(value, "wip", &machine_config(&[])).is_empty());
}

fn load_problem(toml: &str) -> Option<String> {
    parse_config(toml).unwrap().validate().err()
}

#[test]
fn extension_field_name_loads() {
    assert_eq!(
        load_problem("[tasks.fields.x-machine]\nvalues = [\"alpha\"]\n"),
        None
    );
}

#[test]
fn unknown_field_name_fails_load() {
    let problem = load_problem("[tasks.fields.machine]\nrequired = true\n").unwrap();
    assert!(problem.contains("tasks.fields.machine"), "{problem}");
    assert!(problem.contains("x-<ns>"), "{problem}");
}

#[test]
fn core_field_with_its_own_table_fails_load_naming_it() {
    let problem = load_problem("[tasks.fields.priority]\nrequired = true\n").unwrap();
    assert!(problem.contains("tasks.fields.priority"), "{problem}");
    assert!(problem.contains("[tasks.priority]"), "{problem}");
}
