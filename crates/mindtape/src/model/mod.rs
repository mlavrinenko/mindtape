//! The task item model and its projection from a harvested marker.
//!
//! # Files
//!
//! - `canonical.rs`: the canonical task-JSON schema shared by `mt ls --json` and `mt import`.
//! - `field_descriptor.rs`: one descriptor per described field and its allowed values.
//! - `fields.rs`: the registry of fields comparable in a filter or a `--sort` spec.
//! - `handle.rs`: stem hashes, the short code an agent types to address a task.
//! - `marker.rs`: the typed reads over a harvested `mindtape.task` marker.
//! - `naming.rs`: the `[tasks.naming]` template and the shared minter.
//! - `numeric.rs`: the numeric-field primitive: scale, classification, resolve and display.
//! - `reference.rs`: task references, and the one resolver that reads them.
//! - `task_spans.rs`: where a task file's `task.with(...)` call and its arguments sit.
//! - `writable.rs`: the fields a generic write command may set, and their literals.

pub mod canonical;
pub(crate) mod field_descriptor;
pub(crate) mod fields;
pub mod handle;
pub mod marker;
pub mod naming;
pub mod numeric;
pub mod reference;
pub mod task_spans;
pub mod writable;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use typst_harvest::HVal;

use crate::model::marker::{
    ShapeError, field, field_str, parse_num_value, parse_status, parse_str_array,
};
use crate::model::numeric::NumValue;

/// A task's status, holding a lifecycle name, the date the task entered it, and
/// an optional note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    /// Lifecycle name, such as `proposed`, `wip`, `done`.
    pub name: String,
    /// Date the task entered the status, `YYYY-MM-DD`.
    pub date: Option<String>,
    /// Free-form note.
    pub note: Option<String>,
}

/// One marker a task body emitted, kept for a check rule to read as the
/// reserved `markers` input ([`crate::rules::CheckSpec::field`]). The loader
/// keeps every harvested marker except the `mindtape.task` one, in document
/// order, so a project check can read the metadata its own body emits.
///
/// It carries no source location: a breach a check raises still lands on the
/// task's `task.with(…)` line, like every other rule's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyMarker {
    /// The marker's label (`#metadata(..) <label>`), `None` for an unlabelled
    /// marker.
    pub label: Option<String>,
    /// The marker's value, the Typst tree the harvest projected. A date in it
    /// arrives as its `YYYY-MM-DD` text, the form the cache stores.
    pub value: HVal,
}

