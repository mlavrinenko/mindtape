//! Read and parse canonical task-JSON input: a file arg or stdin (`-`), as a
//! bare array (`mt ls --json`'s default shape) or the `{ "tasks": [...] }` /
//! `{ "tasks": ..., "applied": ... }` envelope (`mt ls --json`'s rule-applied
//! shape). `applied` is ignored either way. A top-level `schema_version` is
//! checked when present (`model/canonical.rs`'s [`SCHEMA_VERSION`]); absence is
//! tolerated for hand-written or jq-produced records
//! (`truth/tasks/emit-schema-version-in-the-ls-json-envelope.typ`).

use std::io::Read as _;

use anyhow::{Context, Result, bail};
use mindtape::{CanonicalTask, SCHEMA_VERSION};
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

/// Pull the record array out of either accepted top-level shape, checking a
/// document-level `schema_version` first when the input is an object.
fn extract_array(value: Value) -> Result<Vec<Value>> {
    match value {
        Value::Array(items) => Ok(items),
        Value::Object(mut map) => {
            check_schema_version(map.get("schema_version"))?;
            match map.remove("tasks") {
                Some(Value::Array(items)) => Ok(items),
                Some(_) => bail!("`tasks` must be a JSON array"),
                None => bail!("expected a JSON array or a {{\"tasks\": [...]}} envelope"),
            }
        }
        other => bail!("expected a JSON array or object, got {other}"),
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
fn check_schema_version(version: Option<&Value>) -> Result<()> {
    let Some(version) = version else {
        return Ok(());
    };
    let found = version.as_u64().with_context(|| {
        format!("`schema_version` must be a non-negative integer, got {version}")
    })?;
    if found > u64::from(SCHEMA_VERSION) {
        bail!(
            "canonical task-JSON schema_version {found} is newer than mt import supports \
             (supports up to {SCHEMA_VERSION}); upgrade mt to import this input"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_records;

    #[test]
    fn bare_array_with_no_schema_version_parses() {
        let records =
            parse_records(r#"[{"title": "Solo"}]"#).expect("bare array must parse without it");
        assert_eq!(records.len(), 1);
        assert_eq!(records.first().map(|r| r.title.as_str()), Some("Solo"));
    }

    #[test]
    fn envelope_without_schema_version_parses() {
        let records = parse_records(r#"{"tasks": [{"title": "Solo"}]}"#)
            .expect("envelope without it must still parse");
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn envelope_with_current_schema_version_parses() {
        let raw = format!(
            r#"{{"schema_version": {}, "tasks": [{{"title": "Solo"}}]}}"#,
            super::SCHEMA_VERSION
        );
        let records = parse_records(&raw).expect("the current version must be accepted");
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn envelope_with_newer_schema_version_is_rejected() {
        let too_new = u64::from(super::SCHEMA_VERSION) + 1;
        let raw = format!(r#"{{"schema_version": {too_new}, "tasks": []}}"#);
        let err = match parse_records(&raw) {
            Ok(records) => panic!("expected rejection, got {} records", records.len()),
            Err(err) => err.to_string(),
        };
        assert!(err.contains(&too_new.to_string()), "{err}");
        assert!(err.contains(&super::SCHEMA_VERSION.to_string()), "{err}");
    }

    #[test]
    fn schema_version_must_be_a_non_negative_integer() {
        let err = match parse_records(r#"{"schema_version": "one", "tasks": []}"#) {
            Ok(records) => panic!("expected rejection, got {} records", records.len()),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("schema_version"), "{err}");
    }
}
