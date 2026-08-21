//! The task item model and its projection from a harvested marker.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use typst_harvest::HVal;

use crate::numeric::{InputValue, NumValue};

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

/// A core field whose marker value did not match its expected Typst shape
/// (e.g. a string where `tags:` expects an array). Detected during
/// [`Task::from_marker`] — the only place that still sees the raw,
/// un-degraded value — instead of silently coercing the field to empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeError {
    /// Field name (`tags`, `links`, `estimate`, `recur`).
    pub field: String,
    /// Human-readable cause, naming the expected shape and what was found.
    pub message: String,
}

/// One task: the portable item model harvested from a per-file artifact.
#[derive(Debug, Clone, Default, PartialEq)]
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
    /// The actor currently working the task. Orthogonal to status; not
    /// ownership. Plain free text until a project constrains it via
    /// `[tasks.assignee]` (allowed list) or the actors registry.
    pub assignee: Option<String>,
    /// Estimate as an opaque `unit -> quantity` map (e.g. `{days: 5, hours: 5}`
    /// or `{points: 8}`). Units are free labels; never converted or compared.
    pub estimate: BTreeMap<String, f64>,
    /// Typed relation edges from `links:` (task↔task dependency, hierarchy,
    /// and cross-reference edges), each carrying a kind and optional note.
    /// Targets are raw paths as written in the DSL (e.g. `"other.typ"` or
    /// `"../sub/task.typ"`), resolved to file paths at validation time;
    /// history-pointer entries are excluded. See [`crate::link::parse_links`].
    pub links: Vec<crate::link::Link>,
    /// Authored tags, exactly as written in the file.
    pub tags: Vec<String>,
    /// Folder-derived virtual tags, injected at load by [`crate::derive`]. Never
    /// written to the file; empty unless the task's root enables derivation.
    pub derived_tags: Vec<String>,
    /// Nesting depth of the file under its root: 0 for a task sitting directly
    /// in the root, 1 for one in a sub-folder, and so on. Computed at load from
    /// the file's location (like [`derived_tags`](Self::derived_tags)); never
    /// authored. Zero when constructed without a loader (e.g. [`Task::from_marker`]).
    pub depth: u8,
    /// Level in the dependency graph: 0 for a task with no in-set blocking
    /// dependency, otherwise one more than the deepest task it depends on.
    /// Ordering by it puts every dependency ahead of its dependents, and leaves
    /// independent tasks tied for a secondary key to break — unlike
    /// [`depth`](Self::depth), which is folder nesting and says nothing about
    /// the graph. Derived over the whole loaded set by [`crate::dep_rank`];
    /// zero when constructed without a loader.
    pub topo: u32,
    /// How many non-terminal tasks transitively wait on this one — the work
    /// finishing it would unblock. Zero for a terminal task, which blocks
    /// nothing. Derived over the whole loaded set by [`crate::dep_rank`]; zero
    /// when constructed without a loader.
    pub blocks: u32,
    /// Due date, `YYYY-MM-DD`.
    pub due: Option<String>,
    /// Start date, `YYYY-MM-DD`.
    pub start: Option<String>,
    /// Last re-affirmation date, `YYYY-MM-DD`.
    pub reviewed: Option<String>,
    /// Recurrence rule + policy (`every()`/`rrule()`), when the task recurs.
    /// Advanced only by an explicit `mt recur` — see [`crate::recur`].
    pub recur: Option<crate::recur::Recur>,
    /// Opaque consumer extensions, keyed by `x-<ns>` slot.
    pub extras: BTreeMap<String, HVal>,
    /// Core fields present in the marker but the wrong Typst shape — coerced
    /// to empty/absent same as an absent field, but flagged here so `check`
    /// can report it instead of staying silent. See [`ShapeError`].
    pub shape_errors: Vec<ShapeError>,
    /// Raw Typst source after the `#show: task.with(…)` marker — the prose body
    /// (`== Summary`, notes), verbatim and whitespace-trimmed. Populated by the
    /// loader from the file source; empty when constructed without a loader (e.g.
    /// [`Task::from_marker`], which sees no source). The emitter appends it back
    /// verbatim, so a `mt ls --json | mt import` round-trip preserves the body.
    pub body: String,
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
        let mut shape_errors = Vec::new();
        let tags = parse_str_array(field(fields, "tags"), "tags", &mut shape_errors);
        let estimate = parse_num_map(field(fields, "estimate"), "estimate", &mut shape_errors);
        let links = crate::link::parse_links(field(fields, "links"), &mut shape_errors);
        let recur = crate::recur::parse_recur(field(fields, "recur"), &mut shape_errors);
        Self {
            id,
            file,
            title: field_str(fields, "title").unwrap_or_default(),
            status: parse_status(field(fields, "status")),
            priority: parse_num_value(field(fields, "priority")),
            difficulty: parse_num_value(field(fields, "difficulty")),
            kind: field_str(fields, "kind"),
            assignee: field_str(fields, "assignee").filter(|s| !s.is_empty()),
            estimate,
            links,
            tags,
            derived_tags: Vec::new(),
            depth: 0,
            topo: 0,
            blocks: 0,
            due: field_str(fields, "due"),
            start: field_str(fields, "start"),
            reviewed: field_str(fields, "reviewed"),
            recur,
            extras,
            shape_errors,
            body: String::new(),
        }
    }

    /// Authored and derived tags, deduped (authored first, then derived).
    #[must_use]
    pub fn all_tags(&self) -> Vec<String> {
        let mut out = self.tags.clone();
        for tag in &self.derived_tags {
            if !out.contains(tag) {
                out.push(tag.clone());
            }
        }
        out
    }

    /// Whether this task's status is in the given terminal set.
    #[must_use]
    pub fn is_terminal(&self, terminal: &[String]) -> bool {
        self.status
            .as_ref()
            .is_some_and(|status| terminal.iter().any(|name| name == &status.name))
    }

    /// Raw targets of `links:` edges of the given `kind` (e.g. `"depends-on"`,
    /// `"parent"`), in authored order. The shared read path for readiness, the
    /// dependency graph, and the filter language's `links(kind)` predicate.
    pub fn link_targets<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.links
            .iter()
            .filter(move |link| link.kind == kind)
            .map(|link| link.task.as_str())
    }

    /// `file` as a portable, `/`-separated string — the root-relative path the
    /// filter language's `file` field and the `--sort file` key both read (see
    /// `truth/tasks/filter-and-sort-tasks-by-file.typ`). Built from
    /// [`Path::components`] and rejoined with `/` rather than [`Path::display`],
    /// so the same value comes out on every platform: `Display` would print the
    /// OS-native separator (`\` on Windows), making a filter or `[views.*]`
    /// expression written on one machine not match on another. This is the
    /// stored value, not the cwd-relative form the `file` display column
    /// renders (`crates/mindtape-cli/src/render/file_path.rs`) — display is
    /// localized, the stored value is not, the same split dates already have
    /// between the ISO value and `[ls] date-format`.
    #[must_use]
    pub fn file_key(&self) -> String {
        self.file
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
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
    // Build a component list from base, then append target's components.
    let mut components: Vec<&str> = Vec::new();
    push_components(&mut components, base)?;
    push_components(&mut components, Path::new(target))?;
    Some(components.iter().collect())
}

