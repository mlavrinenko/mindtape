//! The task item model and its projection from a harvested marker.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use typst_harvest::HVal;

use crate::numeric::NumValue;

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
    /// Identity: defaults to the file stem; overridden by an explicit `id:`
    /// field in the task marker.
    pub id: String,
    /// Path to the source file, relative to the project root.
    pub file: PathBuf,
    /// Human-readable title.
    pub title: String,
    /// Lifecycle status.
    pub status: Option<Status>,
    /// Priority as a numeric-field value (a point on the scale or a band label).
    pub priority: Option<NumValue>,
    /// Difficulty as a numeric-field value (a point on the scale or an anchor
    /// label).
    pub difficulty: Option<NumValue>,
    /// Kind enum value.
    pub kind: Option<String>,
    /// Estimate as an opaque `unit -> quantity` map (e.g. `{days: 5, hours: 5}`
    /// or `{points: 8}`). Units are free labels; never converted or compared.
    pub estimate: BTreeMap<String, f64>,
    /// Raw link targets from `depends-on` (paths as written in the DSL, e.g.
    /// `"other.typ"` or `"../sub/task.typ"`). Resolved to file paths at
    /// validation time; history-pointer entries are excluded.
    pub depends_on: Vec<String>,
    /// Raw link targets from `relates-to`. See [`depends_on`](Self::depends_on).
    pub relates_to: Vec<String>,
    /// Raw link targets from `supersedes`. See [`depends_on`](Self::depends_on).
    pub supersedes: Vec<String>,
    /// Raw link targets from `duplicates`. See [`depends_on`](Self::depends_on).
    pub duplicates: Vec<String>,
    /// Tags.
    pub tags: Vec<String>,
    /// Due date, `YYYY-MM-DD`.
    pub due: Option<String>,
    /// Start date, `YYYY-MM-DD`.
    pub start: Option<String>,
    /// Last re-affirmation date, `YYYY-MM-DD`.
    pub reviewed: Option<String>,
    /// Opaque consumer extensions, keyed by `x-<ns>` slot.
    pub extras: BTreeMap<String, HVal>,
}

impl Task {
    /// Build a task from its file stem, relative path, and a `mindtape.task`
    /// marker value (the `(marker, fields, extras)` dict).
    ///
    /// An explicit non-empty `id:` in `fields` wins; otherwise the `stem` is
    /// used as the identity.
    #[must_use]
    pub fn from_marker(stem: String, file: PathBuf, marker: &HVal) -> Self {
        let fields = marker.get("fields");
        let extras = marker
            .get("extras")
            .and_then(HVal::as_dict)
            .cloned()
            .unwrap_or_default();
        let explicit_id = field_str(fields, "id").filter(|s| !s.is_empty());
        let id = explicit_id.unwrap_or(stem);
        Self {
            id,
            file,
            title: field_str(fields, "title").unwrap_or_default(),
            status: parse_status(field(fields, "status")),
            priority: parse_num_value(field(fields, "priority")),
            difficulty: parse_num_value(field(fields, "difficulty")),
            kind: field_str(fields, "kind"),
            estimate: parse_num_map(field(fields, "estimate")),
            depends_on: parse_links(field(fields, "depends-on")),
            relates_to: parse_links(field(fields, "relates-to")),
            supersedes: parse_links(field(fields, "supersedes")),
            duplicates: parse_links(field(fields, "duplicates")),
            tags: parse_str_array(field(fields, "tags")),
            due: field_str(fields, "due"),
            start: field_str(fields, "start"),
            reviewed: field_str(fields, "reviewed"),
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

/// Whether a link target is a history pointer (a SWHID tombstone) rather than a
/// task↔task edge. `cleanup` rewrites a removed task's inbound links into these;
/// they name no loaded task, so the model never surfaces them as graph edges and
/// validation never resolves them. Delegates the scheme check to the crate that
/// owns the SWHID parser.
#[must_use]
pub fn is_history_pointer(target: &str) -> bool {
    swhid_mint::is_swhid_url(target)
}

/// Normalize a link target (a stem or path like `foo.task-260101.typ`) to the
/// file-stem identity used by the `cleanup` command for matching against task
/// identities. Not used for link resolution in validation; see
/// [`resolve_link_path`] for path-based resolution.
#[must_use]
pub fn normalize_link(target: &str) -> String {
    Path::new(target).file_stem().map_or_else(
        || target.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

/// Resolve a raw link `target` (as written in the Typst DSL, e.g. `"other.typ"`
/// or `"../sub/task.typ"`) relative to the directory of the linking task's
/// `task_file` (relative to project root). Returns a normalised path relative to
/// the project root, with `..` components collapsed without touching the
/// filesystem.
///
/// Returns `None` if the resulting path escapes the project root (too many `..`).
#[must_use]
pub fn resolve_link_path(task_file: &Path, target: &str) -> Option<PathBuf> {
    // Start from the directory of the linking task.
    let base = task_file.parent().unwrap_or(Path::new(""));
    // Build a component list and push each component of base, then target.
    let mut components: Vec<&str> = Vec::new();
    for c in base.components() {
        match c {
            std::path::Component::Normal(s) => {
                components.push(s.to_str()?);
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                components.pop()?; // None means escapes root
            }
            _ => return None, // absolute or prefix — unexpected
        }
    }
    for c in Path::new(target).components() {
        match c {
            std::path::Component::Normal(s) => {
                components.push(s.to_str()?);
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                components.pop()?; // None means escapes root
            }
            _ => return None, // absolute or prefix — unexpected
        }
    }
    Some(components.iter().collect())
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

/// Project a marker field into a numeric-field value: a number becomes
/// [`NumValue::Num`], a string a [`NumValue::Label`]; anything else is absent.
fn parse_num_value(value: Option<&HVal>) -> Option<NumValue> {
    let value = value?;
    if let Some(num) = value.as_f64() {
        Some(NumValue::Num(num))
    } else {
        value.as_str().map(|text| NumValue::Label(text.to_owned()))
    }
}

fn parse_num_map(value: Option<&HVal>) -> BTreeMap<String, f64> {
    value
        .and_then(HVal::as_dict)
        .map(|dict| {
            dict.iter()
                .filter_map(|(key, val)| val.as_f64().map(|num| (key.clone(), num)))
                .collect()
        })
        .unwrap_or_default()
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
                .filter(|target| !is_history_pointer(target))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
