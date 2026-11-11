//! Ordered value operations from `-f`/`--values` and the `--set` tokens.
//!
//! Collection keeps the caller's order instead of collapsing it into one map:
//! `tags+=a` followed by `tags=b` reaches a writer as an append and then a set,
//! so the writer can drop the earlier append rather than guess what the map
//! meant. A single [`Collected`] feeds both slot fill and the writable-field
//! writer in [`super::field_values`].
//!
//! Split out of `values.rs` to keep that file under its line budget; the
//! source readers it depends on stay there.

use std::collections::BTreeMap;

use mindtape::model::writable;
use mindtape::{FieldOp, RawValue, SlotKind, SlotSpec};

use super::tokens::{TokenOp, parse_token};
use super::values::{parse_source, read_source};
use crate::diag::{Command, Diag, Hint, Note};

type Result<T> = std::result::Result<T, Diag>;

/// Every ordered value operation for one `mt add`, before the template's slots
/// are known.
#[derive(Debug)]
pub struct Collected {
    /// Sources and tokens in caller order: `-f` keys first, then `--set`, then
    /// the bare arguments after `--`.
    pub ops: Vec<(String, FieldOp)>,
}

impl Collected {
    /// The fill map for `specs`: each slot's operations applied in order, with
    /// a list slot's `+=` extending an explicit list or its `default`.
    ///
    /// # Errors
    /// Returns an error for a `+=` on a slot that holds one value.
    pub fn slot_values(&self, specs: &[SlotSpec]) -> Result<BTreeMap<String, RawValue>> {
        let mut out = BTreeMap::new();
        for spec in specs {
            if let Some(value) = slot_value(spec, &self.ops)? {
                out.insert(spec.name.clone(), value);
            }
        }
        Ok(out)
    }
}

/// Merge `--values` sources (paths; `-` = stdin; JSON or YAML, see
/// [`super::values::parse_source`]), then the `key OP value` tokens — `--set`
/// entries first, then the bare arguments after `--` — into ordered operations.
///
/// Sources are applied left to right: a later source's value for a key is a
/// later `Set`, and an explicit `null`/`~` removes every earlier operation for
/// that key — the same "last source wins" rule extended to an explicit unset.
///
/// # Errors
/// Returns an error when a source is unreadable, not a JSON object or YAML
/// mapping, holds an unsupported value shape, or a token is not a valid
/// assignment (see [`super::tokens::parse_token`]).
pub fn collect_values(values: &[String], sets: &[String], tokens: &[String]) -> Result<Collected> {
    let mut ops: Vec<(String, FieldOp)> = Vec::new();
    for source in values {
        let text = read_source(source)?;
        let patch = parse_source(source, &text)?;
        for (key, value) in patch {
            match value {
                Some(raw) => ops.push((key, FieldOp::Set(raw))),
                None => ops.retain(|(name, _)| name != &key),
            }
        }
    }
    for token in sets.iter().chain(tokens) {
        ops.push(field_op(token)?);
    }
    Ok(Collected { ops })
}

/// One token as the library's ordered field operation.
fn field_op(token: &str) -> Result<(String, FieldOp)> {
    let (name, op) = parse_token(token)?;
    let op = match op {
        TokenOp::Set(value) | TokenOp::File(value) => FieldOp::Set(RawValue::Str(value)),
        TokenOp::Append(item) => FieldOp::Append(item),
    };
    Ok((name, op))
}

/// A slot's final value from its ordered operations, or `None` when it has
/// none and the template's own default should stand.
fn slot_value(spec: &SlotSpec, ops: &[(String, FieldOp)]) -> Result<Option<RawValue>> {
    let mut value: Option<RawValue> = None;
    let mut pending: Vec<String> = Vec::new();
    for (_, op) in ops.iter().filter(|(name, _)| name == &spec.name) {
        match op {
            FieldOp::Set(raw) => {
                value = Some(base_value(spec, raw));
                pending.clear();
            }
            FieldOp::Append(item) => {
                if !is_list_slot(spec) {
                    return Err(append_to_scalar(spec));
                }
                match value.as_mut() {
                    Some(held) => append_item(held, item),
                    None => pending.push(item.clone()),
                }
            }
        }
    }
    if value.is_some() {
        return Ok(value);
    }
    if pending.is_empty() {
        return Ok(None);
    }
    let mut items = default_items(spec);
    for item in pending {
        if !items.contains(&item) {
            items.push(item);
        }
    }
    Ok(Some(RawValue::List(items)))
}

