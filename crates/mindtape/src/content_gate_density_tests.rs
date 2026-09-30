//! Tests for the density split helpers: how a violation names its unit, how a
//! unit is snipped to its first few words, and the sentence/trim rules behind
//! `split_sentences` and `push_trimmed`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::body_content::DensityUnit;

use super::{density_subject, push_trimmed, snippet, split_sentences};

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

#[test]
fn split_sentences_splits_on_terminators_followed_by_whitespace() {
    assert_eq!(
        split_sentences("One. Two! Three? Four"),
        vec!["One.", "Two!", "Three?", "Four"]
    );
}

#[test]
fn split_sentences_splits_on_a_terminator_at_the_end() {
    assert_eq!(split_sentences("Only one."), vec!["Only one."]);
}

/// A `.` inside a number is not a sentence end: it is not followed by
/// whitespace, so the whole run stays one sentence.
#[test]
fn split_sentences_does_not_split_inside_a_number() {
    assert_eq!(
        split_sentences("Version 3.14 is out."),
        vec!["Version 3.14 is out."]
    );
}

#[test]
fn split_sentences_of_empty_input_is_empty() {
    assert!(split_sentences("").is_empty());
}

#[test]
fn split_sentences_drops_whitespace_only_fragments() {
    assert!(split_sentences("   \n  ").is_empty());
}

// --- push_trimmed ------------------------------------------------------

#[test]
fn push_trimmed_pushes_trimmed_nonempty_and_clears() {
    let mut sentences = Vec::new();
    let mut current = "  hi there  ".to_owned();
    push_trimmed(&mut sentences, &mut current);
    assert_eq!(sentences, vec!["hi there"]);
    assert!(current.is_empty());
}

#[test]
fn push_trimmed_ignores_whitespace_only() {
    let mut sentences = Vec::new();
    let mut current = "   ".to_owned();
    push_trimmed(&mut sentences, &mut current);
    assert!(sentences.is_empty());
    assert!(current.is_empty());
}
