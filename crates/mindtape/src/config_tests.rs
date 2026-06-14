#![allow(clippy::unwrap_used)]

use super::{Requirement, parse_config};

#[test]
fn defaults_when_empty() {
    let config = parse_config("").unwrap();
    assert_eq!(config.tasks.roots, vec!["tasks".to_owned()]);
    assert!(config.tasks.status.values.is_empty());
    assert!(config.query.default_filter.is_none());
}

#[test]
fn parses_full_config() {
    let text = r#"
[tasks]
roots = ["truth/tasks"]

[tasks.status]
values = ["proposed", "wip", "done"]
terminal = ["done"]

[tasks.fields.priority]
values = ["low", "high"]
required = "non-terminal"

[tasks.fields.kind]
default = "feature"
required = false

[tasks.fields.estimate-hours]
type = "number"
min = 0

[tasks.tags]
allowed = ["a", "b"]

[query]
default_filter = "status != \"done\""
"#;
    let config = parse_config(text).unwrap();
    assert_eq!(config.tasks.roots, vec!["truth/tasks".to_owned()]);
    assert_eq!(config.tasks.status.terminal, vec!["done".to_owned()]);

    let priority = config.tasks.fields.get("priority").unwrap();
    assert_eq!(priority.required, Requirement::NonTerminal);
    assert_eq!(priority.values, vec!["low".to_owned(), "high".to_owned()]);

    let kind = config.tasks.fields.get("kind").unwrap();
    assert_eq!(kind.required, Requirement::No);
    assert_eq!(kind.default.as_deref(), Some("feature"));

    let estimate = config.tasks.fields.get("estimate-hours").unwrap();
    assert_eq!(estimate.kind.as_deref(), Some("number"));
    assert_eq!(estimate.min, Some(0.0));

    assert_eq!(
        config.tasks.tags.allowed,
        vec!["a".to_owned(), "b".to_owned()]
    );
    assert_eq!(
        config.query.default_filter.as_deref(),
        Some("status != \"done\"")
    );
}

#[test]
fn required_accepts_bool() {
    let config = parse_config("[tasks.fields.priority]\nrequired = true\n").unwrap();
    assert_eq!(
        config.tasks.fields.get("priority").unwrap().required,
        Requirement::Always
    );

    let config = parse_config("[tasks.fields.priority]\nrequired = false\n").unwrap();
    assert_eq!(
        config.tasks.fields.get("priority").unwrap().required,
        Requirement::No
    );
}

#[test]
fn rejects_invalid_toml() {
    assert!(parse_config("not = valid = toml").is_err());
}
