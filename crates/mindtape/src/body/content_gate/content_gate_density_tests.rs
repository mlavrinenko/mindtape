//! Tests for the density split helpers: how a violation names its unit, how a
//! unit is snipped to its first few words, and the sentence/trim rules behind
//! `split_sentences` and `push_trimmed`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::body::content::DensityUnit;

use regex::Regex;

use super::{density_subject, density_violation, push_trimmed, snippet, split_sentences};

// --- density_subject ---------------------------------------------------

#[test]
fn density_subject_names_the_section_for_section_units() {
    assert_eq!(
        density_subject("Scope", DensityUnit::Section, 0, "ignored"),
        "section `Scope`"
    );
}

#[test]
fn density_subject_names_a_paragraph_by_ordinal_and_snippet() {
    assert_eq!(
        density_subject("Scope", DensityUnit::Paragraph, 1, "one two three"),
        "section `Scope` paragraph 2 (\"one two three\")"
    );
}

#[test]
fn density_subject_names_a_sentence_by_ordinal_and_snippet() {
    assert_eq!(
        density_subject("Scope", DensityUnit::Sentence, 0, "one two three"),
        "section `Scope` sentence 1 (\"one two three\")"
    );
}

// --- snippet -----------------------------------------------------------

#[test]
fn snippet_returns_short_text_verbatim() {
    assert_eq!(snippet("hello world"), "hello world");
}

#[test]
fn snippet_does_not_elide_at_exactly_six_words() {
    assert_eq!(snippet("a b c d e f"), "a b c d e f");
}

#[test]
fn snippet_elides_past_six_words() {
    assert_eq!(snippet("a b c d e f g"), "a b c d e f…");
}

// --- split_sentences ---------------------------------------------------

/// The sentences `split_sentences` finds in `text` taken as one paragraph.
fn sentences(text: &str) -> Vec<&str> {
    split_sentences(text, 0..text.len())
        .into_iter()
        .map(|range| &text[range])
        .collect()
}

#[test]
fn split_sentences_splits_on_terminators_followed_by_whitespace() {
    assert_eq!(
        sentences("One. Two! Three? Four"),
        vec!["One.", "Two!", "Three?", "Four"]
    );
}

#[test]
fn split_sentences_splits_on_a_terminator_at_the_end() {
    assert_eq!(sentences("Only one."), vec!["Only one."]);
}

/// A `.` inside a number is not a sentence end: it is not followed by
/// whitespace, so the whole run stays one sentence.
#[test]
fn split_sentences_does_not_split_inside_a_number() {
    assert_eq!(
        sentences("Version 3.14 is out."),
        vec!["Version 3.14 is out."]
    );
}

#[test]
fn split_sentences_of_empty_input_is_empty() {
    assert!(sentences("").is_empty());
}

#[test]
fn split_sentences_drops_whitespace_only_fragments() {
    assert!(sentences("   \n  ").is_empty());
}

// --- push_trimmed ------------------------------------------------------

#[test]
fn push_trimmed_pushes_the_trimmed_range_shifted_by_the_offset() {
    let mut sentences = Vec::new();
    push_trimmed(&mut sentences, "  hi there  ", 0..12, 10);
    assert_eq!(sentences, vec![12..20]);
}

#[test]
fn push_trimmed_ignores_whitespace_only() {
    let mut sentences = Vec::new();
    push_trimmed(&mut sentences, "   ", 0..3, 0);
    assert!(sentences.is_empty());
}

// --- inline code -------------------------------------------------------

/// The open-question pattern, counted per sentence with no match allowed.
fn question(content: &str) -> Option<String> {
    let re = Regex::new(r"\?(\s|$)").unwrap();
    density_violation(content, "TO-BE", &re, 0, DensityUnit::Sentence)
}

/// The sentence a breach of [`question`] names, as `2 ("Why?")`.
fn flagged(content: &str) -> Option<String> {
    let found = question(content)?;
    let named = found.strip_prefix("section `TO-BE` sentence ")?;
    let end = named.find(") must carry")?;
    Some(named[..=end].to_owned())
}

#[test]
fn a_question_mark_inside_a_code_span_does_not_count() {
    assert_eq!(question("- `mt ls ? --help` shows the flag.\n"), None);
    assert_eq!(question("Run ``` a? b ``` now.\n"), None);
    // A single backtick inside a triple span is code, not a close.
    assert_eq!(question("Run ``` a ` b? ``` now.\n"), None);
    assert_eq!(question("Fine.\n\n```sh\nmt ls ? --help\n```\n"), None);
}

#[test]
fn a_breach_names_the_sentence_and_the_pattern() {
    assert_eq!(
        question("- `mt ls ? --help` shows the flag. Then why?\n").as_deref(),
        Some(
            "section `TO-BE` sentence 2 (\"Then why?\") must carry at most 0 matches \
             of `\\?(\\s|$)` (found 1)"
        )
    );
}

#[test]
fn code_never_splits_a_sentence_or_hides_the_prose_after_it() {
    let cases = [
        // A line that opens three backticks and closes them on the same line
        // is inline code, not a fence that swallows every line after it.
        (
            "``` a? ```\n\nWhy is this here?\n",
            "2 (\"Why is this here?\")",
        ),
        // An escaped backtick opens no span.
        ("A \\` tick? and \\` more.\n", "1 (\"A \\` tick?\")"),
        // The snippet drops the list marker and quotes a span as written.
        (
            "- Run `mt ls` first.\n- Should it also cover the board?\n",
            "2 (\"Should it also cover the board?\")",
        ),
        (
            "`mt ls` lists them, enough?\n",
            "1 (\"`mt ls` lists them, enough?\")",
        ),
        // Two backticks are an empty span, and a run that never closes is text.
        ("An `` empty span? Yes.\n", "1 (\"An `` empty span?\")"),
        ("A `stray tick? Yes.\n", "1 (\"A `stray tick?\")"),
        // A fence drops out whole, so the prose after it is its own sentence.
        ("```sh\nmt ls\n```\nWhy?\n", "1 (\"Why?\")"),
    ];
    for (content, sentence) in cases {
        assert_eq!(flagged(content).as_deref(), Some(sentence), "{content:?}");
    }
}
