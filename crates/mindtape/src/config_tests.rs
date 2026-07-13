#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{HeaderAction, HeaderPathSegment, Requirement, RootSpec, load_config, parse_config};

#[test]
fn defaults_when_empty() {
    let config = parse_config("").unwrap();
    assert_eq!(
        config.tasks.roots,
        vec![RootSpec::Short("tasks".to_owned())]
    );
    // The embedded floor supplies the six-value legend; `done`/`cancelled`/
    // `failed` are terminal, in legend order (the omitted-legend bug this
    // floor fixes: `Legend::default()` alone is empty).
    assert_eq!(
        config.tasks.status.terminal(),
        vec![
            "done".to_owned(),
            "cancelled".to_owned(),
            "failed".to_owned()
        ]
    );
    // White-list philosophy: an omitted estimate section accepts any unit.
    assert!(config.tasks.estimate.units.is_empty());
    // The default `mt ls` listing is the floor's `[views.default]` view.
    assert_eq!(
        config.ls_default_view().map(|view| view.filter.as_str()),
        Some("!terminal && (miss(start) || start <= now())")
    );
}

#[test]
fn explicit_status_legend_replaces_the_floor() {
    // Array-replace, not merge: the user's one-entry legend fully wins.
    let config = parse_config("[[tasks.status]]\nvalue = \"only\"\n").unwrap();
    assert_eq!(config.tasks.status.entries.len(), 1);
    assert_eq!(config.tasks.status.entries[0].value, "only");
    assert!(config.tasks.status.terminal().is_empty());
}

#[test]
fn empty_status_legend_clears_the_floor() {
    let config = parse_config("[tasks]\nstatus = []\n").unwrap();
    assert!(config.tasks.status.is_empty());
}

#[test]
fn load_config_on_missing_file_matches_parse_config_empty() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = load_config(dir.path()).unwrap();
    let parsed = parse_config("").unwrap();
    assert_eq!(loaded, parsed);
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

[views.default]
filter = "status != \"done\""
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
        config.ls_default_view().map(|view| view.filter.as_str()),
        Some("status != \"done\"")
    );
}

#[test]
fn ls_default_view_names_a_declared_view() {
    let config =
        parse_config("[ls]\ndefault-view = \"hot\"\n[views.hot]\nfilter = 'priority > 5'\n")
            .unwrap();
    assert_eq!(
        config.ls_default_view().map(|view| view.filter.as_str()),
        Some("priority > 5")
    );
    assert!(config.validate().is_ok());
}

#[test]
fn ls_default_view_unknown_errors_on_validate() {
    // The floor supplies `[views.default]`, but not `ghost`.
    let config = parse_config("[ls]\ndefault-view = \"ghost\"\n").unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("ls.default-view"), "{err}");
}

#[test]
fn valid_ls_rules_pass_validate() {
    let config = parse_config(
        "[views.inbox]\nfilter = 'status != \"done\"'\n\
         [[ls.rules]]\nname = \"release\"\n\
         condition = 'on_branch(\"release/*\")'\n\
         import-views = [\"inbox\"]\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn ls_rules_task_field_condition_errors_on_validate() {
    let config = parse_config("[[ls.rules]]\ncondition = \"priority > 5\"\n").unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("ls.rules[0].condition"), "{err}");
    assert!(err.contains("not available in a condition"), "{err}");
}

#[test]
fn ls_rules_import_views_still_catch_a_view_cycle() {
    // A rule importing one side of a view cycle must not mask the existing
    // (unextended) `config_views` cycle check: rules can only ever be a sink
    // in the reference graph (nothing can `import-views`/`@`-reference a rule
    // back), so they cannot themselves complete a cycle, but the underlying
    // views-only cycle must still surface.
    let config = parse_config(
        "[views.a]\nfilter = '@b'\n\
         [views.b]\nfilter = '@a'\n\
         [[ls.rules]]\nimport-views = [\"a\"]\n",
    )
    .unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("reference cycle"), "{err}");
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

#[test]
fn ls_columns_order_validates_and_rejects_unknown() {
    let ok = parse_config("[ls]\ncolumns-order = [\"title\", \"status\"]\n").unwrap();
    assert!(ok.validate().is_ok());
    let bad = parse_config("[ls]\ncolumns-order = [\"title\", \"bogus\"]\n").unwrap();
    assert!(bad.validate().is_err(), "unknown column name must fail");
}

#[test]
fn ls_columns_order_rejects_duplicate() {
    let config = parse_config("[ls]\ncolumns-order = [\"status\", \"status\"]\n").unwrap();
    assert!(
        config.validate().is_err(),
        "duplicate column name must fail"
    );
}

#[test]
fn status_transitions_reject_unknown_endpoint() {
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"proposed\"\nto = [\"wip\", \"typo\"]\n\
         [[tasks.status]]\nvalue = \"wip\"\n",
    )
    .unwrap();
    let err = config.validate().unwrap_err();
    assert!(
        err.contains("typo"),
        "unknown transition endpoint must fail: {err}"
    );
}

#[test]
fn status_transitions_sound_graph_validates() {
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"proposed\"\nto = [\"wip\"]\n\
         [[tasks.status]]\nvalue = \"wip\"\nto = [\"done\"]\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}
