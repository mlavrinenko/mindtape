//! `[[on.add]]` shapes: the `when` forms, the shared writer's field
//! whitelist, the `dir` template and the `dir` filter field.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use crate::config::Config;
use crate::config::add_hooks::{
    AddValue, DirTemplate, DirValue, desugar_add_when, template_names, validate_add_hooks,
    validate_component, validate_derived_dir, when_templates,
};
use crate::config::parse_config;
use crate::model::Task;
use crate::model::fields::{self, FieldValue};
use crate::test_support::{tempdir, write_file};

/// A project root, with `template` written under the default template dir.
fn project(template: Option<&str>) -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir();
    if let Some(name) = template {
        write_file(dir.path(), format!(".mindtape/templates/{name}.typ"), "");
    }
    let root = dir.path().to_path_buf();
    (dir, root)
}

/// The config from `on_add`, validated against a root holding `template`.
fn validate(on_add: &str, template: Option<&str>) -> Result<(), String> {
    let (_dir, root) = project(template);
    let config = parse_config(on_add).unwrap();
    validate_add_hooks(&config, &root)
}

/// Assert the config loads.
fn accepts(on_add: &str, template: Option<&str>) {
    if let Err(err) = validate(on_add, template) {
        panic!("expected the config to load: {err}");
    }
}

/// Assert the config is refused with an error containing `needle`.
fn refuses(on_add: &str, template: Option<&str>, needle: &str) {
    match validate(on_add, template) {
        Ok(()) => panic!("expected a config error containing {needle:?}"),
        Err(err) => assert!(err.contains(needle), "{err}"),
    }
}

/// One unnamed hook with `when` and one action.
fn hook(when: &str, action: &str) -> String {
    format!("[[on.add]]\nwhen = '{when}'\nactions = [{action}]\n")
}

#[test]
fn a_valid_hook_keeps_its_shape_and_passes() {
    let text = "[tasks]\nroots = [\"tasks\"]\n\
                [[on.add]]\nname = \"fixes\"\nwhen = 'template == \"fix\"'\n\
                actions = [{ set = { due = \"2026-12-01\", priority = 7 }, \
                append = { tags = [\"backend\"] }, dir = \"tasks/{template}\" }]\n";
    let (_dir, root) = project(Some("fix"));
    let config = parse_config(text).unwrap();

    assert_eq!(config.on.add.len(), 1);
    let entry = &config.on.add[0];
    assert_eq!(entry.name.as_deref(), Some("fixes"));
    assert!(matches!(entry.actions[0].set["priority"], AddValue::Int(7)));
    assert!(validate_add_hooks(&config, &root).is_ok());
}

#[test]
fn action_fields_are_checked_by_the_shared_writer() {
    for (action, needle) in [
        (
            "{ set = { nope = \"x\" } }",
            "on.add[0].actions[0].set.nope: unknown writable field",
        ),
        (
            "{ append = { due = [\"2026-12-01\"] } }",
            "on.add[0].actions[0].append.due: += appends to a list field",
        ),
        (
            "{ append = { tags = [] } }",
            "on.add[0].actions[0].append.tags: append needs at least one item",
        ),
        (
            "{ set = { dir = \"tasks\" } }",
            "on.add[0].actions[0].set.dir: dir is a destination, not a field",
        ),
        (
            "{ set = { tags = [\"a\"] }, append = { tags = [\"b\"] } }",
            "on.add[0].actions[0]: tags is in both set and append",
        ),
        ("{}", "set, append, derive or dir is required"),
    ] {
        refuses(&hook("", action), None, needle);
    }
}

#[test]
fn a_hook_needs_an_action() {
    refuses(
        "[[on.add]]\nwhen = ''\n",
        None,
        "on.add[0]: no actions declared",
    );
}

#[test]
fn both_template_forms_and_operators_are_valid() {
    for when in [
        "template == \"fix\"",
        "template != \"fix\"",
        "from_template(\"fix\")",
        "template == \"task\" && !terminal",
        "title == \"template\"",
    ] {
        accepts(&hook(when, "{ set = { kind = \"fix\" } }"), Some("fix"));
    }
}