/// One task, the portable item model harvested from a per-file artifact.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Task {
    /// The task's identity, which defaults to the file stem. An explicit `id:`
    /// field in the task marker overrides it.
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
    /// The actor currently working the task. Orthogonal to status, and not
    /// ownership. Plain free text until a project constrains it through
    /// `[tasks.assignee]` (allowed list) or the actors registry.
    pub assignee: Option<String>,
    /// Estimate as an opaque `unit -> quantity` map (such as
    /// `{days: 5, hours: 5}` or `{points: 8}`). Units are free labels, never
    /// converted or compared.
    /// A `count(...)` unit appears here once [`crate::estimate_queries::annotate`]
    /// resolves it.
    pub estimate: BTreeMap<String, f64>,
    /// The units written as `count(n, ..queries)`, as authored. See
    /// [`crate::estimate_queries`].
    pub estimate_counts: BTreeMap<String, crate::estimate_queries::CountEstimate>,
    /// Typed relation edges from `links:` (task↔task dependency, hierarchy,
    /// and cross-reference edges), each carrying a kind and optional note.
    /// Targets are raw paths as written in the DSL (such as `"other.typ"` or
    /// `"../sub/task.typ"`), resolved to file paths at validation time. The
    /// list excludes history-pointer entries. See [`crate::link::parse_links`].
    pub links: Vec<crate::link::Link>,
    /// What the task waits on outside the repo, as `checker:id` entries from
    /// `waits-on:`, in authored order. See [`crate::waits`].
    pub waits_on: Vec<String>,
    /// What the checkers answered for [`waits_on`](Self::waits_on), keyed by
    /// entry. Stamped at load by [`crate::waits::annotate`] for an open task,
    /// never authored and never cached. An entry with no answer never blocks
    /// the task: a closed task's wait is history, and nothing asks for it.
    pub answers: BTreeMap<String, crate::waits::Answer>,
    /// Authored tags, exactly as written in the file.
    pub tags: Vec<String>,
    /// Folder-derived virtual tags, injected at load by [`crate::loader::derive`]. Never
    /// written to the file, and empty unless the task's root enables derivation.
    pub derived_tags: Vec<String>,
    /// Nesting depth of the file under its root: 0 for a task sitting directly
    /// in the root, 1 for one in a sub-folder, and so on. Computed at load from
    /// the file's location (like [`derived_tags`](Self::derived_tags)), and never
    /// authored. Zero when constructed without a loader (such as
    /// [`Task::from_marker`]).
    pub depth: u8,
    /// Level in the dependency graph: 0 for a task with no in-set blocking
    /// dependency, otherwise one more than the deepest task it depends on.
    /// Ordering by it puts every dependency ahead of its dependents, and leaves
    /// independent tasks tied for a secondary key to break.
    /// [`depth`](Self::depth) differs: it is folder nesting and says nothing
    /// about the graph. [`crate::graph::dep_rank`] derives it over the whole
    /// loaded set, and it is zero when constructed without a loader.
    pub topo: u32,
    /// How many non-terminal tasks transitively wait on this one, the work that
    /// finishing it would unblock. Zero for a terminal task, which never blocks
    /// anything. [`crate::graph::dep_rank`] derives it over the whole loaded set,
    /// and it is zero when constructed without a loader.
    pub blocks: u32,
    /// How many tasks reference this one with a `parent` edge. Derived over the
    /// whole loaded set by [`crate::graph::dep_rank`], and zero when constructed
    /// without a loader.
    pub children: u32,
    /// How many of those [`children`](Self::children) are terminal.
    pub children_done: u32,
    /// Due date, `YYYY-MM-DD`.
    pub due: Option<String>,
    /// Start date, `YYYY-MM-DD`.
    pub start: Option<String>,
    /// Last re-affirmation date, `YYYY-MM-DD`.
    pub reviewed: Option<String>,
    /// Recurrence rule + policy (`every()`/`rrule()`), when the task recurs.
    /// Advanced only by an explicit `mt recur`, see [`crate::recur`].
    pub recur: Option<crate::recur::Recur>,
    /// Opaque consumer extensions, keyed by `x-<ns>` slot.
    pub extras: BTreeMap<String, HVal>,
    /// Fields `[[tasks.collectors]]` computed, keyed by the collector's name
    /// and read as `derived.<name>`. Never authored and never cached: the
    /// loader fills it from the task's source and the project's config, and
    /// `mt import` ignores it.
    pub derived: BTreeMap<String, HVal>,
    /// The reason a collector could not produce its field: an eval error, a
    /// panic or a value that does not match `returns`. `mt check` reports each
    /// as a `Collector` violation at `derived.<name>`, and `mt eval` prints it.
    pub collector_problems: Vec<crate::problem::Problem>,
    /// Core fields, and elements inside them, present in the marker but the
    /// wrong Typst shape. They coerce to empty or absent the same as an absent
    /// field, but they land here so `check` can report them instead of staying
    /// silent. Each specifies the position it occupies (`tags[1]`,
    /// `estimate.days`). See [`ShapeError`] and [`crate::model::marker`].
    pub shape_errors: Vec<ShapeError>,
    /// Raw Typst source after the `#show: task.with(…)` marker: the prose body
    /// (`== Summary`, notes), verbatim and whitespace-trimmed. The loader fills
    /// it from the file source, and it is empty when constructed without a
    /// loader (such as [`Task::from_marker`], which has no source). The emitter
    /// appends it back verbatim, so a `mt ls --json | mt import` round-trip
    /// preserves the body.
    pub body: String,
    /// Every body marker except the `mindtape.task` one, in document order,
    /// each with its label and value ([`BodyMarker`]). The loader fills it from
    /// the harvest, or from the cache entry that harvest wrote, so a warm hit
    /// and a cold eval agree. It is empty when constructed without a loader
    /// (such as [`Task::from_marker`]). No listing or eval projection prints it:
    /// the one consumer is a check rule's reserved `markers` input.
    pub markers: Vec<BodyMarker>,
    /// Where the `#show: task.with(…)` call and each of its named arguments sit
    /// in the source. A violation the validation pass raises can then point at
    /// the file rather than only name the task. The loader reads it off the
    /// source, once, alongside the [`body`](Self::body) slice after the same
    /// call. [`crate::validate::candidate`] reads it the same way, from the
    /// source it is about to write.
    ///
    /// Empty when the loader has no source. That covers a task built straight
    /// from a marker ([`Task::from_marker`], `mt import`'s records), which has
    /// no line to point at, and [`marker_location`](Self::marker_location)
    /// reports line 1.
    pub spans: crate::model::task_spans::TaskCallSpans,
}

