#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{HeaderAction, HeaderPathSegment, Requirement, parse_config};

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

#[test]
fn parses_header_require_rule() {
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    let rules = &config.tasks.body.headers.rules;
    assert_eq!(rules.len(), 1);
    assert_eq!(
        rules[0].path,
        vec![HeaderPathSegment::Exact("Summary".to_owned())]
    );
    assert_eq!(rules[0].action, HeaderAction::Require);
    assert!(rules[0].when.is_none());
    assert!(rules[0].message.is_none());
}

#[test]
fn parses_header_mixed_path_and_when() {
    let text = r#"
[[tasks.body.headers.rules]]
path    = ["Approach", { match = "^Phase", recursive = true }]
action  = "forbid"
when    = 'has_tag("milestone")'
message = "no phases"
"#;
    let config = parse_config(text).unwrap();
    let rule = &config.tasks.body.headers.rules[0];
    assert_eq!(rule.path.len(), 2);
    assert_eq!(
        rule.path[0],
        HeaderPathSegment::Exact("Approach".to_owned())
    );
    assert_eq!(
        rule.path[1],
        HeaderPathSegment::Pattern {
            pattern: "^Phase".to_owned(),
            recursive: true,
        }
    );
    assert_eq!(rule.action, HeaderAction::Forbid);
    assert_eq!(rule.when.as_deref(), Some("has_tag(\"milestone\")"));
    assert_eq!(rule.message.as_deref(), Some("no phases"));
}

#[test]
fn parses_pattern_segment_without_recursive() {
    let text =
        "[[tasks.body.headers.rules]]\npath = [{ match = \"^Phase\" }]\naction = \"forbid\"\n";
    let config = parse_config(text).unwrap();
    assert_eq!(
        config.tasks.body.headers.rules[0].path[0],
        HeaderPathSegment::Pattern {
            pattern: "^Phase".to_owned(),
            recursive: false,
        }
    );
}

#[test]
fn rejects_invalid_header_action() {
    let text = "[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"unknown\"\n";
    assert!(parse_config(text).is_err());
}

#[test]
fn rejects_pattern_segment_without_match_key() {
    let text = "[[tasks.body.headers.rules]]\npath = [{ recursive = true }]\naction = \"forbid\"\n";
    assert!(parse_config(text).is_err());
}

#[test]
fn header_rules_default_to_empty() {
    let config = parse_config("").unwrap();
    assert!(config.tasks.body.headers.rules.is_empty());
}