/// Whether a slot holds a list. A `+=` on any other kind is a caller mistake,
/// not a list of one item.
fn is_list_slot(spec: &SlotSpec) -> bool {
    matches!(spec.kind, SlotKind::List | SlotKind::MultiChoice)
}

fn append_to_scalar(spec: &SlotSpec) -> Diag {
    Diag::usage("+= appends to a list slot")
        .value(spec.name.clone())
        .hint(Hint::new("write one value with slot=value"))
}

/// A value from an explicit `=`. A string for a list slot is one item, the
/// same spelling `mt set tags=bug` writes.
fn base_value(spec: &SlotSpec, value: &RawValue) -> RawValue {
    if !is_list_slot(spec) {
        return value.clone();
    }
    match value {
        RawValue::Str(text) => RawValue::List(vec![text.clone()]),
        RawValue::List(_) | RawValue::Links(_) => value.clone(),
        scalar => RawValue::List(vec![scalar_text(scalar)]),
    }
}

pub(super) fn scalar_text(value: &RawValue) -> String {
    match value {
        RawValue::Str(text) => text.clone(),
        RawValue::Int(number) => number.to_string(),
        RawValue::Float(number) => mindtape::format_num(*number),
        RawValue::Bool(flag) => flag.to_string(),
        RawValue::List(_) | RawValue::Links(_) => String::new(),
    }
}

fn append_item(value: &mut RawValue, item: &str) {
    match value {
        RawValue::List(items) => {
            if !items.iter().any(|held| held == item) {
                items.push(item.to_owned());
            }
        }
        slot => *slot = RawValue::List(vec![item.to_owned()]),
    }
}

fn default_items(spec: &SlotSpec) -> Vec<String> {
    spec.default
        .as_deref()
        .map(mindtape::parse_string_array)
        .unwrap_or_default()
}

/// The hint for a field `mt add` writes by another spelling. `status` and `id`
/// have an add-time answer of their own: the hints `mt set` gives (flip, mv)
/// name a task that does not exist until the add files it.
fn add_elsewhere(name: &str) -> Option<&'static str> {
    match name {
        "status" => Some("use --status to file the new task at a status"),
        "id" => Some("the id is stamped when the task is filed; tasks.id.format decides it"),
        other => writable::elsewhere(other),
    }
}

/// Reject any operation key that names no slot in `specs` and no writable
/// field — a `--set`/`-f` typo would otherwise silently vanish (the template
/// default fills in instead) with nothing in the output to notice.
///
/// # Errors
/// Returns an error naming the unknown key(s), refusing a `+=` on a scalar
/// writable field by name, or naming the command that owns a field written
/// elsewhere.
pub fn reject_unknown_keys(ops: &[(String, FieldOp)], specs: &[SlotSpec]) -> Result<()> {
    for (name, op) in ops {
        if matches!(op, FieldOp::Append(_))
            && writable::get(name).is_some()
            && !writable::is_list(name)
        {
            return Err(Diag::usage("+= appends to a list field")
                .value(name.clone())
                .hint(Hint::new("replace the value with <field>=<value>")));
        }
        if name == "dir" {
            return Err(Diag::usage("mt add does not write this field")
                .value(name.clone())
                .hint(Hint::new(add_elsewhere("dir").unwrap_or(
                    "a directory is not a task field; use --dir when adding",
                ))));
        }
    }
    let mut known: Vec<String> = specs.iter().map(|spec| spec.name.clone()).collect();
    known.extend(writable::names().map(str::to_owned));
    let mut seen: Vec<&str> = Vec::new();
    for (name, _) in ops {
        if !seen.contains(&name.as_str()) {
            seen.push(name);
        }
    }
    let unknown: Vec<&str> = seen
        .into_iter()
        .filter(|key| !known.iter().any(|name| name == key))
        .collect();
    if unknown.is_empty() {
        return Ok(());
    }
    let mut keys = unknown.into_iter();
    let first = keys.next().unwrap_or_default();
    if let Some(owner) = add_elsewhere(first) {
        return Err(Diag::usage("mt add does not write this field")
            .value(first)
            .hint(Hint::new(owner)));
    }
    let diag = Diag::usage("unknown slot or field").value(first);
    let known: Vec<&str> = known.iter().map(String::as_str).collect();
    let diag = diag.did_you_mean(first, &known);
    let diag = keys.fold(diag, |diag, key| {
        diag.note(Note::new("also unknown").value(key))
    });
    Err(diag.hint(Hint::run(
        Command::mt().word("add").word("--describe"),
        "to see the slots",
    )))
}
