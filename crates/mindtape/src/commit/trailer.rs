//! One trailer value, read as refs: the split every commit check and the
//! `touches()` index share.
//!
//! Commas and whitespace separate refs. A double-quoted stretch keeps either
//! separator, and a backslash outside quotes escapes a quote, backslash,
//! comma or whitespace. A bare value is a stem as written; a path's file stem
//! names the task. `[commits] external-refs` patterns keep a value opaque.

use regex::Regex;

/// The stem a trailer value names: a bare value as written, a path's file stem.
pub(crate) fn value_stem(value: &str) -> String {
    if !is_path_form(value) {
        return value.to_owned();
    }
    std::path::Path::new(value).file_stem().map_or_else(
        || value.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

/// `value` as a trailer writes it: quoted and escaped when it holds a
/// separator, quote or backslash.
pub(crate) fn ref_text(value: &str) -> String {
    if value
        .chars()
        .any(|c| c == ',' || c == '"' || c == '\\' || c.is_whitespace())
    {
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    } else {
        value.to_owned()
    }
}

/// Whether a trailer value is a path, not a bare stem.
pub(crate) fn is_path_form(value: &str) -> bool {
    value.contains(['/', '\\']) || value.ends_with(".typ")
}

/// Split one trailer value into refs: commas and whitespace separate, a
/// double-quoted stretch keeps them, and a backslash escapes a quote,
/// backslash, comma or whitespace.
pub(crate) fn split_refs(value: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => quoted = !quoted,
            '\\' => match chars.peek().copied() {
                Some(next)
                    if next == '"' || next == '\\' || next == ',' || next.is_whitespace() =>
                {
                    current.push(next);
                    chars.next();
                }
                _ => current.push('\\'),
            },
            c if !quoted && (c == ',' || c.is_whitespace()) => {
                if !current.is_empty() {
                    refs.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        refs.push(current);
    }
    refs
}

/// Compile the project's `[commits] external-refs` patterns. Load-time
/// validation refuses a bad one, so an uncompilable pattern left in a config
/// that skipped validation matches nothing.
pub(crate) fn external_matchers(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
}

/// Whether `value` matches one of the compiled external-ref patterns.
pub(crate) fn matches_external(value: &str, matchers: &[Regex]) -> bool {
    matchers.iter().any(|matcher| matcher.is_match(value))
}

#[cfg(test)]
#[path = "trailer_tests.rs"]
mod tests;
