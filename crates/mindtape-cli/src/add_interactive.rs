//! Interactive editor fill for `mt add`.
//!
//! Builds a commented YAML scaffold from a template's discovered slots, opens it
//! in the user's editor, and parses the saved buffer back into fill values. The
//! `mindtape` library stays format-blind; this glue lives in the CLI.

use std::collections::BTreeMap;
use std::io::Write;
use std::process::Command;

use anyhow::{Context, Result, bail};
use mindtape::{AddConfig, InteractiveMode, RawValue, SlotKind, SlotSpec};

use crate::yaml_values::parse_mapping;

const DEFAULT_HEADER: &str = "\
# mt add — fill the values below, then save and quit.
# '#' lines are ignored. Leave a value empty to use its default (or to skip it).
# 'text' slots use a YAML block scalar (|-); keep the two-space indentation.";

/// Decide whether `add` should open the interactive editor.
///
/// `force` is the `--edit` flag: it overrides a `never` setting and the `auto`
/// missing-slot trigger, but still requires a terminal. The non-interactive path
/// returns `false`.
///
/// `tty` should be `std::io::stdin().is_terminal() && std::io::stdout().is_terminal()`.
///
/// # Errors
/// Returns an error when an editor is demanded (`--edit`, or `interactive` set to
/// `always`) but `tty` is false.
pub fn should_open(
    force: bool,
    config: &AddConfig,
    specs: &[SlotSpec],
    seed: &BTreeMap<String, RawValue>,
    tty: bool,
) -> Result<bool> {
    if config.interactive == InteractiveMode::Never && !force {
        return Ok(false);
    }
    if force || config.interactive == InteractiveMode::Always {
        if !tty {
            bail!("interactive add needs a terminal; pass values with -f/--set instead");
        }
        return Ok(true);
    }
    Ok(tty && has_unfilled_required(specs, seed))
}

fn has_unfilled_required(specs: &[SlotSpec], seed: &BTreeMap<String, RawValue>) -> bool {
    specs
        .iter()
        .any(|spec| is_unfilled_required(spec) && !seed.contains_key(&spec.name))
}

/// Open the editor on a scaffold for `specs` pre-filled from `seed`, returning
/// the values the user saved.
///
/// # Errors
/// Returns an error when no editor can be spawned, the editor exits without
/// saving, or the saved buffer is not a valid YAML mapping.
pub fn run(
    config: &AddConfig,
    specs: &[SlotSpec],
    seed: &BTreeMap<String, RawValue>,
) -> Result<BTreeMap<String, RawValue>> {
    let buffer = scaffold(config, specs, seed);
    let command = editor_command(config.editor.as_deref());
    let edited = open_in_editor(&command, &buffer)?;
    parse_buffer(&edited)
}

/// Last-resort editor when neither `[add] editor` nor `$VISUAL`/`$EDITOR` says
/// anything: the one editor the platform is guaranteed to have. `vi` is
/// POSIX-mandated and absent from Windows, where `notepad` is the equivalent
/// floor — it also blocks until closed, which is what this path needs.
#[cfg(not(windows))]
const FALLBACK_EDITOR: &str = "vi";
#[cfg(windows)]
const FALLBACK_EDITOR: &str = "notepad";

/// Resolve the editor command: config override, then `$VISUAL`, then `$EDITOR`,
/// then [`FALLBACK_EDITOR`].
fn editor_command(override_cmd: Option<&str>) -> String {
    if let Some(cmd) = override_cmd
        && !cmd.trim().is_empty()
    {
        return cmd.trim().to_owned();
    }
    for var in ["VISUAL", "EDITOR"] {
        if let Ok(value) = std::env::var(var)
            && !value.trim().is_empty()
        {
            return value;
        }
    }
    FALLBACK_EDITOR.to_owned()
}

/// Write `initial` to a scratch `.yaml` file, run `command` on it, and read it
/// back after the editor exits.
fn open_in_editor(command: &str, initial: &str) -> Result<String> {
    let mut file = tempfile::Builder::new()
        .prefix("mindtape-add-")
        .suffix(".yaml")
        .tempfile()
        .context("create scratch buffer")?;
    file.write_all(initial.as_bytes())
        .context("write scratch buffer")?;
    file.flush().context("flush scratch buffer")?;
    let path = file.into_temp_path();

    let mut parts = command.split_whitespace();
    let Some(program) = parts.next() else {
        bail!("empty editor command");
    };
    let status = Command::new(program)
        .args(parts)
        .arg(&*path)
        .status()
        .with_context(|| format!("spawn editor {command:?}"))?;
    if !status.success() {
        bail!("editor {command:?} exited without saving");
    }
    std::fs::read_to_string(&path).context("read edited buffer")
}

