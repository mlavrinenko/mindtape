//! Tests for the scaffold rendering: YAML generation from discovered slots,
//! the comment block, the pre-fill from seed values and defaults, and the
//! quoting. The parse-back and the editor session live in
//! `cmd/add/interactive_tests.rs` and `cmd/add/interactive_tests_editor.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::{AddConfig, RawValue};

use crate::cmd::add::support_tests::{
    GOALS_SOURCE, LINKS_SOURCE, TAGS_SOURCE, TITLE_SOURCE, X_BOOL_SOURCE, X_FLOAT_SOURCE,
    X_OPTIONAL_SOURCE, X_SOURCE, X_TEXT_SOURCE, comments_off, header_config, links, list, parse,
    scaffold_of, scaffold_with, scaffold_with_config, seed,
};

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

#[test]
fn scaffold_emits_one_entry_per_slot_with_comments() {
    let buffer = scaffold_of(TEMPLATE);
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
    let buffer = scaffold_of(TEMPLATE);
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
    let buffer = scaffold_of(TITLE_SOURCE);
    assert!(!buffer.ends_with('\n'), "no trailing newline: {buffer:?}");
    assert!(
        buffer.ends_with("title: "),
        "ends at value position: {buffer:?}"
    );
}

#[test]
fn scaffold_custom_header_is_comment_prefixed() {
    let config = header_config("My note\nSecond line");
    let buffer = scaffold_with_config(&config, TITLE_SOURCE);
    assert!(
        buffer.starts_with("# My note\n# Second line\n\n# "),
        "{buffer}"
    );
}

#[test]
fn scaffold_empty_header_disables_it() {
    let config = header_config("");
    let buffer = scaffold_with_config(&config, TITLE_SOURCE);
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
    let buffer = scaffold_with_config(&config, TEMPLATE);
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
    parse(&buffer);
}

#[test]
fn scaffold_comments_on_spaces_entries_with_blank_lines() {
    let buffer = scaffold_of(TEMPLATE);
    assert!(
        buffer.contains("\n\n"),
        "comment groups stay spaced: {buffer:?}"
    );
}

#[test]
fn scaffold_prefills_from_seed() {
    let seed = seed(&[
        ("title", RawValue::Str("Hello".to_owned())),
        ("priority", RawValue::Int(8)),
    ]);
    let buffer = scaffold_with(TEMPLATE, &seed);
    assert!(buffer.contains("title: \"Hello\""), "{buffer}");
    assert!(buffer.contains("priority: 8"), "{buffer}");
}

#[test]
fn scaffold_renders_empty_list_default_as_bracket_pair() {
    // TEMPLATE's tags slot has `default: ()` — an empty list. It should surface
    // as an explicit empty flow sequence, not a blank scalar.
    let buffer = scaffold_of(TEMPLATE);
    assert!(buffer.contains("tags: []"), "{buffer}");
    // Left untouched, it parses back to an empty list (not dropped as null).
    let values = parse(&buffer);
    assert_eq!(values["tags"], RawValue::List(Vec::new()));
}

#[test]
fn scaffold_renders_links_as_a_yaml_list() {
    let buffer = scaffold_of(LINKS_SOURCE);
    assert!(buffer.contains("links: []"), "{buffer}");
    assert_eq!(parse(&buffer)["links"], RawValue::List(Vec::new()));

    let seed = seed(&[("links", links())]);
    let buffer = scaffold_with(LINKS_SOURCE, &seed);
    assert!(
        buffer.contains("links:\n  - kind: \"surfaced-by\"\n    task: \"origin.typ\"\n"),
        "{buffer}"
    );
    let values = parse(&buffer);
    assert_eq!(values["links"], seed["links"]);
}

#[test]
fn scaffold_leaves_non_empty_list_default_blank() {
    // A non-empty list default stays blank so an untouched slot falls through to
    // the default rather than being overwritten with an empty `[]`.
    let buffer = scaffold_of(TAGS_SOURCE);
    assert!(buffer.ends_with("tags: "), "left blank: {buffer:?}");
    assert!(!buffer.contains("tags: []"), "not bracketed: {buffer}");
    // The default is advertised in the comment.
    assert!(buffer.contains("# default:"), "{buffer}");
}

