//! Tests for the scaffold rendering: YAML generation from discovered slots,
//! the comment block, the pre-fill from seed values and defaults, and the
//! quoting. The parse-back and the editor session live in
//! `cmd/add/interactive_tests.rs` and `cmd/add/interactive_tests_editor.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::{AddConfig, LinkValue, RawValue, discover_slots};

use super::scaffold;
use crate::cmd::add::interactive::parse_buffer;

pub(crate) const TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title", prompt: "Task title"),
  priority: slot("priority", kind: "int", default: 5),
  tags: slot("tags", kind: "list", default: ()),
)

== Summary

#slot("summary", kind: "text", prompt: "One-paragraph summary")
"#
);

pub(crate) fn specs() -> Vec<mindtape::SlotSpec> {
    discover_slots(TEMPLATE)
}

/// The scaffold a template source renders, with no seed values.
fn scaffold_of(source: &str) -> String {
    scaffold(
        &AddConfig::default(),
        &discover_slots(source),
        &BTreeMap::new(),
    )
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
fn scaffold_renders_links_as_a_yaml_list() {
    let specs =
        discover_slots("#show: task.with(links: slot(\"links\", kind: \"links\", default: ()))");
    let buffer = scaffold(&AddConfig::default(), &specs, &BTreeMap::new());
    assert!(buffer.contains("links: []"), "{buffer}");
    assert_eq!(
        parse_buffer(&buffer).unwrap()["links"],
        RawValue::List(Vec::new())
    );

    let seed = BTreeMap::from([(
        "links".to_owned(),
        RawValue::Links(vec![
            LinkValue {
                task: "origin.typ".to_owned(),
                kind: "surfaced-by".to_owned(),
                note: None,
            },
            LinkValue {
                task: "other.typ".to_owned(),
                kind: "related".to_owned(),
                note: Some("why".to_owned()),
            },
        ]),
    )]);
    let buffer = scaffold(&AddConfig::default(), &specs, &seed);
    assert!(
        buffer.contains("links:\n  - kind: \"surfaced-by\"\n    task: \"origin.typ\"\n"),
        "{buffer}"
    );
    let values = parse_buffer(&buffer).unwrap();
    assert_eq!(values["links"], seed["links"]);
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

// ── push_comment: the shape line shows what a good answer looks like ──

#[test]
fn scaffold_prints_a_slots_example_under_its_prompt() {
    let buffer = scaffold_of(
        r#"#show: task.with(title: slot("title", prompt: "Task title", example: "Sort tasks without a due date last"))"#,
    );
    assert!(
        buffer.contains(
            "# Task title [string, required]\n# e.g. Sort tasks without a due date last\ntitle: "
        ),
        "{buffer}"
    );
}

#[test]
fn scaffold_prints_a_date_slots_format_and_sample() {
    let buffer =
        scaffold_of(r#"#show: task.with(due: slot("due", kind: "date", required: false))"#);
    assert!(
        buffer.contains("# due [date, optional]\n# YYYY-MM-DD, e.g. 2026-11-17\ndue: "),
        "{buffer}"
    );
}

#[test]
fn a_date_slots_example_replaces_the_sample() {
    let buffer = scaffold_of(
        r#"#show: task.with(due: slot("due", kind: "date", required: false, example: "2027-01-05"))"#,
    );
    assert!(buffer.contains("# YYYY-MM-DD, e.g. 2027-01-05"), "{buffer}");
    assert!(!buffer.contains("2026-11-17"), "{buffer}");
}

#[test]
fn a_multiline_example_stays_a_comment() {
    let buffer = scaffold_of(
        "#show: task.with(as-is: slot(\"as-is\", kind: \"raw\", example: `+ Run the command.\n\n+ 🔴 It prints nothing.`))",
    );
    assert!(
        buffer.contains("# e.g. + Run the command.\n#\n# + 🔴 It prints nothing.\n"),
        "{buffer}"
    );
    // Every line stays a comment, so the buffer is still a YAML mapping.
    parse_buffer(&buffer).unwrap();
}

const MULTILINE_RAW_DEFAULT: &str = r#"#show: task.with(as-is: slot("as-is", kind: "raw", default: ```typ
+ Run it.

+ 🔴 It prints nothing.
```))"#;

#[test]
fn a_multiline_raw_default_stays_a_comment_and_parses_with_comments_on() {
    let buffer = scaffold_of(MULTILINE_RAW_DEFAULT);
    assert!(
        buffer.contains("# default: + Run it.\n#\n# + 🔴 It prints nothing.\n"),
        "{buffer}"
    );
    assert_eq!(
        parse_buffer(&buffer).unwrap()["as-is"],
        RawValue::Str("+ Run it.\n\n+ 🔴 It prints nothing.".to_owned())
    );
}

#[test]
fn a_multiline_raw_default_parses_with_comments_off() {
    let config = AddConfig {
        comments: false,
        ..Default::default()
    };
    let buffer = scaffold(
        &config,
        &discover_slots(MULTILINE_RAW_DEFAULT),
        &BTreeMap::new(),
    );
    assert!(
        buffer.contains("as-is: |-\n  + Run it.\n\n  + 🔴 It prints nothing."),
        "{buffer}"
    );
    assert_eq!(
        parse_buffer(&buffer).unwrap()["as-is"],
        RawValue::Str("+ Run it.\n\n+ 🔴 It prints nothing.".to_owned())
    );
}

// ── default_entry: a date default pre-fills so the editor shows the shape ──

#[test]
fn scaffold_prefills_a_date_default() {
    let buffer = scaffold_of(
        r#"#show: task.with(due: slot("due", kind: "date", required: false, default: 2026-11-17))"#,
    );
    assert!(buffer.contains("due: 2026-11-17"), "{buffer}");
    // Left untouched, it parses back to the same date, so the default survives.
    assert_eq!(
        parse_buffer(&buffer).unwrap()["due"],
        RawValue::Str("2026-11-17".to_owned())
    );
}

#[test]
fn a_date_default_that_is_not_a_ymd_literal_stays_blank() {
    // `on(2026, 11, 17)` is a valid Typst default but not a value the fill
    // accepts, so the scaffold leaves the slot blank and the comment shows it.
    let buffer = scaffold_of(
        r#"#show: task.with(due: slot("due", kind: "date", required: false, default: on(2026, 11, 17)))"#,
    );
    assert!(buffer.ends_with("due: "), "left blank: {buffer:?}");
    assert!(buffer.contains("# default: on(2026, 11, 17)"), "{buffer}");
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

#[test]
fn raw_slot_seed_renders_as_a_block() {
    let mut seed = BTreeMap::new();
    let goals = "== Goals\n\n+ #Helps(adopter, to: guide-the-implementer)[why].";
    seed.insert("goals".to_owned(), RawValue::Str(goals.to_owned()));
    let specs = discover_slots("#show: task.with(goals: slot(\"goals\", kind: \"raw\"))");
    let buffer = scaffold(&AddConfig::default(), &specs, &seed);
    assert!(
        buffer.contains(
            "goals: |-\n  == Goals\n\n  + #Helps(adopter, to: guide-the-implementer)[why]."
        ),
        "{buffer}"
    );
    assert_eq!(
        parse_buffer(&buffer).unwrap()["goals"],
        RawValue::Str(goals.to_owned())
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
