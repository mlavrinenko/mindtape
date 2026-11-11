//! The writable core fields: one table, one renderer and one mutation path
//! shared by `mt set`, `mt add --set` and the add hooks.
//!
//! A field is writable when its value is one unambiguous Typst literal: a
//! quoted string, a bare number, an `on(y, m, d)` call, or an array of quoted
//! strings. Everything with a richer argument shape — a dated status, a typed
//! edge, a renamed stem, an estimate's unit dict — keeps its own command, and
//! [`ELSEWHERE`] names that command instead of reporting a typo.
//!
//! The rendering lives here, not in the CLI, so the value `mt set` writes and
//! the value `mt add` inserts cannot drift: both call [`render_op`], and both
//! use [`apply_op`] when they need the value reflected in a [`Task`] before
//! rendering.

use crate::config::Config;
use crate::date::parse_ymd;
use crate::emit::format::{typ_array, typ_str};
use crate::model::Task;
use crate::model::numeric::{NumValue, format_num};
use crate::template::RawValue;

/// What a writable field's value looks like once rendered as Typst.
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    /// A quoted, escaped string literal.
    Text,
    /// A bare number. Whether it is in bounds, or on the configured scale at
    /// all, is the validator's call, not this module's.
    Num,
    /// A `YYYY-MM-DD` argument, rendered as the prelude's `on(y, m, d)`.
    Date,
    /// An array literal. The accessor reads the field's current items, which
    /// `+=` appends to; naming it per field is what stops a second list field
    /// from silently appending to `tags`.
    List(fn(&Task) -> &[String]),
}

/// One field a write command may set.
#[derive(Debug)]
pub struct WritableField {
    /// The field's name, as the task DSL spells it.
    pub name: &'static str,
    /// The literal its value renders to.
    pub shape: Shape,
}

/// Every writable field, in the task emitter's own field order: the scalars
/// and lists whose Typst literal is unambiguous. `priority` is the raw numeric
/// form only — a framework priority (`ice(impact: 8, …)`) is a call with its
/// own inputs, and packing those into one `=<value>` token would invent a
/// spelling nothing else in the CLI uses.
pub const WRITABLE: &[WritableField] = &[
    WritableField {
        name: "title",
        shape: Shape::Text,
    },
    WritableField {
        name: "priority",
        shape: Shape::Num,
    },
    WritableField {
        name: "difficulty",
        shape: Shape::Num,
    },
    WritableField {
        name: "kind",
        shape: Shape::Text,
    },
    WritableField {
        name: "assignee",
        shape: Shape::Text,
    },
    WritableField {
        name: "waits-on",
        shape: Shape::List(|task| &task.waits_on),
    },
    WritableField {
        name: "tags",
        shape: Shape::List(|task| &task.tags),
    },
    WritableField {
        name: "due",
        shape: Shape::Date,
    },
    WritableField {
        name: "start",
        shape: Shape::Date,
    },
];

/// Core fields no generic writer sets, and the hint that says what to do
/// instead. Refused by name rather than as unknown: `status` is not a typo for
/// `start`, and a message that suggested so would teach the wrong CLI.
pub const ELSEWHERE: &[(&str, &str)] = &[
    (
        "status",
        "use mt flip <status> <task>, which dates it and follows the legend's transitions",
    ),
    (
        "links",
        "use mt link <task> <kind> <target>, which writes typed edges and checks each target",
    ),
    (
        "reviewed",
        "use mt review <task>, which stamps today's date",
    ),
    (
        "recur",
        "use mt recur <task>, the only command that advances a recurrence",
    ),
    (
        "estimate",
        "an estimate is a unit dict, not a scalar, so edit it by hand and run mt check",
    ),
    (
        "id",
        "use mt mv <task> <new-stem>, which renames it and retargets every inbound edge",
    ),
    (
        "dir",
        "a directory is not a field; use mt mv <task> <new-stem> for a filed task, or --dir when adding",
    ),
];

