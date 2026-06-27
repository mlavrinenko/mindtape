#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{HeaderAction, HeaderPathSegment, Requirement, RootSpec, parse_config};

#[test]
fn defaults_when_empty() {
    let config = parse_config("").unwrap();
    assert_eq!(
        config.tasks.roots,
        vec![RootSpec::Short("tasks".to_owned())]
    );
    assert!(config.tasks.status.is_empty());
    // White-list philosophy: an omitted estimate section accepts any unit.
    assert!(config.tasks.estimate.units.is_empty());
    assert!(config.ls.default_filter.is_none());
}

#[test]
fn parses_full_config() {
    let text = r#"
[tasks]
roots = ["truth/tasks"]

[[tasks.status]]
value = "proposed"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.fields.priority]
values = ["low", "high"]
required = "non-terminal"

[tasks.fields.kind]
default = "feature"
required = false

[tasks.estimate]
units = ["hours", "days"]

[tasks.tags]
allowed = ["a", "b"]

[ls]
default-filter = "status != \"done\""
"#;
    let config = parse_config(text).unwrap();
    assert_eq!(
        config.tasks.roots,
        vec![RootSpec::Short("truth/tasks".to_owned())]
    );
    assert_eq!(config.tasks.status.terminal(), vec!["done".to_owned()]);

    let priority = config.tasks.fields.get("priority").unwrap();
    assert_eq!(priority.required, Requirement::NonTerminal);
    assert_eq!(priority.values, vec!["low".to_owned(), "high".to_owned()]);

    let kind = config.tasks.fields.get("kind").unwrap();
    assert_eq!(kind.required, Requirement::No);
    assert_eq!(kind.default.as_deref(), Some("feature"));

    let estimate = &config.tasks.estimate;
    assert_eq!(estimate.units, vec!["hours".to_owned(), "days".to_owned()]);

    assert_eq!(
        config.tasks.tags.allowed,
        vec!["a".to_owned(), "b".to_owned()]
    );
    assert_eq!(
        config.ls.default_filter.as_deref(),
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

#[test]
fn project_code_parses() {
    let config = parse_config("[project]\ncode = \"MT\"\n").unwrap();
    assert_eq!(config.project.code.as_deref(), Some("MT"));
}

#[test]
fn project_code_defaults_to_none() {
    let config = parse_config("").unwrap();
    assert!(config.project.code.is_none());
}

#[test]
fn id_format_defaults_to_string() {
    use super::IdFormat;
    let config = parse_config("").unwrap();
    assert_eq!(config.tasks.id.format, IdFormat::String);
}

#[test]
fn id_format_parses_uuid_v4() {
    use super::IdFormat;
    let config = parse_config("[tasks.id]\nformat = \"uuid-v4\"\n").unwrap();
    assert_eq!(config.tasks.id.format, IdFormat::UuidV4);
}

#[test]
fn id_format_parses_uuid_v7() {
    use super::IdFormat;
    let config = parse_config("[tasks.id]\nformat = \"uuid-v7\"\n").unwrap();
    assert_eq!(config.tasks.id.format, IdFormat::UuidV7);
}

#[test]
fn id_format_rejects_unknown() {
    assert!(parse_config("[tasks.id]\nformat = \"base62\"\n").is_err());
}

#[test]
fn effective_project_code_uses_configured_code() {
    use super::effective_project_code;
    use std::path::Path;
    let config = parse_config("[project]\ncode = \"MT\"\n").unwrap();
    assert_eq!(
        effective_project_code(&config, Path::new("/some/other-name")),
        "MT"
    );
}

#[test]
fn parses_numeric_priority_with_bands() {
    let text = r#"
[tasks.priority]
min = 0
max = 9
required = "non-terminal"
[[tasks.priority.bands]]
label = "low"
min = 0
max = 4
[[tasks.priority.bands]]
label = "high"
min = 5
max = 9
"#;
    let config = parse_config(text).unwrap();
    assert_eq!(config.tasks.priority.field.bands.len(), 2);
    assert_eq!(
        config.tasks.priority.field.required,
        Requirement::NonTerminal
    );
    assert!(config.validate().is_ok());
}

#[test]
fn validate_rejects_band_coverage_gap() {
    let text = r#"
[tasks.priority]
min = 0
max = 9
[[tasks.priority.bands]]
label = "low"
min = 0
max = 2
[[tasks.priority.bands]]
label = "high"
min = 5
max = 9
"#;
    let config = parse_config(text).unwrap();
    assert!(config.validate().is_err());
}

#[test]
fn effective_project_code_falls_back_to_dir_name() {
    use super::effective_project_code;
    use std::path::Path;
    let config = parse_config("").unwrap();
    assert_eq!(
        effective_project_code(&config, Path::new("/home/user/mindtape")),
        "mindtape"
    );
}

#[test]
fn default_sort_absent_means_empty() {
    let config = parse_config("").unwrap();
    assert!(config.ls.default_sort.is_empty());
}

#[test]
fn default_sort_parses_and_validates_known_fields() {
    let config = parse_config(
        r#"
[ls]
default-sort = ["ready", "priority:desc", "due:asc", "id"]
"#,
    )
    .unwrap();
    assert_eq!(
        config.ls.default_sort,
        vec!["ready", "priority:desc", "due:asc", "id"]
    );
}

#[test]
fn rejects_unknown_top_level_section() {
    assert!(parse_config("[typo-section]\nfoo = 1\n").is_err());
}

#[test]
fn rejects_unknown_ls_key() {
    assert!(parse_config("[ls]\ndefult-filter = \"!terminal\"\n").is_err());
}

#[test]
fn rejects_unknown_status_entry_key() {
    assert!(parse_config("[[tasks.status]]\nvalue = \"wip\"\nbadkey = true\n").is_err());
}

#[test]
fn default_sort_rejects_unknown_field_on_validate() {
    let config = parse_config(
        r#"
[ls]
default-sort = ["no_such_field"]
"#,
    )
    .unwrap();
    assert!(
        config.validate().is_err(),
        "expected validation error for unknown sort field"
    );
}

#[test]
fn ls_columns_parses_visibility() {
    let config = parse_config(
        r#"
[ls.columns.priority]
visibility = "visible"
"#,
    )
    .unwrap();
    assert_eq!(
        config.ls.columns["priority"].visibility,
        crate::Visibility::Visible
    );
    assert!(config.validate().is_ok());
}

#[test]
fn ls_columns_rejects_unknown_column_on_validate() {
    let config = parse_config(
        r#"
[ls.columns.nope]
visibility = "hidden"
"#,
    )
    .unwrap();
    assert!(
        config.validate().is_err(),
        "expected validation error for unknown column key"
    );
}