impl Task {
    /// The `task.with(…)` call's source position, for a violation that points
    /// at the file instead of naming the task. Line 1 when the loader has no
    /// source (see [`spans`](Self::spans)).
    #[must_use]
    pub fn marker_location(&self) -> crate::validate::Location {
        crate::validate::Location::at(self.file.clone(), self.spans.call_line().unwrap_or(1))
    }

    /// Where a breach of `field` points: the line of that argument in the
    /// `task.with(…)` call, else the call's own line
    /// ([`marker_location`](Self::marker_location)). A sub-path such as
    /// `estimate.days` or `tags[1]` resolves to its root field.
    #[must_use]
    pub fn field_location(&self, field: &str) -> crate::validate::Location {
        crate::validate::Location::at(self.file.clone(), self.spans.field_line(field).unwrap_or(1))
    }

    /// Build a task from its file stem, relative path, and a `mindtape.task`
    /// marker value (the `(marker, fields, extras)` dict).
    ///
    /// An explicit non-empty `id:` in `fields` wins, and otherwise the identity
    /// is the `stem`.
    ///
    /// Every field goes through [`crate::model::marker`], so a value the
    /// projection cannot read lands in [`shape_errors`](Self::shape_errors)
    /// instead of reading as unset. That covers the field itself and an element
    /// inside it.
    #[must_use]
    pub fn from_marker(stem: String, file: PathBuf, marker: &HVal) -> Self {
        let fields = marker.get("fields");
        let extras = marker
            .get("extras")
            .and_then(HVal::as_dict)
            .cloned()
            .unwrap_or_default();
        let mut errors = Vec::new();
        let explicit_id = field_str(fields, "id", &mut errors).filter(|s| !s.is_empty());
        let id = explicit_id.unwrap_or(stem);
        let tags = parse_str_array(field(fields, "tags"), "tags", &mut errors);
        let waits_on = parse_str_array(field(fields, "waits-on"), "waits-on", &mut errors);
        let (estimate, estimate_counts) =
            crate::estimate_queries::parse_estimate(field(fields, "estimate"), &mut errors);
        let links = crate::link::parse_links(field(fields, "links"), &mut errors);
        let recur = crate::recur::parse_recur(field(fields, "recur"), &mut errors);
        Self {
            id,
            file,
            title: field_str(fields, "title", &mut errors).unwrap_or_default(),
            status: parse_status(field(fields, "status"), &mut errors),
            priority: parse_num_value(field(fields, "priority"), "priority", &mut errors),
            difficulty: parse_num_value(field(fields, "difficulty"), "difficulty", &mut errors),
            kind: field_str(fields, "kind", &mut errors),
            assignee: field_str(fields, "assignee", &mut errors).filter(|s| !s.is_empty()),
            estimate,
            estimate_counts,
            links,
            waits_on,
            answers: BTreeMap::new(),
            tags,
            derived_tags: Vec::new(),
            depth: 0,
            topo: 0,
            blocks: 0,
            children: 0,
            children_done: 0,
            due: field_str(fields, "due", &mut errors),
            start: field_str(fields, "start", &mut errors),
            reviewed: field_str(fields, "reviewed", &mut errors),
            recur,
            extras,
            derived: BTreeMap::new(),
            collector_problems: Vec::new(),
            shape_errors: errors,
            body: String::new(),
            markers: Vec::new(),
            spans: crate::model::task_spans::TaskCallSpans::default(),
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

    /// Terminal children over all children, `0..=1`, and `None` for a task with
    /// no children. A leaf then never reads as finished or unstarted work.
    #[must_use]
    pub fn progress(&self) -> Option<f64> {
        (self.children > 0).then(|| f64::from(self.children_done) / f64::from(self.children))
    }

    /// Whether this task's status is in the given terminal set.
    #[must_use]
    pub fn is_terminal(&self, terminal: &[String]) -> bool {
        self.status
            .as_ref()
            .is_some_and(|status| terminal.iter().any(|name| name == &status.name))
    }

    /// Raw targets of `links:` edges of the given `kind` (such as `"depends-on"`,
    /// `"parent"`), in authored order. The shared read path for readiness, the
    /// dependency graph, and the filter language's `links(kind)` predicate.
    pub fn link_targets<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.links
            .iter()
            .filter(move |link| link.kind == kind)
            .map(|link| link.task.as_str())
    }

    /// `file` as a portable, `/`-separated string, the root-relative path the
    /// filter language's `path` field and the `--sort path` key both read (see
    /// `truth/tasks/2026-07/filter-and-sort-tasks-by-file.typ`, and
    /// `truth/tasks/2026-09/the-file-filter-field-and-the-file-column-name-different-paths.typ`
    /// for why that field is not spelled `file`). The value comes from
    /// [`Path::components`], rejoined with `/` rather than [`Path::display`], so
    /// it is the same on every platform. `Display` would print the OS-native
    /// separator (`\` on Windows), which makes a filter or `[views.*]`
    /// expression written on one machine not match on another. This is the
    /// stored value, not the cwd-relative form the `file` display column
    /// renders (`crates/mindtape-cli/src/render/file_path.rs`). Display is
    /// localizes and the stored value does not, the same split dates already have
    /// between the ISO value and `[ls] date-format`.
    #[must_use]
    pub fn file_key(&self) -> String {
        crate::slash_path(&self.file)
    }
}

/// Whether a link target is a history pointer (a SWHID tombstone) rather than a
/// task↔task edge. `cleanup` rewrites a removed task's inbound links into these.
/// They reference no loaded task. The model then never surfaces them as graph
/// edges, and validation never resolves them. Delegates the scheme check to the
/// crate that owns the SWHID parser.
#[must_use]
pub fn is_history_pointer(target: &str) -> bool {
    swhid_mint::is_swhid_url(target)
}

/// Normalize a link target (a stem or path like `foo.task-260101.typ`) to the
/// file-stem identity used by the `cleanup` command for matching against task
/// identities. It plays no part in link resolution in validation, see
/// [`resolve_link_path`] for path-based resolution.
#[must_use]
pub fn normalize_link(target: &str) -> String {
    Path::new(target).file_stem().map_or_else(
        || target.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

/// Resolve a raw link `target` (as written in the Typst DSL, such as `"other.typ"`
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

/// The path of `to` (root-relative) seen from `from_dir` (root-relative), for
/// writing a link target: `tasks/live.typ` pointing at
/// `tasks/.archive/gone.typ` becomes `.archive/gone.typ`. The inverse of
/// [`resolve_link_path`].
#[must_use]
pub fn relative_target(from_dir: &Path, to: &Path) -> PathBuf {
    let from: Vec<_> = from_dir.components().collect();
    let to: Vec<_> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in 0..(from.len() - common) {
        out.push("..");
    }
    for component in to.iter().skip(common) {
        out.push(component.as_os_str());
    }
    out
}

/// Append `path`'s components to `out`, collapsing `.` and `..` against the
/// accumulated list. Returns `None` if a `..` pops past the root or a component
/// is absolute/non-UTF-8, with the same escape semantics as [`resolve_link_path`].
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

#[cfg(test)]
mod tests;
