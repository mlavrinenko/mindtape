//! Tests for the scaffold rendering / parse-back round trip: YAML scaffold
//! generation from discovered slots, and parsing an edited buffer back into
//! typed fill values. Editor invocation and open/close gating live in
//! `cmd/add/interactive_tests_editor.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::{AddConfig, RawValue, discover_slots};

use super::{parse_buffer, scaffold};

pub(super) const TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title"),
  priority: slot("priority", kind: "int", default: 5),
  tags: slot("tags", kind: "list", default: ()),
)

== Summary

#slot("summary", kind: "text", prompt: "One-paragraph summary")
"#;

pub(super) fn specs() -> Vec<mindtape::SlotSpec> {
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
fn scaffold_sorts_required_slots_before_optional_preserving_relative_order() {
    // TEMPLATE order is title, priority, tags, summary; priority and tags have
    // defaults (optional), title and summary don't (required).
    let buffer = scaffold(&AddConfig::default(), &specs(), &BTreeMap::new());
    let title = buffer.find("title:").unwrap();
    let summary = buffer.find("summary:").unwrap();
    let priority = buffer.find("priority:").unwrap();
    let tags = buffer.find("tags:").unwrap();
    assert!(title < summary, "{buffer}");
    assert!(summary < priority, "{buffer}");
    assert!(priority < tags, "{buffer}");
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
fn scaffold_comments_false_strips_every_comment_but_stays_fillable() {
    let config = AddConfig {
        comments: false,
        // A custom header is ignored once comments are off.
        header: Some("ignored".to_owned()),
        ..Default::default()
    };
    let buffer = scaffold(&config, &specs(), &BTreeMap::new());
    assert!(
        !buffer
            .lines()
            .any(|line| line.trim_start().starts_with('#')),
        "no comment lines: {buffer}"
    );
    // Compact: no blank line separates the bare entries.
    assert!(
        !buffer.contains("\n\n"),
        "no blank lines between entries: {buffer:?}"
    );
    // Bare mapping is still present and parses back cleanly (defaults fill in).
    assert!(buffer.contains("priority: 5"), "{buffer}");
    assert!(buffer.contains("title: "), "{buffer}");
    parse_buffer(&buffer).unwrap();
}

#[test]
fn scaffold_comments_on_spaces_entries_with_blank_lines() {
    let buffer = scaffold(&AddConfig::default(), &specs(), &BTreeMap::new());
    assert!(
        buffer.contains("\n\n"),
        "comment groups stay spaced: {buffer:?}"
    );
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
fn scaffold_renders_empty_list_default_as_bracket_pair() {
    // TEMPLATE's tags slot has `default: ()` — an empty list. It should surface
    // as an explicit empty flow sequence, not a blank scalar.
    let buffer = scaffold(&AddConfig::default(), &specs(), &BTreeMap::new());
    assert!(buffer.contains("tags: []"), "{buffer}");
    // Left untouched, it parses back to an empty list (not dropped as null).
    let values = parse_buffer(&buffer).unwrap();
    assert_eq!(values["tags"], RawValue::List(Vec::new()));
}

#[test]
fn scaffold_leaves_non_empty_list_default_blank() {
    // A non-empty list default stays blank so an untouched slot falls through to
    // the default rather than being overwritten with an empty `[]`.
    let specs = discover_slots(
        "#show: task.with(tags: slot(\"tags\", kind: \"list\", default: (\"backend\",)))",
    );
    let buffer = scaffold(&AddConfig::default(), &specs, &BTreeMap::new());
    assert!(buffer.ends_with("tags: "), "left blank: {buffer:?}");
    assert!(!buffer.contains("tags: []"), "not bracketed: {buffer}");
    // The default is advertised in the comment.
    assert!(buffer.contains("# default:"), "{buffer}");
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

// ── commentify: either condition alone triggers the comment prefix ──

#[test]
fn custom_header_comment_lines_kept() {
    let config = AddConfig {
        header: Some("# Already a comment\nReal text".to_owned()),
        ..Default::default()
    };
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    let buffer = scaffold(&config, &specs, &BTreeMap::new());
    // Real: `# Already a comment` kept as-is (starts with # → kept).
    // Mutant: `false && true` → adds `# ` prefix → `# # Already a comment`.
    assert!(buffer.starts_with("# Already a comment"), "{buffer}");
}

// ── push_comment: both conditions must hold before a comment is pushed ──

#[test]
fn non_required_slot_label_is_optional() {
    let specs = discover_slots("#show: task.with(x: slot(\"x\", required: false))");
    let buffer = scaffold(&AddConfig::default(), &specs, &BTreeMap::new());
    // spec.required=false, default.is_none()=true → real: "optional", mutant: "required"
    assert!(buffer.contains("[string, optional]"), "{buffer}");
}

// ── seed_entry: the guard selecting the block-scalar form ──

#[test]
fn text_slot_seed_renders_as_block() {
    let mut seed = BTreeMap::new();
    seed.insert("x".to_owned(), RawValue::Str("hello\nworld".to_owned()));
    let specs = discover_slots("#show: task.with(x: slot(\"x\", kind: \"text\"))");
    let buffer = scaffold(&AddConfig::default(), &specs, &seed);
    // Real: Entry::Block renders `|-`.  Mutant (guard→false) renders quoted scalar.
    assert!(
        buffer.contains("|-"),
        "text seed produces block scalar: {buffer}"
    );
    // Mutant would render `x: "hello\nworld"`, which contains the substring
    // `"hello\nworld"` — the block scalar does not.
    assert!(
        !buffer.contains(r#""hello\nworld""#),
        "text seed must not be rendered as quoted scalar: {buffer}"
    );
}

// ── seed_entry: the Float and Bool arms render their typed scalars ──

#[test]
fn float_seed_renders_as_scalar() {
    let mut seed = BTreeMap::new();
    seed.insert("x".to_owned(), RawValue::Float(1.5));
    let specs = discover_slots("#show: task.with(x: slot(\"x\", kind: \"float\"))");
    let buffer = scaffold(&AddConfig::default(), &specs, &seed);
    assert!(buffer.contains("x: 1.5"), "{buffer}");
    // Round-trips back as a typed float, not a string.
    let values = parse_buffer(&buffer).unwrap();
    assert_eq!(values["x"], RawValue::Float(1.5));
}

#[test]
fn bool_seed_renders_as_scalar() {
    let mut seed = BTreeMap::new();
    seed.insert("x".to_owned(), RawValue::Bool(true));
    let specs = discover_slots("#show: task.with(x: slot(\"x\", kind: \"bool\"))");
    let buffer = scaffold(&AddConfig::default(), &specs, &seed);
    assert!(buffer.contains("x: true"), "{buffer}");
    let values = parse_buffer(&buffer).unwrap();
    assert_eq!(values["x"], RawValue::Bool(true));
}

// ── quote: every character that must be escaped, one case each ──

#[test]
fn quote_special_chars_roundtrip() {
    let mut seed = BTreeMap::new();
    // A string containing all four special chars: ", \, newline, tab.
    seed.insert("x".to_owned(), RawValue::Str("a\"b\\c\nd\te".to_owned()));
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    let buffer = scaffold(&AddConfig::default(), &specs, &seed);
    let values = parse_buffer(&buffer).unwrap();
    assert_eq!(values["x"], RawValue::Str("a\"b\\c\nd\te".to_owned()));
}

// ── quote: a tab is escaped, not passed through ──

#[test]
fn quote_escapes_tab_to_backslash_t() {
    let mut seed = BTreeMap::new();
    seed.insert("x".to_owned(), RawValue::Str("a\tb".to_owned()));
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    let buffer = scaffold(&AddConfig::default(), &specs, &seed);
    // Real: '\t' → push_str("\\t") → buffer contains `\t` (two chars).
    // Mutant (arm deleted): '\t' → push(ch) → literal tab (0x09), no `\t`.
    assert!(
        buffer.contains(r"\t"),
        "tab must be YAML-escaped as \\t: {buffer:?}"
    );
}

// ── parse_buffer: an explicit null parses to the unset value ──

#[test]
fn parse_buffer_null_returns_empty() {
    let values = parse_buffer("~").unwrap();
    assert!(values.is_empty());
}
