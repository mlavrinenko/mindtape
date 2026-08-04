//! Tests for `add` value sources and template helpers.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::{Config, RawValue, discover_slots};

use super::{
    BUILTIN_TASK, DescribeFormat, collect_values, describe, env_var_name, layer_env,
    list_templates, parse_sniffed_source, parse_source, reject_unknown_keys, resolve_template,
    slug, stdin_title,
};

/// Write `text` to `dir`/`name` and return the JSON/YAML `-f` value:
/// `collect_values` reads a path from disk, so extension-driven dispatch
/// needs a real file, not an in-memory string.
fn values_file(dir: &std::path::Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

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
fn layer_env_reads_declared_env_name() {
    let specs =
        discover_slots(r#"#f(t: slot("title", env: "MINDTAPE_PRINCIPAL", default: "untitled"))"#);
    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, |key| {
        (key == "MINDTAPE_PRINCIPAL").then(|| "From Declared Env".to_owned())
    });
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From Declared Env".to_owned()))
    );
}

#[test]
fn layer_env_declared_env_unset_falls_through() {
    let specs =
        discover_slots(r#"#f(t: slot("title", env: "MINDTAPE_PRINCIPAL", default: "untitled"))"#);
    // The declared variable is unset: nothing fills, and the implicit
    // `MINDTAPE_ADD_TITLE` is not consulted — a declared `env` replaces it.
    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, |key| {
        (key == "MINDTAPE_ADD_TITLE").then(|| "Implicit".to_owned())
    });
    assert!(values.is_empty());
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
fn collect_set_brackets_accumulate_into_list() {
    let values = collect_values(
        &[],
        &[
            "tags[]=a".to_owned(),
            "tags[]=b".to_owned(),
            "tags[]=b,c".to_owned(),
        ],
    )
    .unwrap();
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec![
            "a".to_owned(),
            "b".to_owned(),
            "b,c".to_owned(), // a literal comma in a value is never split
        ]))
    );
}

#[test]
fn collect_set_brackets_append_to_json_list() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v.json");
    std::fs::write(&path, r#"{"tags":["a"]}"#).unwrap();
    let values = collect_values(
        &[path.to_string_lossy().into_owned()],
        &["tags[]=b".to_owned()],
    )
    .unwrap();
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec!["a".to_owned(), "b".to_owned()]))
    );
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

// ── YAML `-f` dispatch: extension, sniff, malformed input, tabs, blanks ──

#[test]
fn dot_yaml_extension_parses_as_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "title: From YAML\nimpact: 8\n");
    let values = collect_values(&[path], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From YAML".to_owned()))
    );
    assert_eq!(values.get("impact"), Some(&RawValue::Int(8)));
}

#[test]
fn dot_yml_extension_parses_as_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yml", "title: From Yml\n");
    let values = collect_values(&[path], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From Yml".to_owned()))
    );
}

#[test]
fn sniff_picks_yaml_for_non_brace_content() {
    // No filename extension to trust (the stdin `-` shape): a document with
    // no leading `{`/`[` sniffs as YAML.
    let values = parse_sniffed_source("-", "title: Sniffed\ntags:\n  - a\n  - b\n").unwrap();
    assert_eq!(
        values.get("title"),
        Some(&Some(RawValue::Str("Sniffed".to_owned())))
    );
    assert_eq!(
        values.get("tags"),
        Some(&Some(RawValue::List(vec!["a".to_owned(), "b".to_owned()])))
    );
}

#[test]
fn sniff_picks_json_for_brace_content() {
    let values = parse_sniffed_source("-", r#"{"title":"Sniffed JSON"}"#).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&Some(RawValue::Str("Sniffed JSON".to_owned())))
    );
}

#[test]
fn unknown_extension_sniffs_like_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.values", "title: Unknown Extension\n");
    let values = collect_values(&[path], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Unknown Extension".to_owned()))
    );
}