#[test]
fn round_trip_scaffold_then_parse() {
    let seed = seed(&[
        ("title", RawValue::Str("Round trip".to_owned())),
        ("tags", list(&["a", "b"])),
    ]);
    let buffer = scaffold_with(TEMPLATE, &seed);
    let values = parse(&buffer);
    assert_eq!(values["title"], RawValue::Str("Round trip".to_owned()));
    assert_eq!(values["priority"], RawValue::Int(5)); // default pre-fill survives
    assert_eq!(values["tags"], list(&["a", "b"]));
}

// ── commentify: either condition alone triggers the comment prefix ──

#[test]
fn custom_header_comment_lines_kept() {
    let config = header_config("# Already a comment\nReal text");
    let buffer = scaffold_with_config(&config, X_SOURCE);
    // Real: `# Already a comment` kept as-is (starts with # → kept).
    // Mutant: `false && true` → adds `# ` prefix → `# # Already a comment`.
    assert!(buffer.starts_with("# Already a comment"), "{buffer}");
}

// ── push_comment: both conditions must hold before a comment is pushed ──

#[test]
fn non_required_slot_label_is_optional() {
    let buffer = scaffold_of(X_OPTIONAL_SOURCE);
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
    parse(&buffer);
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
        parse(&buffer)["as-is"],
        RawValue::Str("+ Run it.\n\n+ 🔴 It prints nothing.".to_owned())
    );
}

#[test]
fn a_multiline_raw_default_parses_with_comments_off() {
    let config = comments_off();
    let buffer = scaffold_with_config(&config, MULTILINE_RAW_DEFAULT);
    assert!(
        buffer.contains("as-is: |-\n  + Run it.\n\n  + 🔴 It prints nothing."),
        "{buffer}"
    );
    assert_eq!(
        parse(&buffer)["as-is"],
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
        parse(&buffer)["due"],
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
    let seed = seed(&[("x", RawValue::Str("hello\nworld".to_owned()))]);
    let buffer = scaffold_with(X_TEXT_SOURCE, &seed);
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
    let goals = "== Goals\n\n+ #Helps(adopter, to: guide-the-implementer)[why].";
    let seed = seed(&[("goals", RawValue::Str(goals.to_owned()))]);
    let buffer = scaffold_with(GOALS_SOURCE, &seed);
    assert!(
        buffer.contains(
            "goals: |-\n  == Goals\n\n  + #Helps(adopter, to: guide-the-implementer)[why]."
        ),
        "{buffer}"
    );
    assert_eq!(parse(&buffer)["goals"], RawValue::Str(goals.to_owned()));
}

// ── seed_entry: the Float and Bool arms render their typed scalars ──

#[test]
fn float_seed_renders_as_scalar() {
    let seed = seed(&[("x", RawValue::Float(1.5))]);
    let buffer = scaffold_with(X_FLOAT_SOURCE, &seed);
    assert!(buffer.contains("x: 1.5"), "{buffer}");
    // Round-trips back as a typed float, not a string.
    let values = parse(&buffer);
    assert_eq!(values["x"], RawValue::Float(1.5));
}

#[test]
fn bool_seed_renders_as_scalar() {
    let seed = seed(&[("x", RawValue::Bool(true))]);
    let buffer = scaffold_with(X_BOOL_SOURCE, &seed);
    assert!(buffer.contains("x: true"), "{buffer}");
    let values = parse(&buffer);
    assert_eq!(values["x"], RawValue::Bool(true));
}

// ── quote: every character that must be escaped, one case each ──

#[test]
fn quote_special_chars_roundtrip() {
    // A string containing all four special chars: ", \, newline, tab.
    let seed = seed(&[("x", RawValue::Str("a\"b\\c\nd\te".to_owned()))]);
    let buffer = scaffold_with(X_SOURCE, &seed);
    let values = parse(&buffer);
    assert_eq!(values["x"], RawValue::Str("a\"b\\c\nd\te".to_owned()));
}

// ── quote: a tab is escaped, not passed through ──

#[test]
fn quote_escapes_tab_to_backslash_t() {
    let seed = seed(&[("x", RawValue::Str("a\tb".to_owned()))]);
    let buffer = scaffold_with(X_SOURCE, &seed);
    // Real: '\t' → push_str("\\t") → buffer contains `\t` (two chars).
    // Mutant (arm deleted): '\t' → push(ch) → literal tab (0x09), no `\t`.
    assert!(
        buffer.contains(r"\t"),
        "tab must be YAML-escaped as \\t: {buffer:?}"
    );
}
