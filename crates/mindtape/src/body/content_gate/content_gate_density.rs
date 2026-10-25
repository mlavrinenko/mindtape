//! The `density` content assertion: split one section's content into units
//! (the section, its paragraphs, or its sentences), count each unit's regex
//! matches, and report the first unit over the bound.
//!
//! Split out of [`crate::body::content_gate`] to stay under the file-size gate; the
//! split rules ([`unit_ranges`]) are the entire reason this is its own file.

use std::ops::Range;

use regex::Regex;

use crate::body::content::DensityUnit;

/// What a byte of inline code becomes in the text the splits and counts read:
/// neither whitespace, punctuation nor a word character, so no pattern about
/// prose matches it and no split happens inside a span.
const MASK: char = '\u{1}';

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
    let text = strip_fenced_code(content);
    let masked = mask_inline_code(&text);
    unit_ranges(&masked, per)
        .into_iter()
        .enumerate()
        .find_map(|(idx, range)| {
            let found = re
                .find_iter(masked.get(range.clone()).unwrap_or_default())
                .count();
            (found > max).then(|| {
                let unit = text.get(range).unwrap_or_default();
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

/// The unit's first few words, without a leading `- ` or `+ ` list marker,
/// for naming it in a violation message.
fn snippet(text: &str) -> String {
    const WORDS: usize = 6;
    let trimmed = text.trim_start();
    let text = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("+ "))
        .unwrap_or(trimmed);
    let words: Vec<&str> = text.split_whitespace().take(WORDS).collect();
    let joined = words.join(" ");
    if text.split_whitespace().count() > WORDS {
        format!("{joined}…")
    } else {
        joined
    }
}

/// Split a section's content into the byte ranges of the units a density
/// assertion counts matches over, applying the same rules regardless of
/// `per`. `text` has had its fenced code dropped and its inline code masked
/// ([`strip_fenced_code`], [`mask_inline_code`]), so code neither matches nor
/// splits.
///
/// 1. The text is grouped into paragraphs: runs of non-blank lines
///    separated by one or more blank lines. A `- ` or `+ ` list-item line
///    starts a new paragraph even with no blank line before it, so a tight
///    list is one paragraph per item rather than one paragraph for the
///    whole list; its continuation lines (indented text with no bullet)
///    stay part of that item. Nested-subsection prose is already part of
///    `content` by the time it reaches this function (the same span
///    `min-words` counts over), so it is grouped exactly like top-level
///    section prose.
/// 2. At `per = sentence`, each paragraph is further split on `.`, `!`, or
///    `?` followed by whitespace or the paragraph's end.
///
/// `per = section` skips both steps: the whole text is the one unit.
fn unit_ranges(text: &str, per: DensityUnit) -> Vec<Range<usize>> {
    match per {
        DensityUnit::Section => std::iter::once(0..text.len()).collect(),
        DensityUnit::Paragraph => split_paragraphs(text),
        DensityUnit::Sentence => split_paragraphs(text)
            .into_iter()
            .flat_map(|paragraph| split_sentences(text, paragraph))
            .collect(),
    }
}

/// Drop every fenced code block (open/close lines and the lines between)
/// from `content`, keeping every other line verbatim including blank ones,
/// so paragraph boundaries around a removed fence are unaffected. A line
/// that opens with three or more backticks and closes them on the same line
/// is inline code, which [`mask_inline_code`] handles, not a fence.
fn strip_fenced_code(content: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        let opens_fence = trimmed.starts_with("```") && (in_fence || !closes_on_line(trimmed));
        if opens_fence {
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

/// Whether a line that opens with a run of backticks closes that run later
/// on the same line.
fn closes_on_line(line: &str) -> bool {
    let run = line.len() - line.trim_start_matches('`').len();
    line.get(run..)
        .is_some_and(|rest| rest.contains(&"`".repeat(run)))
}

/// `text` with the inside of every inline raw span replaced byte for byte by
/// [`MASK`], so a byte range means the same text in both.
///
/// Typst's raw syntax decides a span: one backtick closes at the next
/// backtick; two make an empty span; three or more close at the next run of
/// as many. A backslash escapes the character after it, so an escaped
/// backtick opens nothing, and a run that never closes is left as it is.
fn mask_inline_code(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(['`', '\\']) {
        let (before, from) = rest.split_at(at);
        out.push_str(before);
        rest = if let Some(escaped) = from.strip_prefix('\\') {
            out.push('\\');
            let next = escaped.chars().next().map_or(0, char::len_utf8);
            let (kept, after) = escaped.split_at(next);
            out.push_str(kept);
            after
        } else {
            mask_span(from, &mut out)
        };
    }
    out.push_str(rest);
    out
}

/// Copy the raw span `from` opens onto `out` with its inside masked, and
/// return the text after it. `from` starts with a backtick.
fn mask_span<'a>(from: &'a str, out: &mut String) -> &'a str {
    let run = from.len() - from.trim_start_matches('`').len();
    let (open, after_open) = from.split_at(run);
    out.push_str(open);
    if run == 2 {
        return after_open;
    }
    let Some(close) = after_open.find(open) else {
        return after_open;
    };
    let (inside, after_inside) = after_open.split_at(close);
    out.extend(std::iter::repeat_n(MASK, inside.len()));
    let (closing, after) = after_inside.split_at(run);
    out.push_str(closing);
    after
}

/// Group lines into paragraph ranges: a blank line always closes the current
/// one; a `- `/`+ ` list-item line closes it too (a new item starts even
/// without a blank line before it) and opens a new paragraph on itself;
/// every other line continues whichever paragraph is open.
fn split_paragraphs(text: &str) -> Vec<Range<usize>> {
    let mut paragraphs = Vec::new();
    let mut current: Option<Range<usize>> = None;
    let mut start = 0;
    for line in text.split_inclusive('\n') {
        let end = start + line.len();
        let trimmed = line.trim_start();
        let is_list_item = trimmed.starts_with("- ") || trimmed.starts_with("+ ");
        if line.trim().is_empty() || is_list_item {
            paragraphs.extend(current.take());
        }
        if !line.trim().is_empty() {
            current = Some(current.map_or(start..end, |open| open.start..end));
        }
        start = end;
    }
    paragraphs.extend(current);
    paragraphs
}

/// Split one paragraph range of `text` on `.`, `!`, or `?` followed by
/// whitespace or the paragraph's end, each sentence trimmed of whitespace.
fn split_sentences(text: &str, paragraph: Range<usize>) -> Vec<Range<usize>> {
    let body = text.get(paragraph.clone()).unwrap_or_default();
    let mut sentences = Vec::new();
    let mut start = 0;
    let mut chars = body.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        let ends = matches!(c, '.' | '!' | '?')
            && chars.peek().is_none_or(|(_, next)| next.is_whitespace());
        if ends {
            let end = at + c.len_utf8();
            push_trimmed(&mut sentences, body, start..end, paragraph.start);
            start = end;
        }
    }
    push_trimmed(&mut sentences, body, start..body.len(), paragraph.start);
    sentences
}

/// Push `range` of `body`, trimmed of whitespace and shifted by `offset`,
/// unless nothing but whitespace is left.
fn push_trimmed(sentences: &mut Vec<Range<usize>>, body: &str, range: Range<usize>, offset: usize) {
    let piece = body.get(range.clone()).unwrap_or_default();
    let lead = piece.len() - piece.trim_start().len();
    let kept = piece.trim().len();
    if kept > 0 {
        let start = offset + range.start + lead;
        sentences.push(start..start + kept);
    }
}

#[cfg(test)]
#[path = "content_gate_density_tests.rs"]
mod tests;