/// Render the YAML scaffold: the (optional) header plus one entry per slot,
/// blocks separated by a blank line. The tail carries no trailing newline so the
/// editor's jump-to-end lands on the last fillable line.
fn scaffold(config: &AddConfig, specs: &[SlotSpec], seed: &BTreeMap<String, RawValue>) -> String {
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
/// comment is dropped when `comments` is false, leaving a bare `name: value`.
fn entry_block(spec: &SlotSpec, seed: Option<&RawValue>, comments: bool) -> String {
    let mut block = String::new();
    if comments {
        push_comment(&mut block, spec);
    }
    push_entry(&mut block, spec, seed);
    block
}

/// Resolve the scaffold header: empty when comments are disabled, the built-in
/// default when unset, empty when the config disables it, else the custom text
/// comment-prefixed.
fn header_text(config: &AddConfig) -> String {
    if !config.comments {
        return String::new();
    }
    match config.header.as_deref() {
        None => DEFAULT_HEADER.to_owned(),
        Some(custom) => commentify(custom),
    }
}

/// Comment-prefix each non-empty line of a custom header so it stays valid YAML;
/// an all-blank header yields the empty string (no header).
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
fn is_unfilled_required(spec: &SlotSpec) -> bool {
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
        out.push_str(&format!("# default: {default}\n"));
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
}

fn push_entry(out: &mut String, spec: &SlotSpec, seed: Option<&RawValue>) {
    match entry_value(spec, seed) {
        Some(Entry::Scalar(text)) => out.push_str(&format!("{}: {text}\n", spec.name)),
        Some(Entry::Block(body)) => push_block(out, &spec.name, &body),
        Some(Entry::List(items)) => push_list(out, &spec.name, &items),
        // Trailing space leaves the cursor at the value position (and pre-places
        // the `: ` YAML mappings require) when the editor jumps to the tail.
        None => out.push_str(&format!("{}: \n", spec.name)),
    }
}

/// Resolve the pre-fill value: a seed value wins, else the slot default. `text`
/// slots always get a block scalar so the editor shows the `|-` affordance.
fn entry_value(spec: &SlotSpec, seed: Option<&RawValue>) -> Option<Entry> {
    if let Some(raw) = seed {
        return Some(seed_entry(spec.kind, raw));
    }
    if spec.kind == SlotKind::Text {
        return Some(Entry::Block(
            spec.default_string.clone().unwrap_or_default(),
        ));
    }
    default_entry(spec)
}

fn seed_entry(kind: SlotKind, raw: &RawValue) -> Entry {
    match raw {
        RawValue::Str(text) if kind == SlotKind::Text => Entry::Block(text.clone()),
        RawValue::Str(text) => Entry::Scalar(quote(text)),
        RawValue::Int(value) => Entry::Scalar(value.to_string()),
        RawValue::Float(value) => Entry::Scalar(value.to_string()),
        RawValue::Bool(value) => Entry::Scalar(value.to_string()),
        RawValue::List(items) => Entry::List(items.clone()),
    }
}

/// Pre-fill from a non-`text` slot's default. String/choice defaults come from
/// the unescaped `default_string`; numeric and bool defaults pass through the raw
/// literal; an empty/absent list default renders as an explicit `[]` so the list
/// kind is visible; a dated default and a non-empty list default are left blank
/// (the comment shows the default).
fn default_entry(spec: &SlotSpec) -> Option<Entry> {
    if let Some(text) = &spec.default_string {
        return Some(Entry::Scalar(quote(text)));
    }
    match spec.kind {
        SlotKind::Int | SlotKind::Float | SlotKind::Real | SlotKind::Bool => {
            spec.default.clone().map(Entry::Scalar)
        }
        SlotKind::List if is_empty_list_default(spec) => Some(Entry::List(Vec::new())),
        _ => None,
    }
}

/// A list slot whose default is empty (`()`) or absent — so an empty `[]` in the
/// scaffold matches its default and leaving it untouched changes nothing. A
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

fn push_list(out: &mut String, name: &str, items: &[String]) {
    if items.is_empty() {
        out.push_str(&format!("{name}: []\n"));
        return;
    }
    out.push_str(&format!("{name}:\n"));
    for item in items {
        out.push_str(&format!("  - {}\n", quote(item)));
    }
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

/// Parse a saved buffer into fill values, dropping null/empty entries. The
/// YAML -> `RawValue` conversion itself lives in `yaml_values` (shared with
/// the `-f/--values` YAML path); only the empty-string drop is specific to
/// this caller — blank means "use the default" in the editor scaffold, but a
/// `-f` file's `""` is a real value the other caller must keep.
///
/// # Errors
/// Returns an error when the buffer is not a YAML mapping, a key is non-string,
/// or a value has an unsupported shape (nested mapping, non-string list item).
fn parse_buffer(text: &str) -> Result<BTreeMap<String, RawValue>> {
    let values = parse_mapping(
        text,
        || "parse the YAML buffer".to_owned(),
        || "the buffer must be a YAML mapping of slot names to values".to_owned(),
    )?;
    Ok(drop_blank(values))
}

/// Drop entries the editor treats as "not filled": a `null` (already `None`
/// from the shared converter) and an empty string (kept as a real value by
/// the shared converter for the `-f` caller, but blank here means "fall
/// through to the default").
fn drop_blank(values: BTreeMap<String, Option<RawValue>>) -> BTreeMap<String, RawValue> {
    values
        .into_iter()
        .filter_map(|(key, value)| {
            let raw = value?;
            if matches!(&raw, RawValue::Str(text) if text.is_empty()) {
                return None;
            }
            Some((key, raw))
        })
        .collect()
}

#[cfg(test)]
#[path = "add_interactive_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "add_interactive_tests_editor.rs"]
mod tests_editor;
