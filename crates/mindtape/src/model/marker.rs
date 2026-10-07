//! Marker-field projection: the typed reads [`crate::model::Task::from_marker`]
//! performs over a harvested `mindtape.task` marker.
//!
//! Every read here is total. A value that does not match the Typst shape the
//! field projects into becomes a [`ShapeError`] on the task instead of an
//! absent value, and [`crate::validate`] fails `check` on any of them
//! regardless of project config. That covers the half of the authoring surface
//! the prelude cannot police: everything that reaches the marker intact is the
//! Rust layer's to validate, and reading it as "unset" is the one answer that
//! tells the author nothing.
//!
//! # Naming a position
//!
//! A report's [`field`](ShapeError::field) is the path an author can count to
//! in their own source, extending the field name the way
//! [`crate::link::parse_links`] does for `links[1][0]`:
//!
//! | what | path |
//! | --- | --- |
//! | the field itself | `tags` |
//! | an array element | `tags[1]` |
//! | a dict entry | `estimate.days` |
//! | a nested one | `priority.inputs.impact`, `status.date` |
//!
//! # What `none` means
//!
//! A `none` DICT ENTRY reads as an unset key, never a report: the prelude's
//! helpers pass their own defaults through, so `ice(impact: 8)` arrives with
//! `confidence: none` and `ease: none` in `inputs`, and an omitted field is
//! `none` by the same route. A `none` ARRAY ELEMENT is reported like any other
//! wrong-typed one — an array has no unset slot, so a `none` in it is a value
//! the author wrote.

use std::collections::BTreeMap;

use typst_harvest::HVal;

use crate::model::Status;
use crate::model::numeric::{InputValue, NumValue};

/// A core field — or an element inside one — whose marker value did not match
/// its expected Typst shape (a string where `tags:` expects an array, a number
/// where `tags[1]` expects a string). Detected during
/// [`crate::model::Task::from_marker`], the only place that still sees the
/// raw, un-degraded value, instead of silently coercing it away.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeError {
    /// Field path: the field name, plus the offending position inside it when
    /// the mismatch is an element (`tags[1]`, `estimate.days`).
    pub field: String,
    /// Human-readable cause, naming the expected shape and what was found.
    pub message: String,
}

/// What a numeric field (`priority`, `difficulty`) accepts.
const NUM_EXPECTED: &str = "a number, a label, or a framework value like `ice(...)`";
/// What the `status:` field accepts.
const STATUS_EXPECTED: &str = "a status like `wip(2026, 6, 14)`";

/// A present, non-`none` marker field. `none` reads as absent, keeping an
/// omitted field and an explicitly-defaulted one indistinguishable — which is
/// what the prelude's parameter defaults emit for every unset slot.
pub(crate) fn field<'a>(fields: Option<&'a HVal>, key: &str) -> Option<&'a HVal> {
    fields
        .and_then(|dict| dict.get(key))
        .filter(|value| !value.is_null())
}

/// A scalar string field (`title`, `kind`, `assignee`, the date slots),
/// reported against its own name when it holds anything else.
pub(crate) fn field_str(
    fields: Option<&HVal>,
    key: &str,
    errors: &mut Vec<ShapeError>,
) -> Option<String> {
    as_string(field(fields, key), key, errors)
}

/// An optional string entry of a marker sub-dict, reported as `<path>.<key>`.
pub(crate) fn dict_str(
    dict: &BTreeMap<String, HVal>,
    path: &str,
    key: &str,
    errors: &mut Vec<ShapeError>,
) -> Option<String> {
    as_string(dict.get(key), &format!("{path}.{key}"), errors)
}

/// A required string entry of a marker sub-dict. Absent and `none` are
/// reported as `found none` rather than read as unset: the sub-dicts this
/// serves are built by a prelude helper that always writes the key, so a
/// missing one is a malformed marker, not an omitted option.
pub(crate) fn dict_str_required(
    dict: &BTreeMap<String, HVal>,
    path: &str,
    key: &str,
    errors: &mut Vec<ShapeError>,
) -> Option<String> {
    let path = format!("{path}.{key}");
    let Some(value) = dict.get(key).filter(|value| !value.is_null()) else {
        errors.push(shape_message(&path, "expected a string, found none"));
        return None;
    };
    as_string(Some(value), &path, errors)
}

/// `value` as a string, reporting against `path` when it is present holding
/// another type. Absent or `none` yields `None` with no report — whether that
/// is legal is the caller's question, not this one's.
fn as_string(value: Option<&HVal>, path: &str, errors: &mut Vec<ShapeError>) -> Option<String> {
    let value = value.filter(|value| !value.is_null())?;
    let Some(text) = value.as_str() else {
        errors.push(shape_error(path, "a string", value));
        return None;
    };
    Some(text.to_owned())
}

/// Project the `status:` field: the prelude's `(name, date, note)` dict. A
/// value that is not a dict at all, or a dict carrying no `name`, is the whole
/// status gone — reported against `status`; a wrong-typed part of an otherwise
/// readable status is reported against that part.
pub(crate) fn parse_status(value: Option<&HVal>, errors: &mut Vec<ShapeError>) -> Option<Status> {
    let value = value?;
    let Some(dict) = value.as_dict() else {
        errors.push(shape_error("status", STATUS_EXPECTED, value));
        return None;
    };
    if dict.get("name").is_none_or(HVal::is_null) {
        errors.push(shape_message(
            "status",
            &format!("expected {STATUS_EXPECTED}, found a status with no `name`"),
        ));
        return None;
    }
    let name = dict_str(dict, "status", "name", errors)?;
    Some(Status {
        name,
        date: dict_str(dict, "status", "date", errors),
        note: dict_str(dict, "status", "note", errors),
    })
}