/// One ordered write to a field: a value to set, or one item to append.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldOp {
    /// `<field>=<value>` (or `<field>@<path>`).
    Set(RawValue),
    /// `<field>+=<value>`.
    Append(String),
}

/// Why a value cannot be written to a field.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WritableError {
    /// `+=` on a field that holds one value.
    #[error("+= appends to a list field")]
    AppendToScalar {
        /// The field the token named.
        field: &'static str,
    },
    /// A value for a numeric field that is not a finite number.
    #[error("a number is expected")]
    NotANumber {
        /// The field the value was for.
        field: &'static str,
        /// The value as the caller gave it.
        value: String,
    },
    /// A value for a date field that is not `YYYY-MM-DD`.
    #[error("a YYYY-MM-DD date is expected")]
    NotADate {
        /// The field the value was for.
        field: &'static str,
        /// The value as the caller gave it.
        value: String,
    },
    /// A whole array given to a field that holds one value.
    #[error("one value is expected")]
    ManyValues {
        /// The field the array was for.
        field: &'static str,
    },
    /// A value shape no writable field holds, such as a list of typed edges.
    #[error("this value shape does not fit the field")]
    WrongShape {
        /// The field the value was for.
        field: &'static str,
    },
}

/// The writable field `name` names.
#[must_use]
pub fn get(name: &str) -> Option<&'static WritableField> {
    WRITABLE.iter().find(|field| field.name == name)
}

/// Every writable field name, in the emitter's field order.
pub fn names() -> impl Iterator<Item = &'static str> {
    WRITABLE.iter().map(|field| field.name)
}

/// Whether `name` is a writable list field.
#[must_use]
pub fn is_list(name: &str) -> bool {
    matches!(get(name).map(|field| field.shape), Some(Shape::List(_)))
}

/// The hint for a core field another command owns, or `None` for any other
/// name. A caller names the field with `Diag::value`.
#[must_use]
pub fn elsewhere(name: &str) -> Option<&'static str> {
    ELSEWHERE
        .iter()
        .find_map(|(field, hint)| (*field == name).then_some(*hint))
}

/// The Typst literal to write, or `None` when the task already says it — a
/// re-run then writes nothing at all.
///
/// # Errors
/// A `+=` on a field that holds no list, or a value the field's shape cannot
/// be made from — refused here, before a file is even read, so the error names
/// the field rather than arriving as a Typst type error later.
pub fn render_op(
    field: &WritableField,
    op: &FieldOp,
    task: &Task,
) -> Result<Option<String>, WritableError> {
    match (field.shape, op) {
        (Shape::List(current), FieldOp::Append(item)) => Ok(append(current(task), item)),
        (Shape::List(_), FieldOp::Set(value)) => {
            let items = list_items(field, value)?;
            let items: Vec<String> = items.iter().map(|item| typ_str(item)).collect();
            Ok(Some(typ_array(&items)))
        }
        (_, FieldOp::Append(_)) => Err(WritableError::AppendToScalar { field: field.name }),
        (Shape::Text, FieldOp::Set(value)) => Ok(Some(typ_str(&scalar_text(field, value)?))),
        (Shape::Num, FieldOp::Set(value)) => {
            let number = number(field, &scalar_text(field, value)?)?;
            Ok(Some(format_num(number)))
        }
        (Shape::Date, FieldOp::Set(value)) => {
            Ok(Some(date_literal(field, &scalar_text(field, value)?)?))
        }
    }
}