#[test]
fn malformed_template_tests_are_refused() {
    for (when, needle) in [
        ("template", "on.add[0].when: bare `template`"),
        ("template > \"fix\"", "only == and != compare template"),
        (
            "template == kind",
            "template compares with a string literal",
        ),
        (
            "from_template(kind)",
            "from_template(...) takes a template name in double quotes",
        ),
        ("from_template(\"nope\")", "unknown template: 'nope'"),
    ] {
        refuses(
            &hook(when, "{ set = { kind = \"fix\" } }"),
            Some("fix"),
            needle,
        );
    }
}

/// The draft the `when` reads has no identity or place yet: the add files the
/// task after every hook ran, so those fields would answer for a synthetic
/// draft rather than the task being added.
#[test]
fn add_unsafe_fields_are_refused_in_a_when() {
    for (when, field) in [
        (r#"dir == "tasks/fixes""#, "dir"),
        (r#"id == "x""#, "id"),
        ("derived.changes > 0", "derived"),
        ("ready", "ready"),
        ("ready()", "ready"),
        ("!blocked()", "blocked"),
    ] {
        refuses(
            &hook(when, "{ set = { kind = \"fix\" } }"),
            None,
            &format!(
                "on.add[0].when: an add when cannot read `{field}`: \
                 it is not known until the task is filed"
            ),
        );
    }
}

#[test]
fn a_when_view_that_reads_an_unsafe_field_is_refused() {
    let text = "[views.fixes]\nfilter = 'dir == \"tasks/fixes\"'\n\n\
                [[on.add]]\nwhen = '@fixes'\n\
                actions = [{ set = { kind = \"fix\" } }]\n";
    let (_dir, root) = project(None);
    let config = parse_config(text).unwrap();
    let err = validate_add_hooks(&config, &root).unwrap_err();
    assert!(
        err.contains("on.add[0].when: an add when cannot read `dir`"),
        "{err}"
    );
}

#[test]
fn a_when_may_read_authored_fields_and_the_old_draft() {
    for when in [
        r#"kind == "fix""#,
        r#"title == "Fix""#,
        r#"old("kind") == "fix""#,
    ] {
        accepts(&hook(when, "{ set = { kind = \"fix\" } }"), None);
    }
}

/// `@` is not an identifier character, so a view reference named `template`
/// must not read as a bare `template` test.
#[test]
fn a_view_named_template_is_no_template_test() {
    let text = "[views.template]\nfilter = 'kind == \"fix\"'\n\n\
                [[on.add]]\nwhen = '@template'\n\
                actions = [{ set = { kind = \"fix\" } }]\n";
    let (_dir, root) = project(None);
    let config = parse_config(text).unwrap();
    if let Err(err) = validate_add_hooks(&config, &root) {
        panic!("expected @template to load as a view reference: {err}");
    }
}

#[test]
fn dir_templates_are_checked() {
    refuses(
        &hook("", "{ dir = \"tasks/{nope}\" }"),
        None,
        "on.add[0].actions[0].dir: unknown dir macro: {nope}",
    );
    for absolute in ["/tasks", "C:\\tasks"] {
        refuses(
            &hook("", &format!("{{ dir = \"{absolute}\" }}")),
            None,
            "dir must be relative to the project root",
        );
    }
    for (bad, needle) in [
        ("../outside", "dir must not hold `.` or `..`"),
        ("tasks/./bug", "dir must not hold `.` or `..`"),
        ("tasks//bug", "dir holds an empty path component"),
    ] {
        refuses(&hook("", &format!("{{ dir = \"{bad}\" }}")), None, needle);
    }
    accepts(&hook("", "{ dir = \"\" }"), None);
}

#[test]
fn dir_reads_the_parent_of_path() {
    let config = Config::default();
    let def = fields::get("dir").unwrap();
    let mut task = Task {
        file: "tasks/fixes/a.typ".into(),
        ..Task::default()
    };

    assert_eq!(
        (def.get)(&task, &config),
        FieldValue::Str("tasks/fixes".to_owned())
    );

    task.file = "a.typ".into();
    assert_eq!((def.get)(&task, &config), FieldValue::Str(String::new()));
}

#[test]
fn template_names_lists_the_builtin_and_every_configured_stem() {
    let (_dir, root) = project(Some("fix"));
    assert_eq!(template_names(&Config::default(), &root), ["fix", "task"]);
}

#[test]
fn dir_macros_expand_to_validated_components() {
    let template = DirTemplate::parse("tasks/{kind}/{title}").unwrap();
    assert_eq!(template.macros().collect::<Vec<_>>(), ["kind", "title"]);
    let value = |name: &str| match name {
        "kind" => Some("bug".to_owned()),
        "title" => Some("login".to_owned()),
        _ => None,
    };
    assert_eq!(template.expand(value).unwrap(), "tasks/bug/login");

    let bad = |_: &str| Some("a/b".to_owned());
    assert!(template.expand(bad).unwrap_err().contains("separator"));
    let missing = |_: &str| None;
    assert!(template.expand(missing).unwrap_err().contains("no value"));

    for bad in ["", ".", ".."] {
        assert!(validate_component(bad).is_err(), "{bad:?}");
    }
    assert!(validate_component("ok").is_ok());
}

#[test]
fn a_derive_needs_a_writable_field_and_a_loadable_function() {
    let dir = tempdir();
    let root = dir.path().to_path_buf();
    write_file(&root, "add.typ", "#let classify(draft) = \"bug\"\n");
    let derive = |body: &str| {
        let config = parse_config(&hook("", body)).unwrap();
        validate_add_hooks(&config, &root)
    };

    for good in [
        "{ derive = { typst = \"add.typ\", function = \"classify\", field = \"kind\" } }",
        "{ dir = { derive = { typst = \"add.typ\", function = \"classify\" } } }",
    ] {
        if let Err(err) = derive(good) {
            panic!("expected {good} to load: {err}");
        }
    }
    for (bad, needle) in [
        (
            "{ derive = { typst = \"add.typ\", function = \"classify\" } }",
            "on.add[0].actions[0].derive.field: a derive action needs a field",
        ),
        (
            "{ derive = { typst = \"add.typ\", function = \"classify\", field = \"nope\" } }",
            "derive.field: unknown writable field: nope",
        ),
        (
            "{ derive = { typst = \"add.typ\", function = \"ghost\", field = \"kind\" } }",
            "does not export `ghost`",
        ),
        (
            "{ derive = { typst = \"ghost.typ\", function = \"classify\", field = \"kind\" } }",
            "`ghost.typ` does not exist",
        ),
        (
            "{ derive = { typst = \"add.typ\", function = \"classify\", field = \"kind\" }, \
             set = { kind = \"bug\" } }",
            "on.add[0].actions[0]: kind is in both derive and set",
        ),
        (
            "{ dir = { derive = { typst = \"ghost.typ\", function = \"classify\" } } }",
            "on.add[0].actions[0].dir.derive: `ghost.typ` does not exist",
        ),
    ] {
        match derive(bad) {
            Ok(()) => panic!("expected a config error containing {needle:?}"),
            Err(err) => assert!(err.contains(needle), "{err}"),
        }
    }
}

#[test]
fn a_dir_derive_parses_and_takes_no_field() {
    let table = "[[on.add]]\nactions = [{ dir = { derive = { typst = \"add.typ\", function = \"f\" } } }]\n";
    let config = parse_config(table).unwrap();
    assert!(matches!(
        config.on.add[0].actions[0].dir,
        Some(DirValue::Derived { .. })
    ));
    let with_field = "[[on.add]]\nactions = [{ dir = { derive = { typst = \"a.typ\", function = \"f\", field = \"kind\" } } }]\n";
    assert!(parse_config(with_field).is_err());
}

#[test]
fn a_derived_dir_is_relative_and_holds_no_dot_components() {
    for good in ["", "tasks", "tasks/bug"] {
        assert!(validate_derived_dir(good).is_ok(), "{good:?}");
    }
    for (bad, needle) in [
        ("/tasks", "relative"),
        ("/tasks/bug", "relative"),
        ("../up", "`.` or `..`"),
        ("tasks/./bug", "`.` or `..`"),
        ("tasks//bug", "empty path component"),
    ] {
        let err = validate_derived_dir(bad).unwrap_err();
        assert!(err.contains(needle), "{bad:?}: {err}");
    }
}

#[test]
fn desugar_rewrites_both_forms_to_constants() {
    use crate::config::hooks::{ALWAYS, NEVER};

    for (when, template, expected) in [
        ("from_template(\"fix\")", "fix", ALWAYS),
        ("template != \"fix\"", "fix", NEVER),
        ("template == \"fix\"", "task", NEVER),
    ] {
        assert_eq!(
            desugar_add_when(when, template).unwrap(),
            expected,
            "{when}"
        );
    }
}

#[test]
fn when_templates_keeps_source_order_across_both_forms() {
    assert_eq!(
        when_templates(r#"template == "zzz" && from_template("yyy")"#).unwrap(),
        ["zzz", "yyy"]
    );
    assert_eq!(
        when_templates(r#"from_template("yyy") || template != "zzz""#).unwrap(),
        ["yyy", "zzz"]
    );
}

/// `-` and `.` continue an identifier in the filter language, so a field
/// named `x-template` or a `derived.template` value is no `template` test.
#[test]
fn a_field_name_ending_in_template_is_not_a_template_test() {
    for when in [
        r#"x-template == "other" && template == "fix""#,
        r#"derived.template == "other" && template == "fix""#,
    ] {
        assert_eq!(when_templates(when).unwrap(), ["fix"], "{when}");
    }
}

#[test]
fn add_values_keep_their_toml_shape() {
    let config =
        parse_config("[[on.add]]\nactions = [{ set = { priority = 1.5, title = true } }]\n")
            .unwrap();
    let set = &config.on.add[0].actions[0].set;
    assert!(matches!(
        set["priority"].raw(),
        crate::template::RawValue::Float(_)
    ));
    assert!(matches!(
        set["title"].raw(),
        crate::template::RawValue::Bool(true)
    ));
}

#[test]
fn dir_value_reads_each_form() {
    let path = DirValue::Path("tasks/{kind}".to_owned());
    assert_eq!(path.template(), Some("tasks/{kind}"));
    assert!(path.derived().is_none());

    let derived = DirValue::Derived {
        derive: super::DerivedDir {
            typst: "add.typ".to_owned(),
            function: "classify".to_owned(),
        },
    };
    assert_eq!(derived.template(), None);
    assert_eq!(derived.derived().unwrap().function, "classify");
}

#[test]
fn writer_and_filter_errors_are_validation_errors() {
    refuses(
        &hook("", "{ set = { due = \"soon\" } }"),
        None,
        "a YYYY-MM-DD date is expected",
    );
    refuses(
        &hook("", "{ append = { nope = [\"x\"] } }"),
        None,
        "unknown writable field",
    );
    refuses(
        &hook("bogus == \"x\"", "{ set = { kind = \"fix\" } }"),
        None,
        "unknown field",
    );
    refuses(
        &hook(
            "",
            "{ derive = { typst = \"add.typ\", function = \"f\", field = \"tags\" }, \
             append = { tags = [\"x\"] } }",
        ),
        None,
        "tags is in both derive and append",
    );
}

#[test]
fn a_malformed_when_reaches_the_validation_error() {
    for (when, needle) in [
        (
            "from_template(kind)",
            "from_template(...) takes a template name",
        ),
        ("kind ==", "the filter is incomplete"),
    ] {
        refuses(&hook(when, "{ set = { kind = \"fix\" } }"), None, needle);
    }
}

#[test]
fn an_empty_dir_macro_is_refused() {
    let err = DirTemplate::parse("tasks/{}").unwrap_err();
    assert!(err.contains("empty `{}` macro"), "{err}");
}

#[test]
fn template_comparison_scanner_stops_at_a_longer_name() {
    assert_eq!(
        desugar_add_when(r#"templateX == "fix""#, "fix").unwrap(),
        r#"templateX == "fix""#
    );
    let err = when_templates(r#"template == "unterminated"#).unwrap_err();
    assert!(err.contains("string literal"), "{err}");
}

#[test]
fn a_when_through_a_view_cycle_is_refused_not_looped() {
    let text = "[views.a]\nfilter = '@b'\n\n[views.b]\nfilter = '@a'\n\n\
                [[on.add]]\nwhen = '@a'\nactions = [{ set = { kind = \"fix\" } }]\n";
    let (_dir, root) = project(None);
    let config = parse_config(text).unwrap();
    let err = validate_add_hooks(&config, &root).unwrap_err();
    assert!(err.contains("nested too deep"), "{err}");
}
