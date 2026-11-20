//! Slot fill: coerce caller-supplied values against a discovered
//! [`super::SlotSpec`] and rewrite the source. Each `slot(...)` call becomes a
//! rendered literal (or its default). The filled task then holds plain values
//! and never calls `slot`.
//!
//! Split from `template/mod.rs` along the module's own two verbs (see its doc
//! comment). That file turns template source into [`super::SlotSpec`]s
//! ("discover"), and this one turns `SlotSpec`s plus caller values into
//! rewritten source ("fill"). [`fill()`] and [`fill_with()`] are the entry
//! points the parent module re-exports.
//!
//! # Files
//!
//! - `error.rs`: [`FillError`], why a fill refused a value.
//! - `markup.rs`: fill-time validation for `raw`-kind slots.

use std::collections::BTreeMap;
use std::ops::Range;

use typst_edit::{Edit, apply};

use crate::date::parse_ymd;

use super::{SlotKind, SlotSpec, discover_slots};

mod error;
pub use error::FillError;

/// Fill-time validation for `raw`-kind slots, split out to keep this file
/// under its line budget. See [`markup::render_raw`].
mod markup;

/// Whether [`fill`] refuses an unfilled required slot or leaves its `slot(...)`
/// call in place.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FillMode {
    /// Refuse a required slot with no value and no default, naming it in
    /// [`FillError::Missing`].
    #[default]
    Strict,
    /// Leave an unfilled required slot as its `slot(...)` call. The task then
    /// files incomplete, and `mt check` names what is missing.
    KeepMissing,
}

impl FillMode {
    /// The mode a run's verification decision implies. A run that verifies
    /// refuses an unfilled required slot, and one that skips verification
    /// leaves the `slot(...)` call in place.
    #[must_use]
    pub fn for_verification(verify: bool) -> Self {
        if verify {
            Self::Strict
        } else {
            Self::KeepMissing
        }
    }
}

/// One edge in a [`RawValue::Links`] value. Mirrors the dict the prelude's
/// `edge(...)` helper writes into `links:`: a target task path, a kind, and an
/// optional note.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkValue {
    /// Path from the filed task to the target task.
    pub task: String,
    /// Link kind, built-in or project-declared.
    pub kind: String,
    /// Optional edge note.
    pub note: Option<String>,
}

/// A caller-supplied fill value, before coercion to a slot's kind. Mirrors the
/// JSON scalar/array shapes a CLI front-end produces, keeping this crate free of
/// any JSON dependency.
#[derive(Debug, Clone, PartialEq)]
pub enum RawValue {
    /// A string (also the `--set k=v` shape, coerced per kind).
    Str(String),
    /// An integer.
    Int(i64),
    /// A float.
    Float(f64),
    /// A boolean.
    Bool(bool),
    /// An array of strings.
    List(Vec<String>),
    /// A list of typed edges.
    Links(Vec<LinkValue>),
}

impl RawValue {
    /// `pub(super)`: also read directly by `template/tests.rs`, a sibling
    /// module under `template`, not just by this file's own coercion helpers.
    pub(super) fn type_name(&self) -> &'static str {
        match self {
            Self::Str(_) => "string",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::List(_) => "list",
            Self::Links(_) => "links",
        }
    }
}

/// The replacement for an unfilled slot. A `text`/`raw` slot's string or
/// raw-block default becomes unquoted body markup, and every other default
/// splices verbatim.
fn default_replacement(spec: &SlotSpec) -> Option<String> {
    literal_replacement(spec, spec.default.as_ref(), spec.default_string.as_ref())
}

/// [`default_replacement`] for the slot's `example`.
fn example_replacement(spec: &SlotSpec) -> Option<String> {
    literal_replacement(spec, spec.example.as_ref(), spec.example_string.as_ref())
}

fn literal_replacement(
    spec: &SlotSpec,
    raw: Option<&String>,
    text: Option<&String>,
) -> Option<String> {
    let raw = raw?;
    match text {
        Some(text) if spec.kind.splices_verbatim() => Some(text.clone()),
        _ => Some(raw.clone()),
    }
}

/// Fill every slot in `source` with its `default`, else its `example`: the
/// task `mt add` files when the caller takes every default. A slot with no
/// default is one `mt add` has to ask for. Its example then fills it and the
/// task is complete.
///
/// # Errors
/// Returns [`FillError::Missing`] naming each slot with neither, and
/// [`FillError::UnknownKind`] for a slot whose kind names nothing.
pub fn fill_defaults(source: &str) -> Result<String, FillError> {
    fill_slots(source, |spec| {
        default_replacement(spec).or_else(|| example_replacement(spec))
    })
}

/// Fill every slot in `source` with its `example`, else its `default`: the
/// task a template files when every slot holds the value its author showed.
///
/// # Errors
/// Returns [`FillError::Missing`] naming each slot with neither, and
/// [`FillError::UnknownKind`] for a slot whose kind names nothing.
pub fn fill_examples(source: &str) -> Result<String, FillError> {
    fill_slots(source, |spec| {
        example_replacement(spec).or_else(|| default_replacement(spec))
    })
}

