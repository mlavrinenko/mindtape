//! Reading an `x-<ns>` extension slot's value: a scalar compares like a core
//! field, a list answers membership through `has(field, value)`, a dict stays
//! presence-only. Readability does not depend on `[tasks.fields.x-<ns>]`, so
//! every test here runs under the default config.

use std::collections::BTreeMap;

use typst_harvest::HVal;

use super::{hit, sample};
use crate::config::Config;
use crate::filter::Filter;
use crate::model::Task;
use crate::ready::Readiness;
use crate::sort::{sort_by_specs, validate_sort_spec};

fn with_extra(name: &str, value: HVal) -> Task {
    let mut task = sample();
    task.extras.insert(name.to_owned(), value);
    task
}

fn text(value: &str) -> HVal {
    HVal::Str(value.to_owned())
}

fn machines(names: &[&str]) -> HVal {
    HVal::Array(names.iter().map(|name| text(name)).collect())
}

#[test]
fn a_scalar_extra_compares_like_a_core_field() {
    let task = with_extra("x-machine", text("alpha"));
    assert!(hit(r#"x-machine == "alpha""#, &task));
    assert!(!hit(r#"x-machine != "alpha""#, &task));
    assert!(!hit(r#"x-machine == "gamma""#, &task));
    assert!(hit(r#"x-machine < "z""#, &task));
    assert!(hit(r#"matches(x-machine, "^al")"#, &task));
    assert!(hit(r#"has(x-machine, "alpha")"#, &task));
    assert!(!hit(r#"has(x-machine, "gamma")"#, &task));
    let numeric = with_extra("x-points", HVal::Int(3));
    assert!(hit("x-points > 2", &numeric));
    assert!(!hit("x-points > 3", &numeric));
    let flag = with_extra("x-shared", HVal::Bool(true));
    assert!(hit("x-shared == true", &flag));
}

#[test]
fn a_list_extra_answers_membership_and_never_equals() {
    let task = with_extra("x-machine", machines(&["alpha", "gamma"]));
    assert!(hit(r#"has(x-machine, "alpha")"#, &task));
    assert!(hit(r#"has(x-machine, "gamma")"#, &task));
    assert!(!hit(r#"has(x-machine, "other")"#, &task));
    assert!(!hit(r#"x-machine == "alpha""#, &task));
    assert!(hit(r#"x-machine != "alpha""#, &task));
    assert!(!hit(r#"x-machine < "z""#, &task));
    assert!(!hit(r#"x-machine >= "a""#, &task));
    assert!(!hit(r#"matches(x-machine, "alpha")"#, &task));
}

#[test]
fn a_missing_extra_is_unequal_to_every_value() {
    let task = sample();
    assert!(hit(r#"x-machine != "alpha""#, &task));
    assert!(!hit(r#"x-machine == "alpha""#, &task));
    assert!(!hit(r#"x-machine < "z""#, &task));
    assert!(!hit(r#"has(x-machine, "alpha")"#, &task));
}

#[test]
fn a_dict_extra_stays_presence_only() {
    let goals = HVal::Dict(BTreeMap::from([("product".to_owned(), text("helps"))]));
    let task = with_extra("x-goals", goals);
    assert!(hit("has(x-goals)", &task));
    assert!(hit(r#"has(x-goals, "product")"#, &task));
    assert!(!hit(r#"has(x-goals, "process")"#, &task));
    let err = Filter::parse(r#"x-goals == "product""#)
        .unwrap()
        .matches(&task, &Config::default())
        .unwrap_err();
    assert!(err.to_string().contains("has(x-goals)"), "{err}");
}

#[test]
fn has_with_a_value_reads_a_core_field_by_equality() {
    let task = sample();
    assert!(hit(r#"has(kind, "feature")"#, &task));
    assert!(!hit(r#"has(kind, "bug")"#, &task));
    assert!(!hit(r#"has(assignee, "ada")"#, &task));
}

#[test]
fn has_refuses_a_third_argument() {
    let err = Filter::parse(r#"has(x-machine, "alpha", "other")"#)
        .unwrap()
        .matches(&sample(), &Config::default())
        .unwrap_err();
    assert!(err.to_string().contains("has("), "{err}");
}

#[test]
fn a_scalar_extra_sorts_with_missing_last() {
    assert!(validate_sort_spec("x-machine:desc").is_ok());
    let mut tasks = vec![
        with_extra("x-machine", text("alpha")),
        sample(),
        with_extra("x-machine", text("gamma")),
    ];
    for (index, task) in tasks.iter_mut().enumerate() {
        task.id = format!("t{index}");
    }
    let config = Config::default();
    sort_by_specs(
        &mut tasks,
        &["x-machine".to_owned()],
        &config,
        &Readiness::empty(),
    )
    .unwrap();
    let order: Vec<&str> = tasks.iter().map(|task| task.id.as_str()).collect();
    assert_eq!(order, ["t0", "t2", "t1"]);
}

#[test]
fn has_reads_the_core_list_and_dict_fields() {
    let mut task = sample();
    task.tags = vec!["frontend".to_owned()];
    task.derived_tags = vec!["web".to_owned()];
    task.estimate = BTreeMap::from([("days".to_owned(), 2.0)]);
    assert!(hit(r#"has(tags, "frontend")"#, &task));
    assert!(hit(r#"has(tags, "web")"#, &task));
    assert!(!hit(r#"has(tags, "backend")"#, &task));
    assert!(hit(r#"has(estimate, "days")"#, &task));
    assert!(!hit(r#"has(estimate, "hours")"#, &task));
}