/// Mutate `task`'s field from `op`, coercing the value exactly as [`render_op`]
/// does. Used by `mt add` after it evaluates the draft task, so a field the
/// template omits can be inserted with its authored value.
///
/// `config` is accepted for the callers that resolve a field against the
/// project's own scales later; this module's shapes need none of it yet.
///
/// # Errors
/// The same refusals as [`render_op`]: an append to a scalar field, a non-number
/// for a numeric one, a non-date for a date one, or a value shape the field
/// cannot hold.
pub fn apply_op(
    task: &mut Task,
    field: &WritableField,
    op: &FieldOp,
    _config: &Config,
) -> Result<(), WritableError> {
    match op {
        FieldOp::Set(value) => set(task, field, value),
        FieldOp::Append(item) => append_to_task(task, field, item),
    }
}

/// The field's current value as a [`RawValue`], for feeding a template slot.
/// `None` when a scalar field is absent; a list field always reads as its
/// (possibly empty) array.
#[must_use]
pub fn current_raw(field: &WritableField, task: &Task) -> Option<RawValue> {
    match field.shape {
        Shape::Text => text_field(field, task).map(|text| RawValue::Str(text.to_owned())),
        Shape::Num => num_field(field, task).map(num_raw),
        Shape::Date => date_field(field, task).map(|text| RawValue::Str(text.to_owned())),
        Shape::List(items) => Some(RawValue::List(items(task).to_vec())),
    }
}

/// The currently held value as a Typst literal: a quoted string, a bare
/// number, an `on(y, m, d)` date or an array. `None` when a scalar field is
/// absent, so a caller can leave an omitted field omitted.
#[must_use]
pub fn current_literal(field: &WritableField, task: &Task) -> Option<String> {
    match field.shape {
        Shape::Text => text_field(field, task).map(typ_str),
        Shape::Num => num_field(field, task).map(num_literal),
        Shape::Date => date_field(field, task)
            .map(|text| date_literal(field, text).unwrap_or_else(|_| typ_str(text))),
        Shape::List(items) => Some(typ_array(
            &items(task)
                .iter()
                .map(|item| typ_str(item))
                .collect::<Vec<_>>(),
        )),
    }
}

/// One field's scalar value as text, refusing an array.
fn scalar_text(field: &WritableField, value: &RawValue) -> Result<String, WritableError> {
    match value {
        RawValue::Str(text) => Ok(text.clone()),
        RawValue::Int(number) => Ok(number.to_string()),
        RawValue::Float(number) => Ok(format_num(*number)),
        RawValue::Bool(flag) => Ok(flag.to_string()),
        RawValue::List(_) => Err(WritableError::ManyValues { field: field.name }),
        RawValue::Links(_) => Err(WritableError::WrongShape { field: field.name }),
    }
}

/// One field's value as list items: a whole array, or one scalar as one item.
fn list_items(field: &WritableField, value: &RawValue) -> Result<Vec<String>, WritableError> {
    match value {
        RawValue::List(items) => Ok(items.clone()),
        RawValue::Links(_) => Err(WritableError::WrongShape { field: field.name }),
        scalar => Ok(vec![scalar_text(field, scalar)?]),
    }
}

/// `+=`: the field's current items plus `item`, or `None` when it already
/// holds it. Deduping here rather than writing a repeated entry keeps the
/// command re-runnable — a script that appends a tag twice is not a script
/// that meant to tag it twice.
fn append(current: &[String], item: &str) -> Option<String> {
    if current.iter().any(|held| held == item) {
        return None;
    }
    let mut items: Vec<String> = current.iter().map(|held| typ_str(held)).collect();
    items.push(typ_str(item));
    Some(typ_array(&items))
}

/// A bare numeric literal, rendered the way the task emitter renders one.
fn number(field: &WritableField, text: &str) -> Result<f64, WritableError> {
    match NumValue::parse(text) {
        NumValue::Num(number) if number.is_finite() => Ok(number),
        _ => Err(WritableError::NotANumber {
            field: field.name,
            value: text.to_owned(),
        }),
    }
}

/// A date argument as the prelude's `on(y, m, d)` call.
fn date_literal(field: &WritableField, text: &str) -> Result<String, WritableError> {
    let (year, month, day) = parse_ymd(text).ok_or_else(|| WritableError::NotADate {
        field: field.name,
        value: text.to_owned(),
    })?;
    Ok(format!("on({year}, {month}, {day})"))
}

