//! The task item model and its projection from a harvested marker.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use typst_harvest::HVal;

/// A task's status: a lifecycle name plus when it was set and an optional note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    /// Lifecycle name, e.g. `proposed`, `wip`, `done`.
    pub name: String,
    /// Date the status was set, `YYYY-MM-DD`.
    pub date: Option<String>,
    /// Free-form note.
    pub note: Option<String>,
}

/// One task: the portable item model harvested from a per-file artifact.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    /// Identity: the file stem (e.g. `implement-auth`).
    pub id: String,
    /// Path to the source file, relative to the project root.
    pub file: PathBuf,
    /// Human-readable title.
    pub title: String,
    /// Lifecycle status.
    pub status: Option<Status>,
    /// Priority enum value.
    pub priority: Option<String>,
    /// Grade enum value.
    pub grade: Option<String>,
    /// Kind enum value.
    pub kind: Option<String>,
    /// Estimated hours.
    pub estimate_hours: Option<f64>,
    /// Task ids this task depends on (normalized stems).
    pub depends_on: Vec<String>,
    /// Task ids this task relates to (normalized stems).
    pub relates_to: Vec<String>,
    /// Tags.
    pub tags: Vec<String>,
    /// Due date, `YYYY-MM-DD`.
    pub due: Option<String>,
    /// Start date, `YYYY-MM-DD`.
    pub start: Option<String>,
    /// Opaque consumer extensions, keyed by `x-<ns>` slot.
    pub extras: BTreeMap<String, HVal>,
}

impl Task {
    /// Build a task from its file stem, relative path, and a `mindtape.task`
    /// marker value (the `(marker, fields, extras)` dict).
    #[must_use]
    pub fn from_marker(id: String, file: PathBuf, marker: &HVal) -> Self {
        let fields = marker.get("fields");
        let extras = marker
            .get("extras")
            .and_then(HVal::as_dict)
            .cloned()
            .unwrap_or_default();
        Self {
            id,
            file,
            title: field_str(fields, "title").unwrap_or_default(),
            status: parse_status(field(fields, "status")),
            priority: field_str(fields, "priority"),
            grade: field_str(fields, "grade"),
            kind: field_str(fields, "kind"),
            estimate_hours: field(fields, "estimate-hours").and_then(HVal::as_f64),
            depends_on: parse_links(field(fields, "depends-on")),
            relates_to: parse_links(field(fields, "relates-to")),
            tags: parse_str_array(field(fields, "tags")),
            due: field_str(fields, "due"),
            start: field_str(fields, "start"),
            extras,
        }
    }

    /// Whether this task's status is in the given terminal set.
    #[must_use]
    pub fn is_terminal(&self, terminal: &[String]) -> bool {
        self.status
            .as_ref()
            .is_some_and(|status| terminal.iter().any(|name| name == &status.name))
    }
}

/// Normalize a link target (a stem or path like `foo.task-260101.typ`) to the
/// file-stem identity used for resolution.
#[must_use]
pub fn normalize_link(target: &str) -> String {
    Path::new(target).file_stem().map_or_else(
        || target.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

fn field<'a>(fields: Option<&'a HVal>, key: &str) -> Option<&'a HVal> {
    fields
        .and_then(|dict| dict.get(key))
        .filter(|value| !value.is_null())
}

fn field_str(fields: Option<&HVal>, key: &str) -> Option<String> {
    field(fields, key)
        .and_then(HVal::as_str)
        .map(ToOwned::to_owned)
}

fn parse_status(value: Option<&HVal>) -> Option<Status> {
    let dict = value?;
    let name = dict.get("name").and_then(HVal::as_str)?.to_owned();
    Some(Status {
        name,
        date: dict
            .get("date")
            .and_then(HVal::as_str)
            .map(ToOwned::to_owned),
        note: dict
            .get("note")
            .and_then(HVal::as_str)
            .map(ToOwned::to_owned),
    })
}

fn parse_str_array(value: Option<&HVal>) -> Vec<String> {
    value
        .and_then(HVal::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_links(value: Option<&HVal>) -> Vec<String> {
    value
        .and_then(HVal::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(HVal::as_str)
                .map(normalize_link)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
