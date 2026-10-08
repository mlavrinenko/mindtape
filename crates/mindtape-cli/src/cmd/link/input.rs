//! `mt link --apply`'s input: the adjacency map, read as YAML.
//!
//! One parser covers both advertised formats — YAML 1.2 is a superset of JSON,
//! so a `.json` adjacency map parses here unchanged, with no extension sniffing
//! and no second error vocabulary (`cmd/add/values.rs` sniffs because a `--set`
//! patch's *values* are typed differently by the two parsers; an adjacency map
//! is strings all the way down, so nothing here can disagree).
//!
//! The shape, stem to kind to targets:
//!
//! ```yaml
//! seed-content-from-telegram-export:
//!   depends-on: [article-content-type, error-record-content-type]
//!   related:
//!     - target: telegram-export-format
//!       note: same export shape
//! ```
//!
//! A target is a bare string, or a mapping carrying its own `note`. The whole
//! document is walked by hand rather than deserialized into a typed tree: every
//! rejection then names the stem and the kind it came from, which an `untagged`
//! enum's "data did not match any variant" cannot.

use std::io::Read as _;

use serde_yaml_ng::Value;

use crate::cmd::link::edit::EdgeSpec;
use crate::diag::{Diag, Hint, Note};

type Result<T> = std::result::Result<T, Diag>;

/// One task's requested edges, as read from the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The linking task, by filename stem — the map's top-level key.
    pub stem: String,
    /// Every edge declared under it, in document order.
    pub edges: Vec<EdgeSpec>,
}

/// Read `source` to a string: `-` reads stdin, anything else is a file path.
///
/// # Errors
/// Returns an error if stdin or the file cannot be read.
pub fn read_source(source: &str) -> Result<String> {
    if source == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|err| Diag::failed("can't read the adjacency map from stdin").io(&err))?;
        return Ok(text);
    }
    std::fs::read_to_string(source).map_err(|err| {
        Diag::failed("can't read the adjacency map")
            .value(source)
            .io(&err)
    })
}

/// Parse an adjacency map into one [`Entry`] per stem, in document order.
///
/// # Errors
/// Returns an error when `text` is not a YAML/JSON mapping of stem to kind to
/// targets, naming the offending stem and kind.
pub fn parse_adjacency(text: &str) -> Result<Vec<Entry>> {
    let value: Value = serde_yaml_ng::from_str(text)
        .map_err(|err| Diag::usage("can't parse the adjacency map").cause(&err))?;
    let mapping = match value {
        Value::Mapping(mapping) => mapping,
        // An empty file is an empty map, not a malformed one: `mt link --apply`
        // over a filtered pipeline that matched nothing has nothing to do, and
        // "nothing to do" is not an error.
        Value::Null => return Ok(Vec::new()),
        other => {
            return Err(Diag::usage(
                "the adjacency map must be a mapping of stem to kind to targets",
            )
            .note(Note::new("got").name(shape_of(&other))));
        }
    };
    mapping
        .iter()
        .map(|(key, value)| entry(key, value))
        .collect()
}

/// One top-level `stem: { kind: targets }` pair.
fn entry(key: &Value, value: &Value) -> Result<Entry> {
    let stem = key
        .as_str()
        .ok_or_else(|| Diag::usage("adjacency map keys must be task stems (strings)"))?
        .to_owned();
    let kinds = value
        .as_mapping()
        .ok_or_else(|| Diag::usage("expected a mapping of kind to targets").value(stem.as_str()))?;
    let mut edges = Vec::new();
    for (kind, targets) in kinds {
        let kind = kind
            .as_str()
            .ok_or_else(|| Diag::usage("link kinds must be strings").value(stem.as_str()))?;
        edges.extend(kind_edges(&stem, kind, targets)?);
    }
    Ok(Entry { stem, edges })
}

/// Every edge one `kind:` key declares. A bare string is a single target, so
/// `parent: umbrella-task` needs no list.
fn kind_edges(stem: &str, kind: &str, targets: &Value) -> Result<Vec<EdgeSpec>> {
    let items = match targets {
        Value::Sequence(items) => items.as_slice(),
        single => return Ok(vec![target_edge(stem, kind, single)?]),
    };
    items
        .iter()
        .map(|item| target_edge(stem, kind, item))
        .collect()
}

/// One target entry: a bare string, or a mapping carrying `target` and `note`.
fn target_edge(stem: &str, kind: &str, item: &Value) -> Result<EdgeSpec> {
    // Every rejection names the task and the kind it came from.
    let refuse = |what: &'static str| Diag::usage(what).rule(stem).context("under", kind);
    if let Some(target) = item.as_str() {
        return Ok(EdgeSpec {
            kind: kind.to_owned(),
            target: target.to_owned(),
            note: None,
        });
    }
    let mapping = item.as_mapping().ok_or_else(|| {
        refuse("a target must be a stem string or a {target, note} mapping")
            .note(Note::new("got").name(shape_of(item)))
    })?;
    for key in mapping.keys() {
        let name = key.as_str().unwrap_or_default();
        if !matches!(name, "target" | "note") {
            return Err(refuse("unknown target key")
                .value(name)
                .hint(Hint::new("a target mapping takes target and note")));
        }
    }
    let target = mapping
        .get(Value::from("target"))
        .and_then(Value::as_str)
        .ok_or_else(|| refuse("a target mapping needs a target key naming the task stem"))?;
    let note = mapping
        .get(Value::from("note"))
        .map(|note| {
            note.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| refuse("note must be a string"))
        })
        .transpose()?;
    Ok(EdgeSpec {
        kind: kind.to_owned(),
        target: target.to_owned(),
        note,
    })
}

/// A YAML value's shape, for an error that says what arrived instead.
fn shape_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Sequence(_) => "a list",
        Value::Mapping(_) => "a mapping",
        Value::Tagged(_) => "a tagged value",
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
