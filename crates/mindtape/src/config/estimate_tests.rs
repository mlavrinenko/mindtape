#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use crate::config::parse_config;
use crate::model::{Status, Task};
use crate::test_support::bare_task;
use crate::validate::validate;

/// A `wip` task declaring `quantity` of `unit`.
fn task(unit: &str, quantity: f64) -> Task {
    Task {
        status: Some(Status {
            name: "wip".to_owned(),
            date: None,
            note: None,
        }),
        estimate: BTreeMap::from([(unit.to_owned(), quantity)]),
        ..bare_task("t")
    }
}

const CONFIG: &str = "[[tasks.status]]\nvalue = \"wip\"\n\n[tasks.estimate]\nunits = [\"files\"]\n";

#[test]
fn a_unit_outside_the_allow_list_is_a_violation() {
    let config = parse_config(CONFIG).unwrap();
    let violations = validate(&[task("furlongs", 3.0)], &config);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations
            .first()
            .unwrap()
            .message
            .contains("not an allowed estimate unit"),
        "{violations:?}"
    );
}

#[test]
fn no_quantity_is_checked_without_a_rule() {
    let config = parse_config(CONFIG).unwrap();
    assert!(validate(&[task("files", 900.0)], &config).is_empty());
}

#[test]
fn the_retired_bound_and_drift_tables_are_unknown_keys() {
    for table in [
        "[tasks.estimate.drift]\nunit = \"files\"\n",
        "[[tasks.estimate.max]]\nfiles = 5\n",
        "[[tasks.estimate.min]]\nfiles = 1\n",
    ] {
        let err = parse_config(&format!("{CONFIG}{table}")).unwrap_err();
        assert!(err.to_string().contains("unknown field"), "{table}: {err}");
    }
}

#[test]
fn queries_name_a_path_set_per_unit() {
    let config = parse_config(&format!(
        "{CONFIG}\n[tasks.estimate.queries]\nfiles = \"src/**\"\n"
    ))
    .unwrap();
    assert_eq!(
        config.tasks.estimate.queries,
        BTreeMap::from([("files".to_owned(), "src/**".to_owned())])
    );
}