/// Project a marker field into a numeric-field value: a number becomes
/// [`NumValue::Num`], a framework dict (`(framework: …, inputs: …)`) a
/// [`NumValue::Framework`], a string a [`NumValue::Label`]. Anything else —
/// including a dict that is not a framework value — is reported against
/// `field_name` rather than read as an unset priority.
pub(crate) fn parse_num_value(
    value: Option<&HVal>,
    field_name: &str,
    errors: &mut Vec<ShapeError>,
) -> Option<NumValue> {
    let value = value?;
    if let Some(num) = value.as_f64() {
        return Some(NumValue::Num(num));
    }
    if let Some(dict) = value.as_dict() {
        let Some(name) = dict.get("framework").and_then(HVal::as_str) else {
            errors.push(shape_error(field_name, NUM_EXPECTED, value));
            return None;
        };
        return Some(NumValue::Framework {
            name: name.to_owned(),
            inputs: parse_inputs(dict.get("inputs"), field_name, errors),
        });
    }
    let Some(text) = value.as_str() else {
        errors.push(shape_error(field_name, NUM_EXPECTED, value));
        return None;
    };
    Some(NumValue::Label(text.to_owned()))
}

/// Project the `inputs` dict of a framework value into typed input values.
/// An input that is not a scalar is reported as `<field>.inputs.<key>`;
/// whether a scalar is the RIGHT one for the framework (and whether a required
/// one is missing) is [`crate::framework::validate`]'s question.
fn parse_inputs(
    value: Option<&HVal>,
    field_name: &str,
    errors: &mut Vec<ShapeError>,
) -> BTreeMap<String, InputValue> {
    let path = format!("{field_name}.inputs");
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return BTreeMap::new();
    };
    let Some(dict) = value.as_dict() else {
        errors.push(shape_error(&path, "a dict", value));
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (key, val) in dict.iter().filter(|(_, val)| !val.is_null()) {
        if let Some(input) = input_value(val) {
            out.insert(key.clone(), input);
        } else {
            errors.push(shape_error(
                &format!("{path}.{key}"),
                "a number, a boolean, or a string",
                val,
            ));
        }
    }
    out
}

fn input_value(value: &HVal) -> Option<InputValue> {
    match value {
        HVal::Bool(flag) => Some(InputValue::Bool(*flag)),
        HVal::Int(_) | HVal::Float(_) => value.as_f64().map(InputValue::Num),
        HVal::Str(text) => Some(InputValue::Text(text.clone())),
        _ => None,
    }
}

/// Project an opaque `unit -> quantity` map (`estimate`). A non-numeric
/// quantity is reported as `<field>.<unit>` — the unit label itself is free
/// text and never checked here.
pub(crate) fn parse_num_map(
    value: Option<&HVal>,
    field_name: &str,
    errors: &mut Vec<ShapeError>,
) -> BTreeMap<String, f64> {
    let Some(value) = value else {
        return BTreeMap::new();
    };
    let Some(dict) = value.as_dict() else {
        errors.push(shape_error(field_name, "a dict", value));
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (key, val) in dict.iter().filter(|(_, val)| !val.is_null()) {
        if let Some(num) = val.as_f64() {
            out.insert(key.clone(), num);
        } else {
            errors.push(shape_error(&format!("{field_name}.{key}"), "a number", val));
        }
    }
    out
}

/// Project an array of strings (`tags`). A non-string entry is reported as
/// `<field>[<index>]` rather than dropped, so a mistyped tag names its own
/// position instead of reading as one tag fewer.
pub(crate) fn parse_str_array(
    value: Option<&HVal>,
    field_name: &str,
    errors: &mut Vec<ShapeError>,
) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        errors.push(shape_error(field_name, "an array", value));
        return Vec::new();
    };
    let mut out = Vec::new();
    for (index, item) in items.iter().enumerate() {
        if let Some(text) = item.as_str() {
            out.push(text.to_owned());
        } else {
            errors.push(shape_error(
                &format!("{field_name}[{index}]"),
                "a string",
                item,
            ));
        }
    }
    out
}

/// Build a [`ShapeError`] naming `field_name`, the expected shape, and the
/// Typst kind of what was actually found.
pub(crate) fn shape_error(field_name: &str, expected: &str, value: &HVal) -> ShapeError {
    ShapeError {
        field: field_name.to_owned(),
        message: format!("expected {expected}, found {}", hval_kind(value)),
    }
}

/// Build a [`ShapeError`] with a hand-written cause, for the reports whose
/// "found" half is not a Typst kind (a weekday name that is not one, a dict
/// missing a key the shape requires).
pub(crate) fn shape_message(field_name: &str, message: &str) -> ShapeError {
    ShapeError {
        field: field_name.to_owned(),
        message: message.to_owned(),
    }
}

fn hval_kind(value: &HVal) -> &'static str {
    match value {
        HVal::Null => "none",
        HVal::Bool(_) => "a boolean",
        HVal::Int(_) | HVal::Float(_) => "a number",
        HVal::Str(_) => "a string",
        HVal::Date(_) => "a date",
        HVal::Array(_) => "an array",
        HVal::Dict(_) => "a dict",
    }
}

#[cfg(test)]
#[path = "marker_tests.rs"]
mod tests;