/// Fill each slot with `replacement`, naming every slot it cannot fill.
fn fill_slots(
    source: &str,
    replacement: impl Fn(&SlotSpec) -> Option<String>,
) -> Result<String, FillError> {
    let specs = discover_slots(source);
    reject_unknown_kind(&specs)?;
    let mut edits = Vec::new();
    let mut missing = Vec::new();
    for spec in &specs {
        match replacement(spec) {
            Some(text) => edits.push(Edit::new(edit_range(spec, source), text)),
            None => missing.push(spec.name.clone()),
        }
    }
    if !missing.is_empty() {
        return Err(FillError::Missing(missing));
    }
    apply(source, edits).map_err(|err| FillError::Edit(err.to_string()))
}

/// A slot whose declared kind names nothing would otherwise fill as a `string`.
/// That writes a wrong task silently rather than reporting a template error.
fn reject_unknown_kind(specs: &[SlotSpec]) -> Result<(), FillError> {
    match specs.iter().find(|spec| spec.unknown_kind.is_some()) {
        Some(spec) => Err(FillError::UnknownKind {
            slot: spec.name.clone(),
            kind: spec.unknown_kind.clone().unwrap_or_default(),
        }),
        None => Ok(()),
    }
}

/// Fill every slot in `source` from `values`, returning the rewritten task
/// source. An unfilled slot falls back to its default. A required slot with
/// neither value nor default is an error.
///
/// # Errors
/// Returns [`FillError`] when a value mismatches its slot kind, a required slot
/// stays unfilled, or the resulting changeset is invalid.
pub fn fill(source: &str, values: &BTreeMap<String, RawValue>) -> Result<String, FillError> {
    fill_with(source, values, FillMode::Strict)
}

/// [`fill`] in `mode`. [`FillMode::KeepMissing`] leaves an unfilled required
/// slot as its `slot(...)` call, so a task that skips verification files
/// incomplete instead of failing the fill.
///
/// # Errors
/// As [`fill`], except that [`FillMode::KeepMissing`] never returns
/// [`FillError::Missing`].
pub fn fill_with(
    source: &str,
    values: &BTreeMap<String, RawValue>,
    mode: FillMode,
) -> Result<String, FillError> {
    let specs = discover_slots(source);
    // Before any value is coerced.
    reject_unknown_kind(&specs)?;
    let mut edits = Vec::new();
    let mut missing = Vec::new();
    for spec in &specs {
        let replacement = match values.get(&spec.name) {
            Some(value) => render(spec, value)?,
            None => match default_replacement(spec) {
                Some(text) => text,
                None if spec.required => {
                    if mode == FillMode::Strict {
                        missing.push(spec.name.clone());
                    }
                    continue;
                }
                None => "none".to_owned(),
            },
        };
        edits.push(Edit::new(edit_range(spec, source), replacement));
    }
    if !missing.is_empty() {
        return Err(FillError::Missing(missing));
    }
    apply(source, edits).map_err(|err| FillError::Edit(err.to_string()))
}

/// Fill only the slots present in `values`, leaving every other `slot(...)`
/// call in place. A draft task built from the result still evaluates, since an
/// unfilled slot returns its `default` at eval time. `mt add` can then run its
/// hooks over a task that is not fully filled yet, then [`fill`] the rest.
///
/// # Errors
/// Returns [`FillError`] when a supplied value mismatches its slot kind, a slot
/// declares an unknown kind, or the resulting changeset is invalid.
pub fn fill_partial(
    source: &str,
    values: &BTreeMap<String, RawValue>,
) -> Result<String, FillError> {
    let specs = discover_slots(source);
    reject_unknown_kind(&specs)?;
    let mut edits = Vec::new();
    for spec in &specs {
        if let Some(value) = values.get(&spec.name) {
            edits.push(Edit::new(edit_range(spec, source), render(spec, value)?));
        }
    }
    apply(source, edits).map_err(|err| FillError::Edit(err.to_string()))
}

/// The byte range to replace for a slot. A markup `text`/`raw` slot extends
/// over the leading `#`, so the spliced body markup replaces the whole
/// `#slot(...)` call.
fn edit_range(spec: &SlotSpec, source: &str) -> Range<usize> {
    if spec.markup
        && spec.kind.splices_verbatim()
        && let Some(prev) = spec.range.start.checked_sub(1)
        && source.as_bytes().get(prev) == Some(&b'#')
    {
        return prev..spec.range.end;
    }
    spec.range.clone()
}

