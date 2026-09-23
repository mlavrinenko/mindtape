//! The `density` content assertion: split one section's content into units
//! (the section, its paragraphs, or its sentences), count each unit's regex
//! matches, and report the first unit over the bound.
//!
//! Split out of [`crate::content_gate`] to stay under the file-size gate; the
//! split rules ([`density_units`]) are the entire reason this is its own file.

use regex::Regex;

use crate::body_content::DensityUnit;

/// The first unit (in document order) whose match count of `re` exceeds
/// `max`, named by ordinal and — for a paragraph or sentence unit — its
/// first few words, or `None` when every unit is within bounds.
pub(super) fn density_violation(
    content: &str,
    heading: &str,
    re: &Regex,
    max: usize,
    per: DensityUnit,
) -> Option<String> {
    density_units(content, per)
        .iter()
        .enumerate()
        .find_map(|(idx, unit)| {
            let found = re.find_iter(unit).count();
            (found > max).then(|| {
                let subject = density_subject(heading, per, idx, unit);
                format!(
                    "{subject} must carry at most {max} matches of `{}` (found {found})",
                    re.as_str()
                )
            })
        })
}

/// What a density violation message calls the offending unit: the section
/// itself for `per = section` (there is only one unit, and the heading
/// already names it), or an ordinal plus a leading-words snippet for a
/// paragraph or sentence.
fn density_subject(heading: &str, per: DensityUnit, idx: usize, unit: &str) -> String {
    match per {
        DensityUnit::Section => format!("section `{heading}`"),
        DensityUnit::Paragraph => {
            format!(
                "section `{heading}` paragraph {} (\"{}\")",
                idx + 1,
                snippet(unit)
            )
        }
        DensityUnit::Sentence => {
            format!(
                "section `{heading}` sentence {} (\"{}\")",
                idx + 1,
                snippet(unit)
            )
        }
    }
}

/// The unit's first few words, for naming it in a violation message.
fn snippet(text: &str) -> String {
    const WORDS: usize = 6;
    let words: Vec<&str> = text.split_whitespace().take(WORDS).collect();
    let joined = words.join(" ");
    if text.split_whitespace().count() > WORDS {
        format!("{joined}…")
    } else {
        joined
    }
}

/// Split a section's content into the units a density assertion counts
/// matches over, applying the same three rules regardless of `per`:
///
/// 1. A fenced code block — a line whose trimmed text opens with three or
///    more backticks, up to the matching close — is dropped before any
///    other split. Neither its own text nor the split itself ever sees it,
///    so a pattern inside a code sample cannot trip (or, at `per =
///    paragraph`/`sentence`, cannot itself become) a unit.
/// 2. What remains is grouped into paragraphs: runs of non-blank lines
///    separated by one or more blank lines. A `- ` or `+ ` list-item line
///    starts a new paragraph even with no blank line before it, so a tight
///    list is one paragraph per item rather than one paragraph for the
///    whole list; its continuation lines (indented text with no bullet)
///    stay part of that item. Nested-subsection prose is already part of
///    `content` by the time it reaches this function (the same span
///    `min-words` counts over), so it is grouped exactly like top-level
///    section prose.
/// 3. At `per = sentence`, each paragraph is further split on `.`, `!`, or
///    `?` followed by whitespace or the paragraph's end.
///
/// `per = section` skips steps 2 and 3: the whole fence-stripped content is
/// the one unit.
fn density_units(content: &str, per: DensityUnit) -> Vec<String> {
    let stripped = strip_fenced_code(content);
    match per {
        DensityUnit::Section => vec![stripped],
        DensityUnit::Paragraph => split_paragraphs(&stripped),
        DensityUnit::Sentence => split_paragraphs(&stripped)
            .iter()
            .flat_map(|p| split_sentences(p))
            .collect(),
    }
}

/// Drop every fenced code block (open/close lines and the lines between)
/// from `content`, keeping every other line verbatim including blank ones,
/// so paragraph boundaries around a removed fence are unaffected.
fn strip_fenced_code(content: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Group lines into paragraphs: a blank line always closes the current one;
/// a `- `/`+ ` list-item line closes it too (a new item starts even without
/// a blank line before it) and opens a new paragraph on itself; every other
/// line continues whichever paragraph is open.
fn split_paragraphs(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let is_list_item = trimmed.starts_with("- ") || trimmed.starts_with("+ ");
        if line.trim().is_empty() {
            flush_paragraph(&mut paragraphs, &mut current);
            continue;
        }
        if is_list_item {
            flush_paragraph(&mut paragraphs, &mut current);
        }
        current.push_str(line);
        current.push('\n');
    }
    flush_paragraph(&mut paragraphs, &mut current);
    paragraphs
}

fn flush_paragraph(paragraphs: &mut Vec<String>, current: &mut String) {
    if !current.trim().is_empty() {
        paragraphs.push(std::mem::take(current));
    }
    current.clear();
}

/// Split one paragraph on `.`, `!`, or `?` followed by whitespace or the
/// paragraph's end.
fn split_sentences(paragraph: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut chars = paragraph.chars().peekable();
    while let Some(c) = chars.next() {
        current.push(c);
        if matches!(c, '.' | '!' | '?') && chars.peek().is_none_or(|next| next.is_whitespace()) {
            push_trimmed(&mut sentences, &mut current);
        }
    }
    push_trimmed(&mut sentences, &mut current);
    sentences
}

fn push_trimmed(sentences: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_owned());
    }
    current.clear();
}
