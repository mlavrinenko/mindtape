//! Shared YAML mapping -> [`RawValue`] conversion for `mt add`'s interactive
//! editor buffer (`add_interactive.rs`) and its `-f/--values` YAML sources
//! (`add_values.rs`). `mindtape` stays format-blind; this glue is CLI-only.
//!
//! A null leaf value converts to `None`, but its key still lands in the
//! returned map (mapped to `None`) rather than being omitted — a caller
//! merging several sources needs to tell "this source explicitly unset the
//! key" from "this source never mentioned the key" apart. Each caller decides
//! what "blank" means for its own path: `add_values::collect_values` treats a
//! `None` entry as an explicit unset when merging sources, while
//! `add_interactive::parse_buffer` simply drops it (and empty strings, which
//! this module keeps as real values — see its own filtering).

use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use mindtape::RawValue;

/// Parse `text` as a single YAML document whose root is a mapping, converting
/// each value to a [`RawValue`] (a `null` value keeps its key, mapped to
/// `None`). A `null` root is an empty mapping (nothing to fill).
///
/// `syntax_context` labels a parse failure (a malformed document, or more
/// than one — the underlying parser already rejects a multi-document
/// stream); `non_mapping` supplies the error for any other root shape. The
/// two callers word both differently, so both are supplied by the caller
/// rather than hard-coded here.
///
/// # Errors
/// Returns an error when `text` fails to parse as YAML, the root is not a
/// mapping, a key is non-string, or a value has an unsupported shape (nested
/// mapping, non-string list item).
pub fn parse_mapping(
    text: &str,
    syntax_context: impl FnOnce() -> String,
    non_mapping: impl FnOnce() -> String,
) -> Result<BTreeMap<String, Option<RawValue>>> {
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).with_context(syntax_context)?;
    let mapping = match value {
        serde_yaml_ng::Value::Mapping(mapping) => mapping,
        serde_yaml_ng::Value::Null => return Ok(BTreeMap::new()),
        _ => bail!(non_mapping()),
    };
    let mut out = BTreeMap::new();
    for (key, raw) in &mapping {
        let Some(name) = key.as_str() else {
            bail!("YAML keys must be strings");
        };
        let value = yaml_to_raw(raw).with_context(|| format!("value for {name:?}"))?;
        out.insert(name.to_owned(), value);
    }
    Ok(out)
}

fn yaml_to_raw(value: &serde_yaml_ng::Value) -> Result<Option<RawValue>> {
    use serde_yaml_ng::Value;
    let raw = match value {
        Value::Null => return Ok(None),
        Value::Bool(flag) => RawValue::Bool(*flag),
        Value::Number(num) => number_to_raw(num),
        Value::String(text) => RawValue::Str(text.clone()),
        Value::Sequence(items) => RawValue::List(sequence_strings(items)?),
        Value::Mapping(_) | Value::Tagged(_) => bail!("nested mapping values are unsupported"),
    };
    Ok(Some(raw))
}

fn number_to_raw(num: &serde_yaml_ng::Number) -> RawValue {
    num.as_i64().map_or_else(
        || RawValue::Float(num.as_f64().unwrap_or_default()),
        RawValue::Int,
    )
}

fn sequence_strings(items: &[serde_yaml_ng::Value]) -> Result<Vec<String>> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let Some(text) = item.as_str() else {
            bail!("list items must be strings");
        };
        out.push(text.to_owned());
    }
    Ok(out)
}

#[cfg(test)]
#[path = "yaml_values_tests.rs"]
mod tests;