/// Set one field from a [`RawValue`].
fn set(task: &mut Task, field: &WritableField, value: &RawValue) -> Result<(), WritableError> {
    if let Shape::List(_) = field.shape {
        let items = list_items(field, value)?;
        let target =
            list_mut(task, field.name).ok_or(WritableError::WrongShape { field: field.name })?;
        *target = items;
        return Ok(());
    }
    let text = scalar_text(field, value)?;
    match field.name {
        "title" => task.title = text,
        "priority" => task.priority = Some(num_value(field, &text)?),
        "difficulty" => task.difficulty = Some(num_value(field, &text)?),
        "kind" => task.kind = Some(text),
        "assignee" => task.assignee = Some(text),
        "due" => {
            date_literal(field, &text)?;
            task.due = Some(text);
        }
        "start" => {
            date_literal(field, &text)?;
            task.start = Some(text);
        }
        _ => return Err(WritableError::WrongShape { field: field.name }),
    }
    Ok(())
}

/// Append one item to a list field, doing nothing when the field holds it.
fn append_to_task(task: &mut Task, field: &WritableField, item: &str) -> Result<(), WritableError> {
    if !matches!(field.shape, Shape::List(_)) {
        return Err(WritableError::AppendToScalar { field: field.name });
    }
    let items =
        list_mut(task, field.name).ok_or(WritableError::WrongShape { field: field.name })?;
    if !items.iter().any(|held| held == item) {
        items.push(item.to_owned());
    }
    Ok(())
}

/// The mutable items of a list field, by name.
fn list_mut<'a>(task: &'a mut Task, name: &str) -> Option<&'a mut Vec<String>> {
    match name {
        "waits-on" => Some(&mut task.waits_on),
        "tags" => Some(&mut task.tags),
        _ => None,
    }
}

fn num_value(field: &WritableField, text: &str) -> Result<NumValue, WritableError> {
    number(field, text).map(NumValue::Num)
}

/// A text field's current value, `None` when absent.
fn text_field<'a>(field: &WritableField, task: &'a Task) -> Option<&'a str> {
    match field.name {
        "title" => Some(&task.title),
        "kind" => task.kind.as_deref(),
        "assignee" => task.assignee.as_deref(),
        _ => None,
    }
}

/// A numeric field's current value.
fn num_field<'a>(field: &WritableField, task: &'a Task) -> Option<&'a NumValue> {
    match field.name {
        "priority" => task.priority.as_ref(),
        "difficulty" => task.difficulty.as_ref(),
        _ => None,
    }
}

/// A date field's current value.
fn date_field<'a>(field: &WritableField, task: &'a Task) -> Option<&'a str> {
    match field.name {
        "due" => task.due.as_deref(),
        "start" => task.start.as_deref(),
        _ => None,
    }
}

/// A numeric field's current value as a [`RawValue`].
///
/// A number reads as its text form, not [`RawValue::Float`]: a template slot
/// may declare `kind: "int"`, whose coercion accepts a string or an integer
/// but not a float, and `3` and `3.0` are the same number to the writer.
fn num_raw(value: &NumValue) -> RawValue {
    match value {
        NumValue::Num(number) => RawValue::Str(format_num(*number)),
        NumValue::Label(label) => RawValue::Str(label.clone()),
        NumValue::Framework { name, .. } => RawValue::Str(name.clone()),
    }
}

/// A numeric field's current value as a Typst literal.
fn num_literal(value: &NumValue) -> String {
    match value {
        NumValue::Num(number) => format_num(*number),
        NumValue::Label(label) => typ_str(label),
        NumValue::Framework { name, .. } => typ_str(name),
    }
}

#[cfg(test)]
#[path = "writable_tests.rs"]
mod tests;
