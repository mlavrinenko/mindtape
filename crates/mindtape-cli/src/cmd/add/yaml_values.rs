//! Shared YAML mapping -> [`RawValue`] conversion for `mt add`'s interactive
//! editor buffer (`cmd/add/interactive.rs`) and its `-f/--values` YAML sources
//! (`cmd/add/values.rs`). `mindtape` stays format-blind; this glue is CLI-only.
//!
//! A null leaf value converts to `None`, but its key still lands in the
//! returned map (mapped to `None`) rather than being omitted — a caller
//! merging several sources needs to tell "this source explicitly unset the
//! key" from "this source never mentioned the key" apart. Each caller decides
//! what "blank" means for its own path: `cmd::add::values::collect_values` treats a
//! `None` entry as an explicit unset when merging sources, while
//! `cmd::add::interactive::parse_buffer` simply drops it (and empty strings, which
//! this module keeps as real values — see its own filtering).

use std::collections::BTreeMap;

use mindtape::{LinkValue, RawValue};

use crate::diag::{Diag, Hint};

/// Parse `text` as a single YAML document whose root is a mapping, converting
/// each value to a [`RawValue`] (a `null` value keeps its key, mapped to
/// `None`). A `null` root is an empty mapping (nothing to fill).
///
/// `source` names where the text came from, a `-f` source, and is quoted in
/// the error. The editor buffer passes `None`, because the user just typed it.
///
/// # Errors
/// Returns a [`Diag`] when `text` fails to parse as YAML (a malformed
/// document, or more than one), the root is not a mapping, a key is
/// non-string, or a value has an unsupported shape (nested mapping,
/// non-string list item).
pub fn parse_mapping(
    text: &str,
    source: Option<&str>,
) -> Result<BTreeMap<String, Option<RawValue>>, Diag> {
    let at = |diag: Diag| match source {
        Some(source) => diag.value(source),
        None => diag,
    };
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).map_err(|err| {
        at(Diag::failed("can't parse YAML"))
            .cause(&err)
            .hint(Hint::new(
                "quote a value that starts with a YAML indicator such as a backtick, \
                     or write it as a `|-` block",
            ))
    })?;
    let mapping = match value {
        serde_yaml_ng::Value::Mapping(mapping) => mapping,
        serde_yaml_ng::Value::Null => return Ok(BTreeMap::new()),
        _ => return Err(at(Diag::usage("values must be a YAML mapping"))),
    };
    let mut out = BTreeMap::new();
    for (key, raw) in &mapping {
        let Some(name) = key.as_str() else {
            return Err(at(Diag::usage("YAML keys must be strings")));
        };
        out.insert(name.to_owned(), yaml_to_raw(name, raw)?);
    }
    Ok(out)
}

fn yaml_to_raw(name: &str, value: &serde_yaml_ng::Value) -> Result<Option<RawValue>, Diag> {
    use serde_yaml_ng::Value;
    let raw = match value {
        Value::Null => return Ok(None),
        Value::Bool(flag) => RawValue::Bool(*flag),
        Value::Number(num) => number_to_raw(num),
        Value::String(text) => RawValue::Str(text.clone()),
        Value::Sequence(items) => sequence(name, items)?,
        Value::Mapping(_) | Value::Tagged(_) => {
            return Err(Diag::usage("nested mappings are unsupported").value(name));
        }
    };
    Ok(Some(raw))
}

/// A YAML sequence is a string list, or, when every item is a mapping, a list
/// of typed edges. A mixed sequence is neither, and the refusal names the slot.
fn sequence(name: &str, items: &[serde_yaml_ng::Value]) -> Result<RawValue, Diag> {
    let mappings = items
        .iter()
        .filter(|item| matches!(item, serde_yaml_ng::Value::Mapping(_)))
        .count();
    if mappings == 0 {
        return Ok(RawValue::List(sequence_strings(name, items)?));
    }
    if mappings != items.len() {
        return Err(Diag::usage("list items must be strings or link mappings").value(name));
    }
    Ok(RawValue::Links(sequence_links(name, items)?))
}

fn sequence_links(name: &str, items: &[serde_yaml_ng::Value]) -> Result<Vec<LinkValue>, Diag> {
    items.iter().map(|item| link_mapping(name, item)).collect()
}

fn link_mapping(slot: &str, item: &serde_yaml_ng::Value) -> Result<LinkValue, Diag> {
    let serde_yaml_ng::Value::Mapping(mapping) = item else {
        return Err(Diag::usage("list items must be strings or link mappings").value(slot));
    };
    let mut fields = LinkFields::default();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err(Diag::usage("link keys must be strings").value(slot));
        };
        fields.set(slot, key, link_text(slot, key, value)?)?;
    }
    fields.finish(slot)
}

fn link_text(slot: &str, key: &str, value: &serde_yaml_ng::Value) -> Result<Option<String>, Diag> {
    match value {
        serde_yaml_ng::Value::Null => Ok(None),
        serde_yaml_ng::Value::String(text) => Ok(Some(text.clone())),
        _ => Err(Diag::usage("a link field must be a string")
            .value(key)
            .context("in slot", slot)),
    }
}

/// The fields one `{kind, task, note}` mapping fills, shared by the YAML and
/// JSON readers so the two value sources cannot disagree on what a link is.
#[derive(Default)]
pub(super) struct LinkFields {
    task: Option<String>,
    kind: Option<String>,
    note: Option<String>,
}

impl LinkFields {
    pub(super) fn set(&mut self, slot: &str, key: &str, value: Option<String>) -> Result<(), Diag> {
        let field = match key {
            "task" => &mut self.task,
            "kind" => &mut self.kind,
            "note" => &mut self.note,
            other => {
                return Err(Diag::usage("unknown link key")
                    .value(other)
                    .context("in slot", slot));
            }
        };
        match value {
            Some(text) => {
                *field = Some(text);
                Ok(())
            }
            None if key == "note" => Ok(()),
            None => Err(Diag::usage("a link field must be a string")
                .value(key)
                .context("in slot", slot)),
        }
    }

    pub(super) fn finish(self, slot: &str) -> Result<LinkValue, Diag> {
        let task = self
            .task
            .ok_or_else(|| Diag::usage("a link needs a task").value(slot))?;
        let kind = self
            .kind
            .ok_or_else(|| Diag::usage("a link needs a kind").value(slot))?;
        if task.is_empty() {
            return Err(Diag::usage("a link needs a task").value(slot));
        }
        if kind.is_empty() {
            return Err(Diag::usage("a link needs a kind").value(slot));
        }
        Ok(LinkValue {
            task,
            kind,
            note: self.note,
        })
    }
}

fn number_to_raw(num: &serde_yaml_ng::Number) -> RawValue {
    num.as_i64().map_or_else(
        || RawValue::Float(num.as_f64().unwrap_or_default()),
        RawValue::Int,
    )
}

fn sequence_strings(name: &str, items: &[serde_yaml_ng::Value]) -> Result<Vec<String>, Diag> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let Some(text) = item.as_str() else {
            return Err(Diag::usage("list items must be strings").value(name));
        };
        out.push(text.to_owned());
    }
    Ok(out)
}

#[cfg(test)]
#[path = "yaml_values_tests.rs"]
mod tests;
