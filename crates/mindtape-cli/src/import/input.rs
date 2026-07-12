//! Read and parse canonical task-JSON input: a file arg or stdin (`-`), as a
//! bare array (`mt ls --json`'s default shape) or the `{ "tasks": [...] }` /
//! `{ "tasks": ..., "applied": ... }` envelope (`mt ls --json`'s rule-applied
//! shape). `applied` is ignored either way.

use std::io::Read as _;

use anyhow::{Context, Result, bail};
use mindtape::CanonicalTask;
use serde_json::Value;

/// Read `source` to a string: `-` reads stdin, anything else is a file path.
///
/// # Errors
/// Returns an error if stdin or the file cannot be read.
pub(super) fn read_source(source: &str) -> Result<String> {
    if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("read canonical task-JSON from stdin")?;
        return Ok(buf);
    }
    std::fs::read_to_string(source).with_context(|| format!("read {source}"))
}

/// Parse `raw` into the canonical records it holds: a bare JSON array, or the
/// `{ "tasks": [...] }` / `{ "tasks": ..., "applied": ... }` envelope.
///
/// # Errors
/// Returns an error if `raw` is not valid JSON, is neither an array nor an
/// object with a `tasks` array, or a record does not match the
/// [`CanonicalTask`] schema.
pub(super) fn parse_records(raw: &str) -> Result<Vec<CanonicalTask>> {
    let value: Value = serde_json::from_str(raw).context("parse canonical task-JSON")?;
    extract_array(value)?
        .into_iter()
        .enumerate()
        .map(|(idx, item)| {
            serde_json::from_value(item)
                .with_context(|| format!("parse canonical record #{}", idx + 1))
        })
        .collect()
}

/// Pull the record array out of either accepted top-level shape.
fn extract_array(value: Value) -> Result<Vec<Value>> {
    match value {
        Value::Array(items) => Ok(items),
        Value::Object(mut map) => match map.remove("tasks") {
            Some(Value::Array(items)) => Ok(items),
            Some(_) => bail!("`tasks` must be a JSON array"),
            None => bail!("expected a JSON array or a {{\"tasks\": [...]}} envelope"),
        },
        other => bail!("expected a JSON array or object, got {other}"),
    }
}
