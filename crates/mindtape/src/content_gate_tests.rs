//! `[[tasks.body.content.rules]]` — rule compilation and section-content
//! checking: path selection (exact + regex segments), section-span slicing
//! through nested subsections, the six assertion shapes, `when` gating, the
//! `message` override, and the word-count semantics.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::config::parse_config;
use crate::content_gate::{GateError, check_content, compile_content_rules};
use crate::test_support::bare_task;

/// One `[[tasks.body.content.rules]]` entry in TOML, compiled and run against
/// `body`, returning the violation messages (heading line, field, message).
fn run(body: &str, rule_toml: &str) -> Vec<String> {
    let config = parse_config(&format!("[[tasks.body.content.rules]]\n{rule_toml}")).unwrap();
    let rules = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let task = bare_task("x");
    check_content(&task, body, &rules, &config)
        .into_iter()
        .map(|v| format!("{:?}:{}/{}", v.location.line(), v.field, v.message))
        .collect()
}

const BODY: &str = "== Summary\n\nA short summary.\n\n== Scope\n\n- item one\n- item two\n\n== Details\n\nNested prose.\n\n=== Sub\n\nMore prose.\n\n== Watch for\n\nIs this an open question?\n";

// --- compile errors ---

#[test]
fn compile_empty_rules_succeeds() {
    assert!(compile_content_rules(&[]).unwrap().is_empty());
}

#[test]
fn compile_rejects_empty_path() {
    let config =
        parse_config("[[tasks.body.content.rules]]\npath = []\nassert = \"non-empty\"\n").unwrap();
    let err = compile_content_rules(&config.tasks.body.content.rules).unwrap_err();
    assert!(matches!(err, GateError::EmptyPath));
}

#[test]
fn compile_rejects_invalid_regex() {
    let config = parse_config(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = { must-match = \"(\" }\n",
    )
    .unwrap();
    let err = compile_content_rules(&config.tasks.body.content.rules).unwrap_err();
    assert!(
        matches!(&err, GateError::Regex { pattern, .. } if pattern == "("),
        "expected Regex error, got: {err}"
    );
}

#[test]
fn compile_rejects_invalid_when() {
    let config = parse_config(
        "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\nwhen = 'nope('\n",
    )
    .unwrap();
    assert!(compile_content_rules(&config.tasks.body.content.rules).is_err());
}

// --- non-empty ---

#[test]
fn non_empty_passes_with_prose() {
    assert!(run(BODY, "path = [\"Summary\"]\nassert = \"non-empty\"\n").is_empty());
}

#[test]
fn non_empty_fails_on_empty_section() {
    let body = "== Summary\n\n== Scope\n\nprose\n";
    let messages = run(body, "path = [\"Summary\"]\nassert = \"non-empty\"\n");
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("Summary") && messages[0].contains("not be empty"));
    assert!(
        messages[0].starts_with("1:body.content.rules[0]/"),
        "violation must point at the heading line: {messages:?}"
    );
}

#[test]
fn named_rule_reports_its_name_instead_of_position() {
    let body = "== Summary\n\n== Scope\n\nprose\n";
    let messages = run(
        body,
        "name = \"summary-non-empty\"\npath = [\"Summary\"]\nassert = \"non-empty\"\n",
    );
    assert_eq!(messages.len(), 1);
    assert!(
        messages[0].starts_with("1:body.content.rules.summary-non-empty/"),
        "{messages:?}"
    );
}

#[test]
fn non_empty_missing_section_passes() {
    // No section named Way off — nothing selected, nothing asserted.
    assert!(run(BODY, "path = [\"Way off\"]\nassert = \"non-empty\"\n").is_empty());
}

#[test]
fn non_empty_counts_raw_block_as_content() {
    // A section that is all #raw is a real shape and must not read as empty.
    let body = "== Scope\n\n#raw[\n  config: 42\n]\n";
    assert!(run(body, "path = [\"Scope\"]\nassert = \"non-empty\"\n").is_empty());
}

// --- min-words ---

#[test]
fn min_words_fails_when_short() {
    let body = "== Scope\n\ntwo words\n";
    let messages = run(body, "path = [\"Scope\"]\nassert = { min-words = 3 }\n");
    assert_eq!(messages.len(), 1);
    assert!(
        messages[0].contains("at least 3 words (found 2)"),
        "{messages:?}"
    );
}

