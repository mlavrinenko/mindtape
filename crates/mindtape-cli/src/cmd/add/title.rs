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

/// Seed a template's title slot from the positional `title_words`, joined with
/// spaces. A higher-precedence value for that slot (`-f`/`--set`) wins; blank
/// input sets nothing. Returns the slot it filled, so the caller can count it
/// as explicit input. An aliased slot also keeps the core `title` value the
/// filename stem is minted from.
#[cfg(test)]
pub(super) fn fill_title_from_args(
    values: &mut BTreeMap<String, RawValue>,
    title_words: &[String],
) -> bool {
    fill_title_from_args_in(values, title_words, "title").is_some()
}

pub(super) fn fill_title_from_args_in(
    values: &mut BTreeMap<String, RawValue>,
    title_words: &[String],
    title_slot: &str,
) -> Option<String> {
    if values.contains_key(title_slot) || title_words.is_empty() {
        return None;
    }
    let title = title_words.join(" ");
    if title.trim().is_empty() {
        return None;
    }
    insert_title(values, title_slot, title);
    Some(title_slot.to_owned())
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

/// Read a piped title from stdin into the template's title slot. Gated to the
/// non-interactive pipe case: a TTY (no pipe) would block, and `-f -` already
/// claims stdin as JSON. A higher-precedence value for that slot
/// (`--set`/`-f`/env) wins.
#[cfg(test)]
pub(super) fn fill_title_from_stdin(
    values: &mut BTreeMap<String, RawValue>,
    args: &AddArgs,
) -> Result<bool> {
    Ok(fill_title_from_stdin_in(values, args, "title")?.is_some())
}

pub(super) fn fill_title_from_stdin_in(
    values: &mut BTreeMap<String, RawValue>,
    args: &AddArgs,
    title_slot: &str,
) -> Result<Option<String>> {
    if values.contains_key(title_slot)
        || std::io::stdin().is_terminal()
        || args.values.iter().any(|source| source == "-")
    {
        return Ok(None);
    }
    let mut raw = String::new();
    std::io::stdin()
        .read_to_string(&mut raw)
        .map_err(|err| Diag::failed("can't read the title from stdin").io(&err))?;
    let Some(title) = stdin_title(&raw) else {
        return Ok(None);
    };
    insert_title(values, title_slot, title);
    Ok(Some(title_slot.to_owned()))
}

/// Put `title` in `slot`, plus the core `title` key a stem mint reads when the
/// slot is aliased.
fn insert_title(values: &mut BTreeMap<String, RawValue>, slot: &str, title: String) {
    let value = RawValue::Str(title);
    if slot != "title" {
        values.insert("title".to_owned(), value.clone());
    }
    values.insert(slot.to_owned(), value);
}
