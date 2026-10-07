#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{Requirement, RootSpec, embedded_defaults, parse_config};

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

[tasks.fields.x-level]
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

    let level = config.tasks.fields.get("x-level").unwrap();
    assert_eq!(level.required, Requirement::Expr("!terminal".to_owned()));
    assert_eq!(level.values, vec!["low".to_owned(), "high".to_owned()]);

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
    let config = parse_config("[tasks.fields.kind]\nrequired = true\n").unwrap();
    assert_eq!(
        config.tasks.fields.get("kind").unwrap().required,
        Requirement::Always
    );

    let config = parse_config("[tasks.fields.kind]\nrequired = false\n").unwrap();
    assert_eq!(
        config.tasks.fields.get("kind").unwrap().required,
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
    let err = parse_config("[tasks.fields.kind]\nrequired = 42\n").unwrap_err();
    assert!(
        err.to_string().contains("a boolean or a filter expression"),
        "catches expecting-noop mutant"
    );
}

#[test]
fn unknown_priority_key_fails_the_load_naming_it() {
    let error = parse_config("[tasks.priority]\nbogus-key = 1\n").unwrap_err();
    assert!(
        format!("{error:?}").contains("bogus-key"),
        "error must name the key: {error:?}"
    );
}

const MACHINE: &str = "[tasks.fields.x-machine]\nvalues = [\"alpha\"]\n";

fn rule_reading(expr: &str) -> String {
    format!("{MACHINE}[[tasks.rules]]\nname = \"r\"\nassert = '{expr}'\nwhen = '{expr}'\n")
}

#[test]
fn a_filter_naming_an_undeclared_extra_warns_with_the_nearest_declared_name() {
    let config = parse_config(&rule_reading("x-machnie == \"alpha\"")).unwrap();
    let warnings = config.warnings();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(
        warnings[0].starts_with("tasks.rules[0].assert: `x-machnie`"),
        "{warnings:?}"
    );
    assert!(
        warnings[0].contains("did you mean: x-machine?"),
        "{warnings:?}"
    );
    assert!(
        warnings[1].starts_with("tasks.rules[0].when:"),
        "{warnings:?}"
    );
}

#[test]
fn a_declared_extra_does_not_warn() {
    let config = parse_config(&rule_reading("x-machine == \"alpha\"")).unwrap();
    assert!(config.warnings().is_empty());
}

#[test]
fn nothing_warns_while_no_extra_is_declared() {
    let toml = "[[tasks.rules]]\nname = \"r\"\nassert = 'x-machnie == \"alpha\"'\n";
    assert!(parse_config(toml).unwrap().warnings().is_empty());
}

#[test]
fn a_view_and_a_required_filter_warn_too() {
    let toml = format!(
        "{MACHINE}required = 'x-machnie == \"alpha\"'\n[views.mine]\nfilter = 'has(x-machnie)'\n"
    );
    let warnings = parse_config(&toml).unwrap().warnings();
    let paths: Vec<&str> = warnings
        .iter()
        .filter_map(|w| w.split(':').next())
        .collect();
    assert_eq!(
        paths,
        ["tasks.fields.x-machine.required", "views.mine.filter"],
        "{warnings:?}"
    );
}
