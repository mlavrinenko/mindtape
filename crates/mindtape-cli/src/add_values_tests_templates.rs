//! Tests for `add` template helpers: filename slugging, stdin title
//! extraction, `--describe` rendering, and template resolution/listing.
//! Value-source merging (`-f`/`--set`/env, JSON/YAML dispatch) lives in
//! `add_values_tests.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::{Config, discover_slots};

use super::{
    BUILTIN_TASK, DescribeFormat, describe, list_templates, resolve_template, slug, stdin_title,
};

#[test]
fn slug_lowercases_and_collapses() {
    assert_eq!(slug("Wire the Add Command!"), "wire-the-add-command");
    assert_eq!(slug("  spaced  out  "), "spaced-out");
    // Unicode alphanumerics are kept and lowercased, not dropped.
    assert_eq!(slug("Café 99"), "café-99");
    assert_eq!(slug("помыть посуду"), "помыть-посуду");
    assert_eq!(slug("施工 任务"), "施工-任务");
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

/// An optional slot with no default is labeled "optional", not "required" —
/// distinguishes the `&&` operator from `||`.
#[test]
fn describe_text_marks_optional_correctly() {
    use mindtape::SlotKind;

    let specs = [mindtape::SlotSpec {
        name: "my-slot".into(),
        kind: SlotKind::Text,
        required: false,
        default: None,
        prompt: None,
        env: None,
        choices: None,
        default_string: None,
        markup: false,
        unknown_kind: None,
        range: 0..0,
    }];
    let text = super::describe(&specs, super::DescribeFormat::Text).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].contains("optional"),
        "non-required slot with no default should be 'optional', got: {}",
        lines[0]
    );
    // Sanity: it should NOT say "required".
    assert!(
        !lines[0].contains("required"),
        "non-required slot must not be labeled 'required'"
    );
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
fn describe_surfaces_declared_env() {
    let specs =
        discover_slots(r#"#f(t: slot("title", env: "MINDTAPE_PRINCIPAL", default: "untitled"))"#);

    let text = describe(&specs, DescribeFormat::Text).unwrap();
    assert!(
        text.lines()
            .any(|line| line.contains("env: MINDTAPE_PRINCIPAL")),
        "text describe should name the env variable, got: {text}"
    );

    let json = describe(&specs, DescribeFormat::Json).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.as_array().unwrap()[0]["env"], "MINDTAPE_PRINCIPAL");

    // A slot without `env` stays quiet in both formats.
    let plain = discover_slots(BUILTIN_TASK);
    let text = describe(&plain, DescribeFormat::Text).unwrap();
    assert!(!text.contains("env:"));
    let json = describe(&plain, DescribeFormat::Json).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.as_array().unwrap()[0]["env"].is_null());
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
