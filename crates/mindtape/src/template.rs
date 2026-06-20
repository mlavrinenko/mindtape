//! Slot-template engine: discover `slot(...)` calls and fill them by source
//! rewrite.
//!
//! A template is an ordinary task artifact whose authored values are `slot(...)`
//! calls (see the prelude helper). This module finds each call on the parse plane
//! (`typst_edit::find_calls` — no eval, no `World`), reads its literal arguments
//! into a [`SlotSpec`], and [`fill`]s the source: each slot is replaced by a
//! rendered literal so the filed task holds plain values and never calls `slot`.
//!
//! Slot arguments must be literals; this engine never evaluates the template.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Range;

use typst_edit::{Call, Edit, apply, find_calls};

use crate::date::parse_ymd;

/// What kind of value a slot holds; drives how a filled value renders to Typst.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    /// A quoted string, e.g. `"hello"`.
    String,
    /// An integer, e.g. `8`.
    Int,
    /// A float, e.g. `0.7`.
    Float,
    /// A boolean, `true` or `false`.
    Bool,
    /// A `YYYY-MM-DD` date, rendered as `on(Y, M, D)`.
    Date,
    /// An array of strings, e.g. `("a", "b",)`.
    List,
    /// Raw markup spliced into the body; a markup `#slot(…)` drops its `#`.
    Text,
    /// A string constrained to the slot's `choices`.
    Choice,
}

impl SlotKind {
    /// The config/DSL spelling of this kind.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::Date => "date",
            Self::List => "list",
            Self::Text => "text",
            Self::Choice => "choice",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        let kind = match text {
            "string" => Self::String,
            "int" => Self::Int,
            "float" => Self::Float,
            "bool" => Self::Bool,
            "date" => Self::Date,
            "list" => Self::List,
            "text" => Self::Text,
            "choice" => Self::Choice,
            _ => return None,
        };
        Some(kind)
    }
}

/// One discovered `slot(...)` call: its declared schema plus the source span to
/// replace when filling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotSpec {
    /// Slot name (first positional argument); the fill key.
    pub name: String,
    /// Value kind.
    pub kind: SlotKind,
    /// Whether a value must be supplied when no default is present.
    pub required: bool,
    /// Optional human prompt for interactive fill / `describe`.
    pub prompt: Option<String>,
    /// Allowed values for a `choice` slot.
    pub choices: Option<Vec<String>>,
    /// Default expression's raw source (spliced verbatim when unfilled). `None`
    /// for `default: none` or an absent `default`.
    pub default: Option<String>,
    /// Unescaped content when `default` is a string literal — used so a `text`
    /// slot's default lands in the body as markup, not as a quoted string.
    pub default_string: Option<String>,
    /// Whether the call is markup (`#slot(…)`), so a `text` fill drops the `#`.
    pub markup: bool,
    /// Byte span of the whole `slot(...)` call.
    pub range: Range<usize>,
}

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
    fn type_name(&self) -> &'static str {
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
    /// The rewrite changeset was invalid (overlap / bounds).
    Edit(String),
}

impl fmt::Display for FillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(names) => write!(f, "missing required slot(s): {}", names.join(", ")),
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
            Self::Edit(message) => write!(f, "rewrite failed: {message}"),
        }
    }
}

impl std::error::Error for FillError {}

/// Discover every `slot(...)` call in `source`, in document order.
#[must_use]
pub fn discover_slots(source: &str) -> Vec<SlotSpec> {
    find_calls(source, "slot")
        .iter()
        .filter_map(|call| spec_from_call(source, call))
        .collect()
}

fn spec_from_call(source: &str, call: &Call) -> Option<SlotSpec> {
    let name = call.positional(0)?.value.clone();
    let kind = call
        .named("kind")
        .and_then(|arg| SlotKind::parse(&arg.value))
        .unwrap_or(SlotKind::String);
    let required = call.named("required").is_none_or(|arg| arg.value == "true");
    let prompt = call.named("prompt").map(|arg| arg.value.clone());
    let choices = call
        .named("choices")
        .map(|arg| parse_string_array(&arg.value));
    let default_arg = call
        .named("default")
        .filter(|arg| arg_raw(source, arg) != "none");
    let default = default_arg.map(|arg| arg_raw(source, arg).to_owned());
    let default_string = default_arg
        .filter(|arg| arg_raw(source, arg).starts_with('"'))
        .map(|arg| arg.value.clone());
    Some(SlotSpec {
        name,
        kind,
        required,
        prompt,
        choices,
        default,
        default_string,
        markup: call.markup,
        range: call.range.clone(),
    })
}

fn arg_raw<'a>(source: &'a str, arg: &typst_edit::Arg) -> &'a str {
    source.get(arg.value_range.clone()).unwrap_or_default()
}

/// The replacement for an unfilled slot: a `text` slot's string-literal default
/// becomes unquoted body markup; every other default splices verbatim.
fn default_replacement(spec: &SlotSpec) -> Option<String> {
    let raw = spec.default.as_ref()?;
    if spec.kind == SlotKind::Text
        && let Some(text) = &spec.default_string
    {
        return Some(text.clone());
    }
    Some(raw.clone())
}

/// Pull the string literals out of an array source like `("a", "b")`. A
/// literals-only scan: collects each `"…"` segment, honouring `\"` escapes.
fn parse_string_array(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = source.chars();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }
        let mut current = String::new();
        let mut escaped = false;
        for inner in chars.by_ref() {
            if escaped {
                current.push(inner);
                escaped = false;
            } else if inner == '\\' {
                escaped = true;
            } else if inner == '"' {
                break;
            } else {
                current.push(inner);
            }
        }
        out.push(current);
    }
    out
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

/// The byte range to replace for a slot. A markup `text` slot extends over the
/// leading `#` so the spliced body markup stands alone.
fn edit_range(spec: &SlotSpec, source: &str) -> Range<usize> {
    if spec.markup
        && spec.kind == SlotKind::Text
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
        SlotKind::Choice => render_choice(spec, value),
        SlotKind::Int => Ok(coerce_int(spec, value)?.to_string()),
        SlotKind::Float => Ok(render_float(coerce_float(spec, value)?)),
        SlotKind::Bool => Ok(coerce_bool(spec, value)?.to_string()),
        SlotKind::Date => render_date(spec, &coerce_str(spec, value)?),
        SlotKind::List => Ok(render_list(&coerce_list(spec, value)?)),
    }
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
        RawValue::Str(text) => Ok(vec![text.clone()]),
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

#[cfg(test)]
#[path = "template_tests.rs"]
mod tests;