/// Append `path`'s components to `out`, collapsing `.` and `..` against the
/// accumulated list. Returns `None` if a `..` pops past the root or a component
/// is absolute/non-UTF-8 — the same escape semantics as [`resolve_link_path`].
fn push_components<'a>(out: &mut Vec<&'a str>, path: &'a Path) -> Option<()> {
    for c in path.components() {
        match c {
            std::path::Component::Normal(s) => {
                out.push(s.to_str()?);
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop()?; // None means escapes root
            }
            _ => return None, // absolute or prefix — unexpected
        }
    }
    Some(())
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
/// [`NumValue::Num`], a framework dict (`(framework: …, inputs: …)`) a
/// [`NumValue::Framework`], a string a [`NumValue::Label`]; else absent.
fn parse_num_value(value: Option<&HVal>) -> Option<NumValue> {
    let value = value?;
    if let Some(num) = value.as_f64() {
        return Some(NumValue::Num(num));
    }
    if let Some(dict) = value.as_dict() {
        let name = dict.get("framework").and_then(HVal::as_str)?;
        return Some(NumValue::Framework {
            name: name.to_owned(),
            inputs: parse_inputs(dict.get("inputs")),
        });
    }
    value.as_str().map(|text| NumValue::Label(text.to_owned()))
}

/// Project the `inputs` dict of a framework value into typed input values.
fn parse_inputs(value: Option<&HVal>) -> BTreeMap<String, InputValue> {
    value
        .and_then(HVal::as_dict)
        .map(|dict| {
            dict.iter()
                .filter_map(|(key, val)| input_value(val).map(|iv| (key.clone(), iv)))
                .collect()
        })
        .unwrap_or_default()
}

fn input_value(value: &HVal) -> Option<InputValue> {
    match value {
        HVal::Bool(flag) => Some(InputValue::Bool(*flag)),
        HVal::Int(_) | HVal::Float(_) => value.as_f64().map(InputValue::Num),
        HVal::Str(text) => Some(InputValue::Text(text.clone())),
        _ => None,
    }
}

fn parse_num_map(
    value: Option<&HVal>,
    field_name: &str,
    errors: &mut Vec<ShapeError>,
) -> BTreeMap<String, f64> {
    let Some(value) = value else {
        return BTreeMap::new();
    };
    let Some(dict) = value.as_dict() else {
        errors.push(shape_error(field_name, "a dict", value));
        return BTreeMap::new();
    };
    dict.iter()
        .filter_map(|(key, val)| val.as_f64().map(|num| (key.clone(), num)))
        .collect()
}

fn parse_str_array(
    value: Option<&HVal>,
    field_name: &str,
    errors: &mut Vec<ShapeError>,
) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        errors.push(shape_error(field_name, "an array", value));
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| item.as_str().map(ToOwned::to_owned))
        .collect()
}

/// Build a [`ShapeError`] naming `field_name`, the expected shape, and the
/// Typst kind of what was actually found.
pub(crate) fn shape_error(field_name: &str, expected: &str, value: &HVal) -> ShapeError {
    ShapeError {
        field: field_name.to_owned(),
        message: format!("expected {expected}, found {}", hval_kind(value)),
    }
}

fn hval_kind(value: &HVal) -> &'static str {
    match value {
        HVal::Null => "none",
        HVal::Bool(_) => "a boolean",
        HVal::Int(_) | HVal::Float(_) => "a number",
        HVal::Str(_) => "a string",
        HVal::Date(_) => "a date",
        HVal::Array(_) => "an array",
        HVal::Dict(_) => "a dict",
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
