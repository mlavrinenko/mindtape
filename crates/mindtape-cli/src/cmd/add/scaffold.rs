//! The YAML scaffold `mt add` opens in the editor.
//!
//! One entry per discovered slot, pre-filled from the seed values and the
//! slot defaults, with a comment above each entry when `[add] comments` is on.
//! The buffer is plain YAML so the editor's own syntax highlighting and the
//! parse-back in [`super::interactive`] agree on what a value is.

use std::collections::BTreeMap;

use mindtape::{AddConfig, LinkValue, RawValue, SlotKind, SlotSpec, parse_ymd};

use crate::cmd::add::values::example_of;

const DEFAULT_HEADER: &str = "\
# mt add — fill the values below, then save and quit.
# '#' lines are ignored. Leave a value empty to use its default (or to skip it).
# 'text' and 'raw' slots use a YAML block scalar (|-); keep the two-space indentation.";

/// Render the YAML scaffold: the (optional) header plus one entry per slot,
/// blocks separated by a blank line. The tail carries no trailing newline so the
/// editor's jump-to-end lands on the last fillable line.
pub(super) fn scaffold(
    config: &AddConfig,
    specs: &[SlotSpec],
    seed: &BTreeMap<String, RawValue>,
) -> String {
    let mut out = String::new();
    let header = header_text(config);
    if !header.is_empty() {
        out.push_str(&header);
        out.push_str("\n\n");
    }
    let mut ordered: Vec<&SlotSpec> = specs.iter().collect();
    ordered.sort_by_key(|spec| !is_unfilled_required(spec));
    let blocks: Vec<String> = ordered
        .iter()
        .map(|spec| entry_block(spec, seed.get(&spec.name), config.comments))
        .collect();
    // With comments on, a blank line spaces out each comment+value group. With
    // comments off there is nothing to separate, so pack the bare mapping tight.
    let separator = if config.comments { "\n" } else { "" };
    out.push_str(&blocks.join(separator));
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

/// One slot's (optional) comment + value entry, ending with a newline. The
/// function drops the comment when `comments` is false, leaving a bare
/// `name: value`.
fn entry_block(spec: &SlotSpec, seed: Option<&RawValue>, comments: bool) -> String {
    let mut block = String::new();
    if comments {
        push_comment(&mut block, spec);
    }
    push_entry(&mut block, spec, seed);
    block
}

/// Resolve the scaffold header: empty when the config disables comments, the
/// built-in default when unset, empty when the config disables it, else the
/// custom text comment-prefixed.
fn header_text(config: &AddConfig) -> String {
    if !config.comments {
        return String::new();
    }
    match config.header.as_deref() {
        None => DEFAULT_HEADER.to_owned(),
        Some(custom) => commentify(custom),
    }
}

/// Comment-prefix each non-empty line of a custom header so it stays valid YAML.
/// An all-blank header yields the empty string (no header).
fn commentify(text: &str) -> String {
    let body = text.trim_end_matches('\n');
    if body.trim().is_empty() {
        return String::new();
    }
    body.lines()
        .map(|line| {
            if line.is_empty() || line.trim_start().starts_with('#') {
                line.to_owned()
            } else {
                format!("# {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A slot still needs a value: declared `required` and no default to fall
/// back on. Used both for the scaffold's `required`/`optional` badge and to
/// sort required slots to the front of the scaffold.
pub(super) fn is_unfilled_required(spec: &SlotSpec) -> bool {
    spec.required && spec.default.is_none()
}

fn push_comment(out: &mut String, spec: &SlotSpec) {
    let need = if is_unfilled_required(spec) {
        "required"
    } else {
        "optional"
    };
    let label = spec.prompt.as_deref().unwrap_or(&spec.name);
    out.push_str(&format!("# {label} [{}, {need}]\n", spec.kind.as_str()));
    if let Some(choices) = &spec.choices {
        out.push_str(&format!("# choices: {}\n", choices.join(", ")));
    }
    if let Some(default) = &spec.default {
        let shown = if spec.kind.splices_verbatim() {
            spec.default_string.as_deref().unwrap_or(default)
        } else {
            default
        };
        push_comment_lines(out, &format!("default: {shown}"));
    }
    push_shape(out, spec);
}

/// The sample a `date` slot shows when it doesn't declare an `example:`.
const DATE_SAMPLE: &str = "2026-11-17";

/// The format a `date` slot's value takes.
const DATE_FORMAT: &str = "YYYY-MM-DD";

/// The line that shows what a good answer looks like. A `date` slot spells its
/// format, because a wrong one reads as valid until the fill rejects it. Every
/// other kind prints its `example:`, which the template author wrote for it.
fn push_shape(out: &mut String, spec: &SlotSpec) {
    let example = example_of(spec);
    if spec.kind == SlotKind::Date {
        let sample = example.unwrap_or(DATE_SAMPLE);
        push_comment_lines(out, &format!("{DATE_FORMAT}, e.g. {sample}"));
    } else if let Some(example) = example {
        push_comment_lines(out, &format!("e.g. {example}"));
    }
}

/// Push `text` as `#`-prefixed lines. A multi-line example then stays a
/// comment, and the buffer stays a YAML mapping.
fn push_comment_lines(out: &mut String, text: &str) {
    for line in text.lines() {
        if line.is_empty() {
            out.push_str("#\n");
        } else {
            out.push_str(&format!("# {line}\n"));
        }
    }
}

/// One scaffold value, shaped by the slot's kind.
enum Entry {
    /// A single-line scalar, already YAML-encoded.
    Scalar(String),
    /// A multi-line `|-` block scalar body.
    Block(String),
    /// A YAML sequence of strings.
    List(Vec<String>),
    /// A YAML sequence of typed edges.
    Links(Vec<LinkValue>),
}

fn push_entry(out: &mut String, spec: &SlotSpec, seed: Option<&RawValue>) {
    match entry_value(spec, seed) {
        Some(Entry::Scalar(text)) => out.push_str(&format!("{}: {text}\n", spec.name)),
        Some(Entry::Block(body)) => push_block(out, &spec.name, &body),
        Some(Entry::List(items)) => push_list(out, &spec.name, &items),
        Some(Entry::Links(items)) => push_links(out, &spec.name, &items),
        // Trailing space leaves the cursor at the value position (and pre-places
        // the `: ` YAML mappings require) when the editor jumps to the tail.
        None => out.push_str(&format!("{}: \n", spec.name)),
    }
}

/// Resolve the pre-fill value: a seed value wins, else the slot default. The
/// kinds that splice body markup always get a block scalar. A multi-line value
/// then stays one YAML block and the editor shows the `|-` affordance.
fn entry_value(spec: &SlotSpec, seed: Option<&RawValue>) -> Option<Entry> {
    if let Some(raw) = seed {
        return Some(seed_entry(spec, raw));
    }
    if spec.kind.splices_verbatim() {
        return Some(Entry::Block(
            spec.default_string.clone().unwrap_or_default(),
        ));
    }
    default_entry(spec)
}

fn seed_entry(spec: &SlotSpec, raw: &RawValue) -> Entry {
    match raw {
        RawValue::Str(text) if spec.kind.splices_verbatim() => Entry::Block(text.clone()),
        RawValue::Str(text) => Entry::Scalar(quote(text)),
        RawValue::Int(value) => Entry::Scalar(value.to_string()),
        RawValue::Float(value) => Entry::Scalar(value.to_string()),
        RawValue::Bool(value) => Entry::Scalar(value.to_string()),
        RawValue::List(items) => Entry::List(items.clone()),
        RawValue::Links(items) => Entry::Links(items.clone()),
    }
}

/// Pre-fill from a non-`text` slot's default. String and choice defaults come
/// from the unescaped `default_string`. Numeric and bool defaults pass through
/// the raw literal. A `date` default pre-fills when it is a `YYYY-MM-DD`
/// literal, the one spelling the fill reads back. The editor then shows the
/// shape. An empty or absent list default renders as an explicit `[]` so the
/// list kind is visible. A non-empty list default, and a date default in any
/// other spelling, stay blank, and the comment shows the default.
fn default_entry(spec: &SlotSpec) -> Option<Entry> {
    if let Some(text) = &spec.default_string {
        return Some(Entry::Scalar(quote(text)));
    }
    match spec.kind {
        SlotKind::Int | SlotKind::Float | SlotKind::Real | SlotKind::Bool => {
            spec.default.clone().map(Entry::Scalar)
        }
        SlotKind::Date => spec
            .default
            .as_deref()
            .filter(|source| parse_ymd(source).is_some())
            .map(ToOwned::to_owned)
            .map(Entry::Scalar),
        SlotKind::List | SlotKind::MultiChoice if is_empty_list_default(spec) => {
            Some(Entry::List(Vec::new()))
        }
        SlotKind::Links if is_empty_list_default(spec) => Some(Entry::Links(Vec::new())),
        _ => None,
    }
}

/// A list slot whose default is empty (`()`) or absent, so an empty `[]` in the
/// scaffold matches its default and an untouched slot keeps that default. A
/// non-empty list default stays blank so an untouched slot falls through to it.
fn is_empty_list_default(spec: &SlotSpec) -> bool {
    match &spec.default {
        None => true,
        Some(source) => source.trim() == "()",
    }
}

fn push_block(out: &mut String, name: &str, body: &str) {
    out.push_str(&format!("{name}: |-\n"));
    for line in body.lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str(&format!("  {line}\n"));
        }
    }
}

fn push_links(out: &mut String, name: &str, links: &[LinkValue]) {
    if !push_sequence_start(out, name, links.is_empty()) {
        return;
    }
    for link in links {
        out.push_str(&format!("  - kind: {}\n", quote(&link.kind)));
        out.push_str(&format!("    task: {}\n", quote(&link.task)));
        if let Some(note) = &link.note {
            out.push_str(&format!("    note: {}\n", quote(note)));
        }
    }
}

fn push_list(out: &mut String, name: &str, items: &[String]) {
    if !push_sequence_start(out, name, items.is_empty()) {
        return;
    }
    for item in items {
        out.push_str(&format!("  - {}\n", quote(item)));
    }
}

/// Open a YAML sequence entry, or write `[]` and return `false` when it has no
/// items. The links and string-list scaffolds share the shape.
fn push_sequence_start(out: &mut String, name: &str, empty: bool) -> bool {
    if empty {
        out.push_str(&format!("{name}: []\n"));
        return false;
    }
    out.push_str(&format!("{name}:\n"));
    true
}

/// Encode `text` as a YAML double-quoted scalar.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
#[path = "scaffold_tests.rs"]
pub(crate) mod tests;
