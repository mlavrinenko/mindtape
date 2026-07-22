#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{Requirement, RootSpec, embedded_defaults, load_config, parse_config};

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
required = "!terminal"

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
    assert_eq!(priority.required, Requirement::Expr("!terminal".to_owned()));
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
required = "!terminal"
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
        Requirement::Expr("!terminal".to_owned())
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
fn rejects_unknown_top_level_section() {
    assert!(parse_config("[typo-section]\nfoo = 1\n").is_err());
}

#[test]
fn rejects_unknown_status_entry_key() {
    assert!(parse_config("[[tasks.status]]\nvalue = \"wip\"\nbadkey = true\n").is_err());
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

#[test]
fn embedded_defaults_matches_compiled_floor() {
    // `embedded_defaults()` is a public accessor for the same `include_str!`
    // constant `parse_config` merges as its base layer; re-including the file
    // here independently and comparing catches a mutant that stubs the
    // accessor's body (e.g. to `""`) without touching the real floor.
    assert_eq!(
        embedded_defaults(),
        include_str!("defaults.toml"),
        "embedded_defaults() must return the compiled floor verbatim"
    );
}

#[test]
fn requirement_deserialize_error_mentions_expected_types() {
    let err = parse_config("[tasks.fields.priority]\nrequired = 42\n").unwrap_err();
    assert!(
        err.to_string().contains("a boolean or a filter expression"),
        "catches expecting-noop mutant"
    );
}
