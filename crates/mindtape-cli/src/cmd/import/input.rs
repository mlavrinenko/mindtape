//! Read and parse canonical task-JSON input: a file arg or stdin (`-`), as a
//! bare array (`mt ls --json`'s default shape) or the `{ "tasks": [...] }` /
//! `{ "tasks": ..., "applied": ... }` envelope (`mt ls --json`'s rule-applied
//! shape). `applied` is ignored either way. A top-level `schema_version` is
//! checked when present (`model/canonical.rs`'s [`SCHEMA_VERSION`]); absence is
//! tolerated for hand-written or jq-produced records
//! (`truth/tasks/emit-schema-version-in-the-ls-json-envelope.typ`).

use std::io::Read as _;

use std::path::Path;

use crate::diag::Result;
use mindtape::{CanonicalTask, SCHEMA_VERSION};
use serde_json::Value;

use crate::diag::{Diag, Hint, Kind, Note};

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

/// Pull the record array out of either accepted top-level shape, checking a
/// document-level `schema_version` first when the input is an object.
fn extract_array(value: Value) -> Result<Vec<Value>, Diag> {
    match value {
        Value::Array(items) => Ok(items),
        Value::Object(mut map) => {
            check_schema_version(map.get("schema_version"))?;
            match map.remove("tasks") {
                Some(Value::Array(items)) => Ok(items),
                Some(_) => Err(Diag::failed("`tasks` must be a JSON array")),
                None => Err(Diag::failed("expected a JSON array or a tasks envelope")
                    .hint(Hint::example(r#"{"tasks": [...]}"#))),
            }
        }
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

/// Check an envelope's optional `schema_version` against the version this
/// binary understands ([`SCHEMA_VERSION`]). Absent is fine (older,
/// hand-written, or jq-produced input never carries it); present-and-no-newer
/// is fine (this schema is a superset of every earlier one so far); a version
/// newer than this binary knows is rejected outright, naming both versions,
/// rather than risk silently mis-parsing a shape this binary has never seen.
///
/// # Errors
/// Returns an error when `version` is present but not a non-negative
/// integer, or is an integer greater than [`SCHEMA_VERSION`].
fn check_schema_version(version: Option<&Value>) -> Result<(), Diag> {
    let Some(version) = version else {
        return Ok(());
    };
    let found = version.as_u64().ok_or_else(|| {
        Diag::failed("`schema_version` must be a non-negative integer").value(version.to_string())
    })?;
    if found > u64::from(SCHEMA_VERSION) {
        return Err(
            Diag::refused("schema_version is newer than mt import supports")
                .value(found.to_string())
                .note(Note::new("the newest supported is").name(SCHEMA_VERSION.to_string()))
                .hint(Hint::new("upgrade mt to import this input")),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_records;

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
    fn envelope_with_current_schema_version_parses() {
        let raw = format!(
            r#"{{"schema_version": {}, "tasks": [{{"title": "Solo"}}]}}"#,
            super::SCHEMA_VERSION
        );
        let records = parse_records("t.json", &raw).expect("the current version must be accepted");
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn envelope_with_newer_schema_version_is_rejected() {
        let too_new = u64::from(super::SCHEMA_VERSION) + 1;
        let raw = format!(r#"{{"schema_version": {too_new}, "tasks": []}}"#);
        let err = match parse_records("t.json", &raw) {
            Ok(records) => panic!("expected rejection, got {} records", records.len()),
            Err(err) => crate::diag::plain(&err),
        };
        assert!(err.contains(&too_new.to_string()), "{err}");
        assert!(err.contains(&super::SCHEMA_VERSION.to_string()), "{err}");
    }

    #[test]
    fn schema_version_must_be_a_non_negative_integer() {
        let err = match parse_records("t.json", r#"{"schema_version": "one", "tasks": []}"#) {
            Ok(records) => panic!("expected rejection, got {} records", records.len()),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("schema_version"), "{err}");
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
}