fn render(spec: &SlotSpec, value: &RawValue) -> Result<String, FillError> {
    match spec.kind {
        SlotKind::String => Ok(quote(&coerce_str(spec, value)?)),
        SlotKind::Text => coerce_str(spec, value),
        SlotKind::Raw => markup::render_raw(spec, value),
        SlotKind::Choice => render_choice(spec, value),
        SlotKind::Int => Ok(coerce_int(spec, value)?.to_string()),
        SlotKind::Float | SlotKind::Real => Ok(render_float(coerce_float(spec, value)?)),
        SlotKind::Bool => Ok(coerce_bool(spec, value)?.to_string()),
        SlotKind::Date => render_date(spec, &coerce_str(spec, value)?),
        SlotKind::List => Ok(render_list(&coerce_list(spec, value)?)),
        SlotKind::MultiChoice => render_multi_choice(spec, value),
        SlotKind::Links => Ok(render_links(&coerce_links(spec, value)?)),
    }
}

fn render_multi_choice(spec: &SlotSpec, value: &RawValue) -> Result<String, FillError> {
    let items = coerce_list(spec, value)?;
    if let Some(choices) = &spec.choices
        && let Some(bad) = items.iter().find(|item| !choices.contains(item))
    {
        return Err(FillError::BadChoice {
            slot: spec.name.clone(),
            value: bad.clone(),
            choices: choices.clone(),
        });
    }
    Ok(render_list(&items))
}

fn render_choice(spec: &SlotSpec, value: &RawValue) -> Result<String, FillError> {
    let text = coerce_str(spec, value)?;
    if let Some(choices) = &spec.choices
        && !choices.contains(&text)
    {
        return Err(FillError::BadChoice {
            slot: spec.name.clone(),
            value: text,
            choices: choices.clone(),
        });
    }
    Ok(quote(&text))
}

fn render_date(spec: &SlotSpec, text: &str) -> Result<String, FillError> {
    let (year, month, day) = parse_ymd(text).ok_or_else(|| FillError::BadDate {
        slot: spec.name.clone(),
        value: text.to_owned(),
    })?;
    Ok(format!("on({year}, {month}, {day})"))
}

fn render_links(items: &[LinkValue]) -> String {
    let rendered: Vec<String> = items
        .iter()
        .map(|link| {
            let note = link
                .note
                .as_deref()
                .map_or_else(|| "none".to_owned(), quote);
            format!(
                "(task: {}, kind: {}, note: {note})",
                quote(&link.task),
                quote(&link.kind)
            )
        })
        .collect();
    render_sequence(&rendered)
}

fn render_list(items: &[String]) -> String {
    let rendered: Vec<String> = items.iter().map(|item| quote(item)).collect();
    render_sequence(&rendered)
}

/// Join already-rendered elements as a Typst array, with the trailing comma.
fn render_sequence(items: &[String]) -> String {
    if items.is_empty() {
        return "()".to_owned();
    }
    format!("({},)", items.join(", "))
}

fn render_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        format!("{value}")
    }
}

fn coerce_str(spec: &SlotSpec, value: &RawValue) -> Result<String, FillError> {
    match value {
        RawValue::Str(text) => Ok(text.clone()),
        RawValue::Int(num) => Ok(num.to_string()),
        RawValue::Float(num) => Ok(render_float(*num)),
        RawValue::Bool(flag) => Ok(flag.to_string()),
        RawValue::List(_) | RawValue::Links(_) => Err(type_error(spec, value)),
    }
}

fn coerce_int(spec: &SlotSpec, value: &RawValue) -> Result<i64, FillError> {
    match value {
        RawValue::Int(num) => Ok(*num),
        RawValue::Str(text) => text.parse().map_err(|_| type_error(spec, value)),
        _ => Err(type_error(spec, value)),
    }
}

fn coerce_float(spec: &SlotSpec, value: &RawValue) -> Result<f64, FillError> {
    match value {
        RawValue::Float(num) => Ok(*num),
        RawValue::Int(num) => Ok(*num as f64),
        RawValue::Str(text) => text.parse().map_err(|_| type_error(spec, value)),
        _ => Err(type_error(spec, value)),
    }
}

fn coerce_bool(spec: &SlotSpec, value: &RawValue) -> Result<bool, FillError> {
    match value {
        RawValue::Bool(flag) => Ok(*flag),
        RawValue::Str(text) => text.parse().map_err(|_| type_error(spec, value)),
        _ => Err(type_error(spec, value)),
    }
}

fn coerce_list(spec: &SlotSpec, value: &RawValue) -> Result<Vec<String>, FillError> {
    match value {
        RawValue::List(items) => Ok(items.clone()),
        _ => Err(type_error(spec, value)),
    }
}

fn coerce_links(spec: &SlotSpec, value: &RawValue) -> Result<Vec<LinkValue>, FillError> {
    match value {
        RawValue::Links(items) => Ok(items.clone()),
        RawValue::List(items) if items.is_empty() => Ok(Vec::new()),
        _ => Err(type_error(spec, value)),
    }
}

fn type_error(spec: &SlotSpec, value: &RawValue) -> FillError {
    FillError::Type {
        slot: spec.name.clone(),
        want: spec.kind,
        got: value.type_name(),
    }
}

/// Quote and escape a string into a Typst string literal.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}
