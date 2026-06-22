//! Tests for `add` value sources and template helpers.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use mindtape::{Config, RawValue, discover_slots};

use super::{
    BUILTIN_TASK, DescribeFormat, collect_values, describe, env_var_name, layer_env,
    list_templates, resolve_template, slug, stdin_title,
};

#[test]
fn slug_lowercases_and_collapses() {
    assert_eq!(slug("Wire the Add Command!"), "wire-the-add-command");
    assert_eq!(slug("  spaced  out  "), "spaced-out");
    assert_eq!(slug("Café 99"), "caf-99");
}

#[test]
fn stdin_title_takes_first_nonblank_line() {
    assert_eq!(stdin_title("hello world\n"), Some("hello world".to_owned()));
    assert_eq!(stdin_title("  spaced  \n"), Some("spaced".to_owned()));
    assert_eq!(stdin_title("\n\nfirst\nsecond\n"), Some("first".to_owned()));
    assert_eq!(stdin_title(""), None);
    assert_eq!(stdin_title("   \n\t\n"), None);
}

#[test]
fn env_var_name_uppercases_and_underscores() {
    assert_eq!(env_var_name("title"), "MINDTAPE_ADD_TITLE");
    assert_eq!(env_var_name("depends-on"), "MINDTAPE_ADD_DEPENDS_ON");
}

#[test]
fn layer_env_fills_unset_skips_seeded_and_blank() {
    let specs = discover_slots(BUILTIN_TASK); // the barebones template: one `title` slot
    let lookup = |key: &str| (key == "MINDTAPE_ADD_TITLE").then(|| "From Env".to_owned());

    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, lookup);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From Env".to_owned()))
    );

    // A value already set by --set/-f is never overwritten by env.
    let mut seeded = BTreeMap::new();
    seeded.insert("title".to_owned(), RawValue::Str("Explicit".to_owned()));
    layer_env(&mut seeded, &specs, lookup);
    assert_eq!(
        seeded.get("title"),
        Some(&RawValue::Str("Explicit".to_owned()))
    );

    // A blank env value sets nothing.
    let mut blank = BTreeMap::new();
    layer_env(&mut blank, &specs, |_| Some("   ".to_owned()));
    assert!(blank.is_empty());
}

#[test]
fn collect_set_overrides_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v.json");
    std::fs::write(&path, r#"{"title":"From JSON","impact":8,"flag":true}"#).unwrap();
    let values = collect_values(
        &[path.to_string_lossy().into_owned()],
        &["title=Override".to_owned()],
    )
    .unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Override".to_owned()))
    );
    assert_eq!(values.get("impact"), Some(&RawValue::Int(8)));
    assert_eq!(values.get("flag"), Some(&RawValue::Bool(true)));
}

#[test]
fn collect_reads_json_array_as_list() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v.json");
    std::fs::write(&path, r#"{"tags":["a","b"]}"#).unwrap();
    let values = collect_values(&[path.to_string_lossy().into_owned()], &[]).unwrap();
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec!["a".to_owned(), "b".to_owned()]))
    );
}

#[test]
fn collect_rejects_non_object_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v.json");
    std::fs::write(&path, "[1,2,3]").unwrap();
    assert!(collect_values(&[path.to_string_lossy().into_owned()], &[]).is_err());
}

#[test]
fn describe_format_parses() {
    assert_eq!(DescribeFormat::parse("text").unwrap(), DescribeFormat::Text);
    assert_eq!(DescribeFormat::parse("json").unwrap(), DescribeFormat::Json);
    assert!(DescribeFormat::parse("yaml").is_err());
}

#[test]
fn describe_json_lists_builtin_slots() {
    let specs = discover_slots(BUILTIN_TASK);
    let json = describe(&specs, DescribeFormat::Json).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let names: Vec<&str> = parsed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .collect();
    // The built-in template is barebones: a single required `title` slot. Other
    // core fields surface only when the project config requires them.
    assert_eq!(names, vec!["title"]);
}

#[test]
fn describe_text_marks_required() {
    let specs = discover_slots(BUILTIN_TASK);
    let text = describe(&specs, DescribeFormat::Text).unwrap();
    assert!(
        text.lines()
            .any(|line| line.contains("title") && line.contains("required"))
    );
}

#[test]
fn resolve_falls_back_to_builtin_task() {
    let dir = tempfile::tempdir().unwrap();
    let source = resolve_template(&Config::default(), dir.path(), "task").unwrap();
    assert!(source.contains("task.with"));
}

#[test]
fn resolve_unknown_template_errors() {
    let dir = tempfile::tempdir().unwrap();
    assert!(resolve_template(&Config::default(), dir.path(), "bug").is_err());
}

#[test]
fn list_includes_builtin_and_local() {
    let dir = tempfile::tempdir().unwrap();
    let templates = dir.path().join(".mindtape/templates");
    std::fs::create_dir_all(&templates).unwrap();
    std::fs::write(templates.join("bug.typ"), "x").unwrap();
    let names = list_templates(&Config::default(), dir.path());
    assert!(names.contains(&"task".to_owned()));
    assert!(names.contains(&"bug".to_owned()));
}