#[test]
fn min_words_passes_when_long_enough() {
    assert!(run(BODY, "path = [\"Scope\"]\nassert = { min-words = 4 }\n").is_empty());
}

#[test]
fn min_words_includes_nested_subsection_prose() {
    // Scope at 4 words would fail alone; the nested subsection's prose is part
    // of the parent section.
    let body = "== Scope\n\n=== Details\n\none two three four five six\n";
    let messages = run(body, "path = [\"Scope\"]\nassert = { min-words = 6 }\n");
    assert!(
        messages.is_empty(),
        "nested prose should count: {messages:?}"
    );
}

#[test]
fn min_words_counts_code_block_tokens() {
    let body = "== Scope\n\n#raw[\n  a b c d e\n]\n";
    assert!(run(body, "path = [\"Scope\"]\nassert = { min-words = 5 }\n").is_empty());
}

#[test]
fn min_words_ignores_punctuation_only_tokens() {
    let body = "== Scope\n\n- - - _ _ _\n";
    let messages = run(body, "path = [\"Scope\"]\nassert = { min-words = 2 }\n");
    assert_eq!(messages.len(), 1, "separators are not words: {messages:?}");
}

// --- max-words ---

#[test]
fn max_words_passes_at_exactly_the_boundary() {
    let body = "== Scope\n\none two three four five\n";
    assert!(run(body, "path = [\"Scope\"]\nassert = { max-words = 5 }\n").is_empty());
}

#[test]
fn max_words_fails_one_over_the_boundary() {
    let body = "== Scope\n\none two three four five six\n";
    let messages = run(body, "path = [\"Scope\"]\nassert = { max-words = 5 }\n");
    assert_eq!(messages.len(), 1);
    assert!(
        messages[0].contains("at most 5 words (found 6)"),
        "{messages:?}"
    );
}

// --- density ---

#[test]
fn density_fails_two_matches_in_one_paragraph_at_max_one() {
    let body = "== Scope\n\none thing — and another — in one paragraph.\n";
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1, per = \"paragraph\" } }\n",
    );
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(
        messages[0].contains("at most 1") && messages[0].contains("found 2"),
        "{messages:?}"
    );
}

#[test]
fn density_passes_two_matches_at_max_two() {
    let body = "== Scope\n\none thing — and another — in one paragraph.\n";
    assert!(
        run(
            body,
            "path = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 2, per = \"paragraph\" } }\n"
        )
        .is_empty()
    );
}

#[test]
fn density_per_defaults_to_section() {
    // Two paragraphs, one match each: over one paragraph at a time this would
    // pass at max 1, but the default `per` pools the whole section.
    let body = "== Scope\n\nfirst — dash.\n\nsecond — dash.\n";
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1 } }\n",
    );
    assert_eq!(
        messages.len(),
        1,
        "default per=section pools the section: {messages:?}"
    );
}

#[test]
fn density_paragraph_split_counts_each_paragraph_separately() {
    // Each paragraph carries one match; per=paragraph must not fail at max 1.
    let body = "== Scope\n\nfirst — dash.\n\nsecond — dash.\n";
    assert!(
        run(
            body,
            "path = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1, per = \"paragraph\" } }\n"
        )
        .is_empty()
    );
}

#[test]
fn density_each_list_item_is_its_own_paragraph() {
    // Two list items back to back, no blank line between: each is still its
    // own paragraph unit, so one dash per item passes at max 1.
    let body = "== Scope\n\n- first — item\n- second — item\n";
    assert!(
        run(
            body,
            "path = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1, per = \"paragraph\" } }\n"
        )
        .is_empty()
    );
}

#[test]
fn density_sentence_split_counts_each_sentence_separately() {
    // Two sentences in one paragraph, one match each: per=sentence must not
    // fail at max 1, though per=paragraph would.
    let body = "== Scope\n\nFirst — sentence. Second — sentence.\n";
    assert!(
        run(
            body,
            "path = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1, per = \"sentence\" } }\n"
        )
        .is_empty()
    );
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1, per = \"paragraph\" } }\n",
    );
    assert_eq!(messages.len(), 1, "{messages:?}");
}

#[test]
fn density_excludes_fenced_code_block_from_matching() {
    // The dash sits only inside a fenced code block; density must not count
    // it, at any granularity.
    let body = "== Scope\n\nprose with no dash.\n\n```\ncode — with a dash\n```\n";
    assert!(
        run(
            body,
            "path = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 0, per = \"paragraph\" } }\n"
        )
        .is_empty(),
        "fenced code content must not count toward density"
    );
}