#[test]
fn malformed_yaml_extension_reports_a_useful_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "title: [unterminated\n");
    let err = collect_values(std::slice::from_ref(&path), &[])
        .unwrap_err()
        .to_string();
    assert!(err.contains(path.as_str()), "{err}");
}

#[test]
fn malformed_sniffed_input_reports_the_json_error() {
    // Leading `{` sniffs as JSON; when both the JSON and the YAML retry fail,
    // the JSON parse error is the one surfaced (see `parse_sniffed_source`).
    let err = parse_sniffed_source("-", "{not valid anything")
        .unwrap_err()
        .to_string();
    assert!(err.contains("parse JSON from -"), "{err}");
}

#[test]
fn tab_indented_json_still_parses() {
    // libyaml rejects tab indentation; a `.json`-dispatched source must keep
    // using serde_json, which accepts it.
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.json", "{\n\t\"title\": \"Tabbed\"\n}\n");
    let values = collect_values(&[path], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Tabbed".to_owned()))
    );
}

#[test]
fn tab_indented_json_still_parses_when_sniffed() {
    let values = parse_sniffed_source("-", "{\n\t\"title\": \"Tabbed\"\n}\n").unwrap();
    assert_eq!(
        values.get("title"),
        Some(&Some(RawValue::Str("Tabbed".to_owned())))
    );
}

#[test]
fn empty_string_survives_a_yaml_values_file() {
    // Unlike the `--edit` scaffold (blank means "use the default"), a `-f`
    // file's `""` is a real value that must reach the fill map unchanged.
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "note: \"\"\n");
    let values = collect_values(&[path], &[]).unwrap();
    assert_eq!(values.get("note"), Some(&RawValue::Str(String::new())));
}

#[test]
fn empty_string_survives_a_json_values_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.json", r#"{"note":""}"#);
    let values = collect_values(&[path], &[]).unwrap();
    assert_eq!(values.get("note"), Some(&RawValue::Str(String::new())));
}

#[test]
fn yaml_non_mapping_root_is_rejected_with_json_shaped_wording() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "- a\n- b\n");
    let err = collect_values(std::slice::from_ref(&path), &[])
        .unwrap_err()
        .to_string();
    assert!(
        err.contains(&format!("values from {path} must be a mapping")),
        "{err}"
    );
}

#[test]
fn yaml_multi_document_stream_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "---\na: 1\n---\nb: 2\n");
    assert!(collect_values(&[path], &[]).is_err());
}

#[test]
fn yaml_extension_dispatch_matches_parse_source() {
    // `parse_source` is the single dispatch point `collect_values` uses per
    // source; pin its extension routing directly.
    let values = parse_source("v.yaml", "x: 1\n").unwrap();
    assert_eq!(values.get("x"), Some(&Some(RawValue::Int(1))));
}

#[test]
fn later_yaml_null_unsets_an_earlier_set_key() {
    let dir = tempfile::tempdir().unwrap();
    let first = values_file(dir.path(), "a.json", r#"{"title":"First","impact":8}"#);
    let second = values_file(dir.path(), "b.yaml", "impact: ~\n");
    let values = collect_values(&[first, second], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("First".to_owned()))
    );
    assert!(
        !values.contains_key("impact"),
        "a later explicit null must unset an earlier value: {values:?}"
    );
}

#[test]
fn reject_unknown_keys_allows_known_slots() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("Fine".to_owned()));
    assert!(reject_unknown_keys(&values, &specs).is_ok());
}

#[test]
fn reject_unknown_keys_errors_naming_the_key_and_known_slots() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("Fine".to_owned()));
    values.insert("priority".to_owned(), RawValue::Str("3".to_owned()));
    let err = reject_unknown_keys(&values, &specs)
        .unwrap_err()
        .to_string();
    assert!(err.contains("priority"), "{err}");
    assert!(err.contains("title"), "{err}");
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
