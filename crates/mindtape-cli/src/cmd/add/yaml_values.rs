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

use mindtape::RawValue;

use crate::diag::Diag;

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
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(text)
        .map_err(|err| at(Diag::failed("can't parse YAML")).cause(&err))?;
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
        Value::Sequence(items) => RawValue::List(sequence_strings(name, items)?),
        Value::Mapping(_) | Value::Tagged(_) => {
            return Err(Diag::usage("nested mappings are unsupported").value(name));
        }
    };
    Ok(Some(raw))
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
