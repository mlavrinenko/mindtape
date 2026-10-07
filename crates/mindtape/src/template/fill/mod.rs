//! Slot fill: coerce caller-supplied values against a discovered
//! [`super::SlotSpec`] and rewrite the source, replacing each `slot(...)`
//! call with a rendered literal (or its default) so the filled task holds
//! plain values and never calls `slot`.
//!
//! Split from `template/mod.rs` along the module's own two verbs (see its doc
//! comment): that file turns template source into [`super::SlotSpec`]s
//! ("discover"), this one turns `SlotSpec`s plus caller values into
//! rewritten source ("fill"). [`fill`] is the sole entry point the parent
//! module re-exports.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Range;

use typst_edit::{Edit, apply};

use crate::date::parse_ymd;

use super::{SlotKind, SlotSpec, discover_slots};

/// Fill-time validation for `raw`-kind slots; split out to keep this file
/// under its line budget. See [`markup::render_raw`].
mod markup;

/// A caller-supplied fill value, before coercion to a slot's kind. Mirrors the
/// JSON scalar/array shapes a CLI front-end produces, keeping this crate free of
/// any JSON dependency.
#[derive(Debug, Clone, PartialEq)]
pub enum RawValue {
    /// A string (also the `--set k=v` shape; coerced per kind).
    Str(String),
    /// An integer.
    Int(i64),
    /// A float.
    Float(f64),
    /// A boolean.
    Bool(bool),
    /// An array of strings.
    List(Vec<String>),
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
        }
    }
}

/// Why a [`fill`] was rejected.
#[derive(Debug, Clone, PartialEq)]
pub enum FillError {
    /// Required slots with no value and no default, by name.
    Missing(Vec<String>),
    /// A value could not be coerced to the slot's kind.
    Type {
        /// Slot name.
        slot: String,
        /// Expected kind.
        want: SlotKind,
        /// The supplied value's shape.
        got: &'static str,
    },
    /// A `choice` value outside the declared `choices`.
    BadChoice {
        /// Slot name.
        slot: String,
        /// The rejected value.
        value: String,
        /// The allowed values.
        choices: Vec<String>,
    },
    /// A `date` value that is not `YYYY-MM-DD`.
    BadDate {
        /// Slot name.
        slot: String,
        /// The rejected value.
        value: String,
    },
    /// A slot declared a `kind:` that names no known kind.
    UnknownKind {
        /// Slot name.
        slot: String,
        /// The unrecognised spelling.
        kind: String,
    },
    /// A `raw` slot's value failed to parse as Typst markup.
    BadMarkup {
        /// Slot name.
        slot: String,
        /// The parser's own message for the first error.
        message: String,
    },
    /// The rewrite changeset was invalid (overlap / bounds).
    Edit(String),
}

impl fmt::Display for FillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(names) => write!(f, "missing required slot(s): {}", names.join(", ")),
            Self::Type {
                slot,
                want: SlotKind::List | SlotKind::MultiChoice,
                got,
            } => write!(
                f,
                "slot {slot:?} expects a list; use --set {slot}+=value (repeatable) or \
                 -f/--values with a JSON/YAML array, got {got}"
            ),
            Self::Type { slot, want, got } => {
                write!(f, "slot {slot:?} expects {}, got {got}", want.as_str())
            }
            Self::BadChoice {
                slot,
                value,
                choices,
            } => write!(
                f,
                "slot {slot:?} value {value:?} is not one of: {}",
                choices.join(", ")
            ),
            Self::BadDate { slot, value } => {
                write!(f, "slot {slot:?} value {value:?} is not a YYYY-MM-DD date")
            }
            Self::UnknownKind { slot, kind } => write!(
                f,
                "slot {slot:?} declares unknown kind {kind:?}; expected one of: \
                 string, int, float, real, bool, date, list, text, raw, choice"
            ),
            Self::BadMarkup { slot, message } => {
                write!(f, "slot {slot:?} is not valid Typst markup: {message}")
            }
            Self::Edit(message) => write!(f, "rewrite failed: {message}"),
        }
    }
}

impl std::error::Error for FillError {}

/// The replacement for an unfilled slot: a `text`/`raw` slot's string or
/// raw-block default becomes unquoted body markup; every other default
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

/// Fill every slot in `source` with its `example`, else its `default`: the
/// task a template files when every slot holds the value its author showed,
/// which is what `mt check` validates.
///
/// # Errors
/// Returns [`FillError::Missing`] naming each slot with neither, and
/// [`FillError::UnknownKind`] for a slot whose kind names nothing.
pub fn fill_examples(source: &str) -> Result<String, FillError> {
    let specs = discover_slots(source);
    reject_unknown_kind(&specs)?;
    let mut edits = Vec::new();
    let mut missing = Vec::new();
    for spec in &specs {
        match example_replacement(spec).or_else(|| default_replacement(spec)) {
            Some(text) => edits.push(Edit::new(edit_range(spec, source), text)),
            None => missing.push(spec.name.clone()),
        }
    }
    if !missing.is_empty() {
        return Err(FillError::Missing(missing));
    }
    apply(source, edits).map_err(|err| FillError::Edit(err.to_string()))
}

/// A slot whose declared kind names nothing would otherwise be filled as a
/// `string`, which is a wrong task written silently rather than a template
/// error reported.
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
/// source. An unfilled slot falls back to its default; a required slot with
/// neither value nor default is an error.
///
/// # Errors
/// Returns [`FillError`] when a value mismatches its slot kind, a required slot
/// is unfilled, or the resulting changeset is invalid.
pub fn fill(source: &str, values: &BTreeMap<String, RawValue>) -> Result<String, FillError> {
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
                    missing.push(spec.name.clone());
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

/// The byte range to replace for a slot. A markup `text`/`raw` slot extends
/// over the leading `#` so the spliced body markup stands alone.
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

fn render_list(items: &[String]) -> String {
    if items.is_empty() {
        return "()".to_owned();
    }
    let rendered: Vec<String> = items.iter().map(|item| quote(item)).collect();
    format!("({},)", rendered.join(", "))
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
        RawValue::List(_) => Err(type_error(spec, value)),
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