// --- must-match / must-not-match ---

#[test]
fn must_match_passes_when_question_mark_present() {
    assert!(
        run(
            BODY,
            "path = [\"Watch for\"]\nassert = { must-match = \"\\\\?\" }\n"
        )
        .is_empty()
    );
}

#[test]
fn must_match_fails_on_open_question_section_without_question() {
    let body = "== Watch for\n\nNo question here.\n";
    let messages = run(
        body,
        "path = [\"Watch for\"]\nassert = { must-match = \"\\\\?\" }\n",
    );
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("must match"), "{messages:?}");
}

#[test]
fn must_not_match_fails_when_pattern_present() {
    let body = "== Scope\n\nTODO finish this.\n";
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { must-not-match = \"TODO\" }\n",
    );
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("must not match"), "{messages:?}");
}

#[test]
fn must_match_scans_subsection_content_too() {
    // The question lives in a nested subsection under Watch for.
    let body = "== Watch for\n\n=== Nested\n\nIs it done?\n";
    assert!(
        run(
            body,
            "path = [\"Watch for\"]\nassert = { must-match = \"\\\\?\" }\n"
        )
        .is_empty()
    );
}

// --- regex path segment ---

#[test]
fn regex_path_selects_matching_heading_text() {
    let body = "== Watch for costs\n\nNo question\n";
    let messages = run(
        body,
        "path = [{ match = \"^Watch\" }]\nassert = { must-match = \"\\\\?\" }\n",
    );
    assert_eq!(
        messages.len(),
        1,
        "regex segment must select the section: {messages:?}"
    );
}

#[test]
fn recursive_regex_path_finds_nested_section() {
    let body = "== Outline\n\n=== Watch for\n\nNo question\n";
    let messages = run(
        body,
        "path = [{ match = \"^Watch\", recursive = true }]\nassert = { must-match = \"\\\\?\" }\n",
    );
    assert_eq!(messages.len(), 1);
}

// --- when gating ---

#[test]
fn when_filter_skips_nonmatching_task() {
    let config = parse_config(
        "[[tasks.body.content.rules]]\n\
         path = [\"Summary\"]\n\
         assert = \"non-empty\"\n\
         when = 'kind == \"bug\"'\n",
    )
    .unwrap();
    let rules = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    // Task carries no kind -> the `when` filter excludes it.
    let mut task = bare_task("x");
    let body = "== Summary\n";
    assert!(check_content(&task, body, &rules, &config).is_empty());

    task.kind = Some("bug".to_owned());
    assert_eq!(check_content(&task, body, &rules, &config).len(), 1);
}

// --- message override ---

#[test]
fn message_override_replaces_default() {
    let config = parse_config(
        "[[tasks.body.content.rules]]\n\
         path = [\"Summary\"]\n\
         assert = \"non-empty\"\n\
         message = \"a summary is mandatory\"\n",
    )
    .unwrap();
    let rules = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let task = bare_task("x");
    let out = check_content(&task, "== Summary\n", &rules, &config);
    assert_eq!(out[0].message, "a summary is mandatory");
}

// --- path traversal mirrors the header rules ---

#[test]
fn nested_path_selects_subsection() {
    let body = "== Outline\n\n=== Watch for\n\nNo question\n";
    let messages = run(
        body,
        "path = [\"Outline\", \"Watch for\"]\nassert = { must-match = \"\\\\?\" }\n",
    );
    assert_eq!(messages.len(), 1);
}

#[test]
fn exact_path_does_not_match_nested_section() {
    // Exact segments are direct-child matches: a nested Watch for is not found
    // by a top-level path.
    let body = "== Outline\n\n=== Watch for\n\nNo question\n";
    let messages = run(
        body,
        "path = [\"Watch for\"]\nassert = { must-match = \"\\\\?\" }\n",
    );
    assert!(messages.is_empty());
}

#[test]
fn section_span_stops_at_same_level_heading() {
    // The Scope section must NOT include the following Details prose.
    let body = "== Scope\n\nscope words\n\n== Details\n\na very long details sentence that would pad the count\n";
    let messages = run(body, "path = [\"Scope\"]\nassert = { min-words = 8 }\n");
    assert_eq!(
        messages.len(),
        1,
        "scope must not borrow details' words: {messages:?}"
    );
}
