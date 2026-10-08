//! `[ls]`/`[views.*]` — the default view, rule import/gating, sort spec
//! validation, and the columns table (`when`, order, duplicate names).
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::parse_config;

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
    // The floor supplies `[views.default]`, but not `ghost` — too far from
    // "default" to suggest, so the error enumerates without a "did you mean".
    let config = parse_config("[ls]\ndefault-view = \"ghost\"\n").unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("ls.default-view"), "{err}");
    let unknown = err.unknown.unwrap();
    assert_eq!(unknown.names, ["default"]);
    assert_eq!(unknown.nearest, None);
}

#[test]
fn ls_default_view_near_miss_suggests_the_close_name() {
    // The floor's own `[views.default]` name is one transposition away.
    let config = parse_config("[ls]\ndefault-view = \"defualt\"\n").unwrap();
    let err = config.validate().unwrap_err();
    assert_eq!(err.unknown.unwrap().nearest.as_deref(), Some("default"));
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
    assert!(err.contains("can't read the task field"), "{err}");
}

#[test]
fn ls_rules_import_views_still_catch_a_view_cycle() {
    // A rule importing one side of a view cycle must not mask the existing
    // (unextended) `config/views.rs` cycle check: rules can only ever be a sink
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
fn rejects_unknown_ls_key() {
    assert!(parse_config("[ls]\ndefult-filter = \"!terminal\"\n").is_err());
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
fn ls_columns_parses_when() {
    let config = parse_config(
        r"
[ls.columns.priority]
when = 'true'
",
    )
    .unwrap();
    assert_eq!(config.ls.columns["priority"].when.as_deref(), Some("true"));
    assert!(config.validate().is_ok());
}

#[test]
fn ls_columns_rejects_unknown_column_on_validate() {
    let config = parse_config(
        r"
[ls.columns.nope]
when = 'false'
",
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
fn ls_hyperlinks_defaults_to_never() {
    let config = parse_config("").unwrap();
    assert_eq!(config.ls.hyperlinks, crate::HyperlinkChoice::Never);
}

#[test]
fn ls_hyperlinks_parses_auto_and_always() {
    let auto = parse_config("[ls]\nhyperlinks = \"auto\"\n").unwrap();
    assert_eq!(auto.ls.hyperlinks, crate::HyperlinkChoice::Auto);
    let always = parse_config("[ls]\nhyperlinks = \"always\"\n").unwrap();
    assert_eq!(always.ls.hyperlinks, crate::HyperlinkChoice::Always);
}

#[test]
fn ls_hyperlink_target_absent_means_none() {
    let config = parse_config("").unwrap();
    assert!(config.ls.hyperlink_target.is_none());
}

#[test]
fn ls_hyperlink_target_parses_custom_template() {
    let config = parse_config("[ls]\nhyperlink-target = \"vscode://file/{abs}\"\n").unwrap();
    assert_eq!(
        config.ls.hyperlink_target.as_deref(),
        Some("vscode://file/{abs}")
    );
}
