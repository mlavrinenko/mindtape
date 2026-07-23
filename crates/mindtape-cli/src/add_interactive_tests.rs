//! Tests for the interactive editor fill: scaffold rendering and parse-back.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::{AddConfig, InteractiveMode, RawValue, discover_slots};

use super::{editor_command, open_in_editor, parse_buffer, run, scaffold, should_open};

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

// ── editor_command: a blank override falls through, it is not accepted ──

#[test]
fn editor_command_env_var_not_empty_used() {
    const CHILD: &str = "_MT_EDITOR_CMD_CHILD";
    if std::env::var(CHILD).is_ok() {
        // Child: report the result of editor_command(None) via stderr so
        // the test harness (which writes to stdout) does not interfere.
        let result = editor_command(None);
        eprintln!("__MT_EDITOR={result}");
        return;
    }
    // Parent: spawn a child with VISUAL set and EDITOR removed so the
    // env-var path is deterministic.
    let exe = std::env::current_exe().unwrap();
    let output = std::process::Command::new(&exe)
        .env(CHILD, "1")
        .env("VISUAL", "custom-editor")
        .env_remove("EDITOR")
        .arg("editor_command_env_var_not_empty_used")
        .arg("--nocapture")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Parse the sentinel line from stderr (the harness writes nothing to it).
    let stderr = String::from_utf8_lossy(&output.stderr);
    let sentinel = stderr
        .lines()
        .find(|l| l.starts_with("__MT_EDITOR="))
        .unwrap_or_else(|| panic!("missing sentinel in stderr:\n{stderr}"));
    let child_result = sentinel.strip_prefix("__MT_EDITOR=").unwrap();
    // Real: `!is_empty` is true → returns "custom-editor".
    // Mutant (no !): `is_empty` is false → skips → falls to "vi".
    assert_eq!(child_result, "custom-editor");
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

// ── should_open: the negation that keeps a non-tty from opening an editor ──

#[test]
fn force_and_never_with_tty_opens() {
    let config = AddConfig {
        interactive: InteractiveMode::Never,
        ..Default::default()
    };
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    // force=true + Never + tty=true: real Ok(true).  Mutant (no !): Ok(false).
    assert!(should_open(true, &config, &specs, &BTreeMap::new(), true).unwrap());
}

// ── should_open / has_unfilled_required: the conjunctions deciding whether
//    a scaffold is worth opening, and the negation guarding each ──

#[test]
fn auto_mode_all_filled_stays_closed() {
    let config = AddConfig::default();
    let mut seed = BTreeMap::new();
    seed.insert("x".to_owned(), RawValue::Str("v".to_owned()));
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    // Seed fills the only required slot; real should_open = false.
    // Any of the four mutants makes should_open return true.
    assert!(!should_open(false, &config, &specs, &seed, true).unwrap());
}

// ── has_unfilled_required: a filled scaffold reports nothing outstanding ──

#[test]
fn auto_mode_unfilled_opens() {
    let config = AddConfig::default();
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    // No seed — has unfilled required; real should_open = true.
    // Mutant (return false) makes should_open = false.
    assert!(should_open(false, &config, &specs, &BTreeMap::new(), true).unwrap());
}

// ── has_unfilled_required: required AND unfilled, not either alone ──

#[test]
fn non_required_slot_not_unfilled() {
    let config = AddConfig::default();
    let specs = discover_slots("#show: task.with(x: slot(\"x\", required: false))");
    // spec not-required, no default, no seed → real has_unfilled_required = false.
    // Mutant: false || (true && true) = true → should_open = true.
    assert!(!should_open(false, &config, &specs, &BTreeMap::new(), true).unwrap());
}

// ── run: the edited buffer's values reach the caller ──

#[test]
fn run_parses_editor_output() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("editor.sh");
    std::fs::write(&script, "cat > \"$1\" << 'EOF'\nx: \"ParsedTitle\"\nEOF\n").unwrap();
    let config = AddConfig {
        editor: Some(format!("bash {}", script.display())),
        ..Default::default()
    };
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    let values = run(&config, &specs, &BTreeMap::new()).unwrap();
    assert_eq!(values["x"], RawValue::Str("ParsedTitle".to_owned()));
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
