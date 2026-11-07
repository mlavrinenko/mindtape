//! Read and parse canonical task-JSON input: a file arg or stdin (`-`), as a
//! bare array (`mt ls --json`'s default shape) or the `{ "tasks": [...] }` /
//! `{ "tasks": ..., "applied": ... }` envelope (`mt ls --json`'s rule-applied
//! shape). `applied` is ignored either way, and an unknown top-level key is
//! ignored so an export from an older `mt` still imports.

use std::io::Read as _;

use std::path::Path;

use crate::diag::Result;
use mindtape::CanonicalTask;
use serde_json::Value;

use crate::diag::{Diag, Hint, Kind};

/// Read `source` to a string: `-` reads stdin, anything else is a file path.
///
/// # Errors
/// Returns an error if stdin or the file cannot be read.
pub(super) fn read_source(source: &str) -> Result<String> {
    if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|err| Diag::failed("can't read stdin").io(&err))?;
        return Ok(buf);
    }
    std::fs::read_to_string(source).map_err(|err| Diag::cannot_read(Path::new(source), &err))
}

/// How `source` reads in a message: the file, or `stdin` for `-`.
fn place_of(source: &str) -> &Path {
    Path::new(if source == "-" { "stdin" } else { source })
}

/// Parse `raw`, which was read from `source`, into the canonical records it
/// holds: a bare JSON array, or the `{ "tasks": [...] }` / `{ "tasks": ...,
/// "applied": ... }` envelope.
///
/// # Errors
/// Returns an error if `raw` is not valid JSON, is neither an array nor an
/// object with a `tasks` array, or a record does not match the
/// [`CanonicalTask`] schema.
pub(super) fn parse_records(source: &str, raw: &str) -> Result<Vec<CanonicalTask>> {
    let place = place_of(source);
    let value: Value = serde_json::from_str(raw).map_err(|err| {
        let message = err.to_string();
        let reason = message
            .rsplit_once(" at line ")
            .map_or(message.as_str(), |(head, _)| head);
        Diag::verbatim(Kind::Failed, format!("not JSON: {reason}")).at(
            place,
            err.line(),
            Some(err.column()),
        )
    })?;
    extract_array(value)
        .map_err(|diag| diag.at_file(place))?
        .into_iter()
        .enumerate()
        .map(|(idx, item)| {
            serde_json::from_value(item).map_err(|err| {
                Diag::verbatim(Kind::Failed, format!("record #{}: {err}", idx + 1)).at_file(place)
            })
        })
        .collect()
}

/// Pull the record array out of either accepted top-level shape. An unknown
/// top-level key is ignored.
fn extract_array(value: Value) -> Result<Vec<Value>, Diag> {
    match value {
        Value::Array(items) => Ok(items),
        Value::Object(mut map) => match map.remove("tasks") {
            Some(Value::Array(items)) => Ok(items),
            Some(_) => Err(Diag::failed("`tasks` must be a JSON array")),
            None => Err(Diag::failed("expected a JSON array or a tasks envelope")
                .hint(Hint::example(r#"{"tasks": [...]}"#))),
        },
        other => Err(Diag::failed("expected a JSON array or object").value(json_kind(&other))),
    }
}

/// What kind of JSON `value` is, for a message about the wrong kind.
fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::{json_kind, parse_records};
    use serde_json::json;

    #[test]
    fn bare_array_with_no_schema_version_parses() {
        let records = parse_records("t.json", r#"[{"title": "Solo"}]"#)
            .expect("bare array must parse without it");
        assert_eq!(records.len(), 1);
        assert_eq!(records.first().map(|r| r.title.as_str()), Some("Solo"));
    }

    #[test]
    fn envelope_without_schema_version_parses() {
        let records = parse_records("t.json", r#"{"tasks": [{"title": "Solo"}]}"#)
            .expect("envelope without it must still parse");
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn a_top_level_unknown_key_is_ignored() {
        let raw = r#"{"nonsense": 99, "tasks": [{"title": "Solo"}]}"#;
        let records = parse_records("t.json", raw).expect("the key is unknown, not checked");
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn a_json_value_that_is_neither_array_nor_object_names_its_kind() {
        for (raw, kind) in [
            ("null", "null"),
            ("true", "boolean"),
            ("5", "number"),
            (r#""x""#, "string"),
        ] {
            let err = match parse_records("t.json", raw) {
                Ok(records) => panic!("expected rejection, got {} records", records.len()),
                Err(err) => crate::diag::plain(&err),
            };
            assert_eq!(
                err,
                format!("error: t.json: expected a JSON array or object: '{kind}'\n")
            );
        }
    }

    /// `extract_array` only asks `json_kind` about the kinds it rejects, so the
    /// two it accepts, array and object, are pinned here directly.
    #[test]
    fn json_kind_names_every_kind() {
        for (value, kind) in [
            (json!(null), "null"),
            (json!(true), "boolean"),
            (json!(5), "number"),
            (json!("x"), "string"),
            (json!([]), "array"),
            (json!({}), "object"),
        ] {
            assert_eq!(json_kind(&value), kind);
        }
    }
}
