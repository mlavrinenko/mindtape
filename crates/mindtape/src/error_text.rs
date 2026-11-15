//! The one line of an error message that a reader needs.
//!
//! A regex engine, a Typst parse and a clap parse all print a multi-line
//! message whose reason sits on one line, prefixed with `error: `. A caller
//! that quotes the reason in its own message takes that line and drops the
//! prefix. The result then reads as a clause rather than as a second error.

/// The first line of `text`, without a leading `error: `.
#[must_use]
pub fn first_line(text: &str) -> String {
    let line = text.lines().next().unwrap_or_default();
    line.strip_prefix("error: ").unwrap_or(line).to_owned()
}

/// The last non-blank line of `text`, trimmed, without a leading `error: `.
#[must_use]
pub fn last_line(text: &str) -> String {
    let line = text
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(text)
        .trim();
    line.strip_prefix("error: ").unwrap_or(line).to_owned()
}
