//! The `words` assertion: the `min` and `max` bounds, the three `count`
//! units, and the section default's inheritance. Split out of
//! `content_gate/tests.rs`, which was at the linecop cap.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests::{BODY, run, run_with};

/// Prose `one two three` and a fenced block of eight tokens, so `all` and
/// `prose` disagree on the same bound.
const FENCED: &str = "== Scope\n\none two three\n\n```\ncode has many words here\n```\n";

#[test]
fn words_min_fails_when_short() {
    let body = "== Scope\n\ntwo words\n";
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { words = { min = 3 } }\n",
    );
    assert_eq!(messages.len(), 1);
    assert!(
        messages[0].contains("at least 3 words (found 2)"),
        "{messages:?}"
    );
}

#[test]
fn words_min_passes_when_long_enough() {
    assert!(
        run(
            BODY,
            "path = [\"Scope\"]\nassert = { words = { min = 4 } }\n"
        )
        .is_empty()
    );
}

#[test]
fn words_min_includes_nested_subsection_prose() {
    // Scope at 4 words would fail alone; the nested subsection's prose is part
    // of the parent section.
    let body = "== Scope\n\n=== Details\n\none two three four five six\n";
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { words = { min = 6 } }\n",
    );
    assert!(
        messages.is_empty(),
        "nested prose should count: {messages:?}"
    );
}

#[test]
fn words_all_counts_fenced_code_tokens() {
    assert!(
        run(
            FENCED,
            "path = [\"Scope\"]\nassert = { words = { max = 4 } }\n"
        )
        .len()
            == 1,
        "the default `all` must count the fenced block"
    );
}

#[test]
fn words_prose_drops_fenced_code() {
    assert!(
        run(
            FENCED,
            "path = [\"Scope\"]\nassert = { words = { max = 3, count = \"prose\" } }\n",
        )
        .is_empty(),
        "the fenced block must not spend a prose cap"
    );
}

#[test]
fn words_prose_counts_inline_code() {
    let body = "== Scope\n\na `quoted code span` sentence\n";
    assert!(
        run(
            body,
            "path = [\"Scope\"]\nassert = { words = { max = 5, count = \"prose\" } }\n",
        )
        .is_empty(),
        "inline code sits inside a sentence and counts as prose"
    );
}

#[test]
fn words_text_masks_inline_code() {
    let body = "== Scope\n\na `quoted code span` sentence\n";
    assert!(
        run(
            body,
            "path = [\"Scope\"]\nassert = { words = { max = 2, count = \"text\" } }\n",
        )
        .is_empty(),
        "an inline raw span must not spend a text cap"
    );
}

#[test]
fn words_inherits_the_section_count() {
    assert!(
        run_with(
            "[tasks.body.content]\nwords = { count = \"prose\" }\n",
            FENCED,
            "path = [\"Scope\"]\nassert = { words = { max = 3 } }\n",
        )
        .is_empty(),
        "a rule without `count` must inherit the section default"
    );
}

#[test]
fn words_rule_count_overrides_the_section_default() {
    let messages = run_with(
        "[tasks.body.content]\nwords = { count = \"prose\" }\n",
        FENCED,
        "path = [\"Scope\"]\nassert = { words = { max = 3, count = \"all\" } }\n",
    );
    assert_eq!(messages.len(), 1, "the rule's own count wins: {messages:?}");
}

#[test]
fn words_ignores_punctuation_only_tokens() {
    let body = "== Scope\n\n- - - _ _ _\n";
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { words = { min = 2 } }\n",
    );
    assert_eq!(messages.len(), 1, "separators are not words: {messages:?}");
}

#[test]
fn words_passes_at_exactly_the_boundary() {
    let body = "== Scope\n\none two three four five\n";
    assert!(
        run(
            body,
            "path = [\"Scope\"]\nassert = { words = { max = 5 } }\n"
        )
        .is_empty()
    );
}

#[test]
fn words_fails_one_over_the_boundary() {
    let body = "== Scope\n\none two three four five six\n";
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { words = { max = 5 } }\n",
    );
    assert_eq!(messages.len(), 1);
    assert!(
        messages[0].contains("at most 5 words (found 6)"),
        "{messages:?}"
    );
}

#[test]
fn words_reports_the_failing_bound_of_two() {
    let body = "== Scope\n\none two\n";
    let messages = run(
        body,
        "path = [\"Scope\"]\nassert = { words = { min = 4, max = 10 } }\n",
    );
    assert_eq!(messages.len(), 1);
    assert!(
        messages[0].contains("at least 4 words (found 2)"),
        "{messages:?}"
    );
}
