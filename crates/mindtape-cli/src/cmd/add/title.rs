//! The positional and piped sources of an `mt add` title: the `TITLE` words
//! with their `+tag` sugar, and a title piped on stdin.

use std::collections::BTreeMap;
use std::io::{IsTerminal, Read};

use mindtape::{RawValue, SlotSpec};

use super::AddArgs;
use super::values::stdin_title;
use crate::diag::{Diag, Result};

/// Split the positional `TITLE` words into the words that name the task and
/// their `+tag` sugar, when the template declares a `tags` slot.
///
/// A `+word` token (the Taskwarrior/todo.txt tag sigil) becomes a tag instead —
/// `mt add fix login +backend` sets `title: "fix login"` and appends
/// `"backend"` to `tags`. Without a `tags` slot the words are left untouched,
/// so a `+`-prefixed word never silently vanishes.
pub(super) fn split_positional(args: &AddArgs, specs: &[SlotSpec]) -> (Vec<String>, Vec<String>) {
    if specs.iter().any(|spec| spec.name == "tags") {
        split_tag_sugar(&args.title)
    } else {
        (args.title.clone(), Vec::new())
    }
}

/// Seed the `title` slot from the positional `title_words`, joined with spaces.
/// A higher-precedence `title` (`-f`/`--set`) wins; blank input sets nothing.
/// Returns whether it set one, so the caller can count it as explicit input.
pub(super) fn fill_title_from_args(
    values: &mut BTreeMap<String, RawValue>,
    title_words: &[String],
) -> bool {
    if values.contains_key("title") || title_words.is_empty() {
        return false;
    }
    let title = title_words.join(" ");
    if title.trim().is_empty() {
        return false;
    }
    values.insert("title".to_owned(), RawValue::Str(title));
    true
}

/// Split `+tag` words out of the positional title, in order. A bare `+` (no
/// tag text) is kept as a title word rather than treated as an empty tag.
fn split_tag_sugar(words: &[String]) -> (Vec<String>, Vec<String>) {
    let mut title_words = Vec::new();
    let mut tags = Vec::new();
    for word in words {
        match word.strip_prefix('+') {
            Some(tag) if !tag.is_empty() => tags.push(tag.to_owned()),
            _ => title_words.push(word.clone()),
        }
    }
    (title_words, tags)
}

/// Read a piped title from stdin into the `title` slot. Gated to the
/// non-interactive pipe case: a TTY (no pipe) would block, and `-f -` already
/// claims stdin as JSON. A higher-precedence `title` (`--set`/`-f`/env) wins.
pub(super) fn fill_title_from_stdin(
    values: &mut BTreeMap<String, RawValue>,
    args: &AddArgs,
) -> Result<bool> {
    if values.contains_key("title")
        || std::io::stdin().is_terminal()
        || args.values.iter().any(|source| source == "-")
    {
        return Ok(false);
    }
    let mut raw = String::new();
    std::io::stdin()
        .read_to_string(&mut raw)
        .map_err(|err| Diag::failed("can't read the title from stdin").io(&err))?;
    let Some(title) = stdin_title(&raw) else {
        return Ok(false);
    };
    values.insert("title".to_owned(), RawValue::Str(title));
    Ok(true)
}
