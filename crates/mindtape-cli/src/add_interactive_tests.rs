//! Tests for the interactive editor fill: scaffold rendering and parse-back.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::{AddConfig, InteractiveMode, RawValue, discover_slots};

use super::{editor_command, open_in_editor, parse_buffer, scaffold, should_open};

const TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title"),
  priority: slot("priority", kind: "int", default: 5),
  tags: slot("tags", kind: "list", default: ()),
)

== Summary

#slot("summary", kind: "text", prompt: "One-paragraph summary")
"#;

fn specs() -> Vec<mindtape::SlotSpec> {
    discover_slots(TEMPLATE)
}

#[test]
fn scaffold_emits_one_entry_per_slot_with_comments() {
    let buffer = scaffold(&AddConfig::default(), &specs(), &BTreeMap::new());
    // Prompt and kind annotations.
    assert!(
        buffer.contains("# Task title [string, required]"),
        "{buffer}"
    );
    assert!(
        buffer.contains("# One-paragraph summary [text, required]"),
        "{buffer}"
    );
    // Default shown for the optional int slot.
    assert!(buffer.contains("# default: 5"), "{buffer}");
    assert!(buffer.contains("priority: 5"), "{buffer}");
    // Required title left blank (trailing space at the value position); text
    // slot gets a block scalar.
    assert!(buffer.contains("title: \n"), "{buffer}");
    assert!(buffer.contains("summary: |-"), "{buffer}");
}

#[test]
fn scaffold_tail_is_fillable_with_no_trailing_blank() {
    let specs = discover_slots("#show: task.with(title: slot(\"title\"))");
    let buffer = scaffold(&AddConfig::default(), &specs, &BTreeMap::new());
    assert!(!buffer.ends_with('\n'), "no trailing newline: {buffer:?}");
    assert!(
        buffer.ends_with("title: "),
        "ends at value position: {buffer:?}"
    );
}

#[test]
fn scaffold_custom_header_is_comment_prefixed() {
    let config = AddConfig {
        header: Some("My note\nSecond line".to_owned()),
        ..Default::default()
    };
    let specs = discover_slots("#show: task.with(title: slot(\"title\"))");
    let buffer = scaffold(&config, &specs, &BTreeMap::new());
    assert!(
        buffer.starts_with("# My note\n# Second line\n\n# "),
        "{buffer}"
    );
}

#[test]
fn scaffold_empty_header_disables_it() {
    let config = AddConfig {
        header: Some(String::new()),
        ..Default::default()
    };
    let specs = discover_slots("#show: task.with(title: slot(\"title\"))");
    let buffer = scaffold(&config, &specs, &BTreeMap::new());
    assert!(buffer.starts_with("# title ["), "no header block: {buffer}");
}

#[test]
fn scaffold_prefills_from_seed() {
    let mut seed = BTreeMap::new();
    seed.insert("title".to_owned(), RawValue::Str("Hello".to_owned()));
    seed.insert("priority".to_owned(), RawValue::Int(8));
    let buffer = scaffold(&AddConfig::default(), &specs(), &seed);
    assert!(buffer.contains("title: \"Hello\""), "{buffer}");
    assert!(buffer.contains("priority: 8"), "{buffer}");
}

#[test]
fn parse_back_yields_typed_values() {
    let buffer = "\
title: \"Fix the parser\"
priority: 8
tags:
  - backend
  - parser
summary: |-
  Line one
  Line two
";
    let values = parse_buffer(buffer).unwrap();
    assert_eq!(values["title"], RawValue::Str("Fix the parser".to_owned()));
    assert_eq!(values["priority"], RawValue::Int(8));
    assert_eq!(
        values["tags"],
        RawValue::List(vec!["backend".to_owned(), "parser".to_owned()])
    );
    assert_eq!(
        values["summary"],
        RawValue::Str("Line one\nLine two".to_owned())
    );
}

#[test]
fn parse_back_drops_empty_and_null() {
    let buffer = "\
title: \"Kept\"
priority:
summary: |-
";
    let values = parse_buffer(buffer).unwrap();
    assert!(values.contains_key("title"));
    assert!(!values.contains_key("priority"), "null dropped");
    assert!(!values.contains_key("summary"), "empty block dropped");
}

#[test]
fn round_trip_scaffold_then_parse() {
    let mut seed = BTreeMap::new();
    seed.insert("title".to_owned(), RawValue::Str("Round trip".to_owned()));
    seed.insert(
        "tags".to_owned(),
        RawValue::List(vec!["a".to_owned(), "b".to_owned()]),
    );
    let buffer = scaffold(&AddConfig::default(), &specs(), &seed);
    let values = parse_buffer(&buffer).unwrap();
    assert_eq!(values["title"], RawValue::Str("Round trip".to_owned()));
    assert_eq!(values["priority"], RawValue::Int(5)); // default pre-fill survives
    assert_eq!(
        values["tags"],
        RawValue::List(vec!["a".to_owned(), "b".to_owned()])
    );
}

#[test]
fn parse_back_rejects_non_mapping() {
    assert!(parse_buffer("- a\n- b\n").is_err());
}

#[test]
fn editor_command_prefers_override() {
    assert_eq!(editor_command(Some("nvim -p")), "nvim -p");
    assert_eq!(editor_command(Some("  ")), fallback_editor());
}

fn fallback_editor() -> String {
    // VISUAL/EDITOR may be set in the test environment; accept any non-empty.
    let resolved = editor_command(None);
    assert!(!resolved.is_empty());
    resolved
}

#[test]
fn open_in_editor_runs_command_and_reads_back() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("fake-editor.sh");
    // The fake editor rewrites its file argument ($1) with fixed YAML.
    std::fs::write(&script, "printf 'title: \"Edited\"\\n' > \"$1\"\n").unwrap();
    let command = format!("bash {}", script.display());
    let edited = open_in_editor(&command, "title:\n").unwrap();
    assert!(edited.contains("title: \"Edited\""), "{edited}");
}

#[test]
fn never_mode_without_force_stays_closed() {
    let config = AddConfig {
        interactive: InteractiveMode::Never,
        ..Default::default()
    };
    // No terminal in tests; should_open must not error and must stay false.
    assert!(!should_open(false, &config, &specs(), &BTreeMap::new(), false).unwrap());
}

#[test]
fn always_mode_errors_without_tty() {
    let config = AddConfig {
        interactive: InteractiveMode::Always,
        ..Default::default()
    };
    assert!(should_open(false, &config, &specs(), &BTreeMap::new(), false).is_err());
}

#[test]
fn always_mode_succeeds_with_tty() {
    let config = AddConfig {
        interactive: InteractiveMode::Always,
        ..Default::default()
    };
    assert!(should_open(false, &config, &specs(), &BTreeMap::new(), true).unwrap());
}
