//! Build a [`Task`] from a harvested file's markers and its source.
//!
//! One path serves both a cache hit and a fresh harvest, so the two cannot
//! drift on the body, its spans or its body markers. It also holds the
//! [`LoadItem`] the loader walks, and the folder-derived tags and nesting depth
//! a file's location gives it.

use std::path::{Path, PathBuf};

use typst_harvest::{HVal, Harvest};

use crate::config::Config;
use crate::loader::derive::RootDeriver;
use crate::model::task_spans::TaskCallSpans;
use crate::model::{BodyMarker, Task};

/// One task file to harvest. It holds the path, the project-relative path, the
/// tags its folder location derives, and its nesting depth under the root.
pub(super) struct LoadItem {
    pub(super) file: PathBuf,
    pub(super) rel: PathBuf,
    pub(super) folder_tags: Vec<String>,
    pub(super) depth: u8,
}

/// Build a [`Task`] from a harvested marker, the file's source and the spans
/// of its `task.with(…)` call. Identity comes from the stem (an explicit `id:`
/// still wins), the folder-derived tags come from the item, and the builder
/// slices the prose body from the source. `markers` is every other marker the
/// body emitted, in document order. Both a cache hit and a fresh harvest funnel
/// through this path, so they agree.
///
/// The caller supplies `spans` rather than letting this parse them. A cache hit
/// replays the spans the cold evaluation stored, and a warm load then never
/// parses a task file through Typst. A caller whose source has no entry parses
/// it itself ([`TaskCallSpans::parse`]).
pub(super) fn assemble_task(
    item: &LoadItem,
    marker: &HVal,
    markers: &[BodyMarker],
    source: &str,
    spans: TaskCallSpans,
) -> Task {
    let id = file_stem(&item.file);
    let mut task = Task::from_marker(id, item.rel.clone(), marker);
    task.derived_tags = item.folder_tags.clone();
    task.markers = markers.to_vec();
    task.depth = item.depth;
    task.spans = spans;
    task.body = task.spans.body(source);
    task
}

/// The markers [`assemble_task`] builds a task from: the `mindtape.task` marker
/// that makes the file a task, and every other marker the body emitted, in
/// document order. They travel together because [`assemble_task`] needs both,
/// and a caller that passed them apart would exceed its argument budget.
#[derive(Clone, Copy)]
pub(crate) struct BodyMarkers<'a> {
    /// The `mindtape.task` marker value.
    pub(crate) task: &'a HVal,
    /// Every body marker except that one, in document order.
    pub(crate) body: &'a [BodyMarker],
}

/// Build a [`Task`] for a file at project-relative `rel` from an already
/// harvested [`BodyMarkers`] and the source it came from. This is the
/// [`assemble_task`] path for a file that [`collect_items`](super::collect_items)
/// never walked, because it is not on disk yet
/// ([`crate::validate::candidate`]). The candidate door supplies the markers
/// its own harvest emitted, so a candidate and a load agree on the list.
pub(crate) fn task_at(
    root: &Path,
    config: &Config,
    rel: &Path,
    markers: BodyMarkers<'_>,
    source: &str,
) -> Task {
    assemble_task(
        &item_at(root, config, rel),
        markers.task,
        markers.body,
        source,
        TaskCallSpans::parse(source),
    )
}

/// The [`LoadItem`] that [`collect_items`](super::collect_items) would walk a
/// file at project-relative `rel` as: the folder-derived tags and nesting depth
/// of the first configured root that contains it. A path outside every root
/// derives neither, the same tags and depth a file written outside the roots
/// would carry once it landed there, which is nothing.
pub(super) fn item_at(root: &Path, config: &Config, rel: &Path) -> LoadItem {
    let (folder_tags, depth) = derivation(config, rel);
    LoadItem {
        file: root.join(rel),
        rel: rel.to_path_buf(),
        folder_tags,
        depth,
    }
}

/// The folder-derived tags and nesting depth [`item_at`] would give a file at
/// project-relative `rel`. They come from the first configured root that
/// admits it, with the root path normalized, or neither when no root does. A
/// root admits a file under its directory that its `exclude` globs do not
/// match.
pub(crate) fn derivation(config: &Config, rel: &Path) -> (Vec<String>, u8) {
    config
        .tasks
        .roots
        .iter()
        .find_map(|spec| {
            let rel_within = spec.admits(rel)?;
            let deriver = RootDeriver::new(spec, &config.tasks.derive_tags);
            Some((
                deriver.tags_for(rel_within),
                crate::loader::derive::depth(rel_within),
            ))
        })
        .unwrap_or_default()
}

fn file_stem(file: &Path) -> String {
    file.file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}

/// Every marker the harvest emitted except the `mindtape.task` one, in document
/// order, as the [`Task::markers`] a check rule reads. The task marker itself is
/// the task, not a body marker, so it never appears here. An unlabelled marker
/// keeps `label: None`.
pub(crate) fn body_markers(harvested: &Harvest) -> Vec<BodyMarker> {
    harvested
        .markers
        .iter()
        .filter(|marker| marker.value.get("marker").and_then(HVal::as_str) != Some("mindtape.task"))
        .map(|marker| BodyMarker {
            label: marker.label.clone(),
            value: flatten_dates(marker.value.clone()),
        })
        .collect()
}

/// `value` with every [`HVal::Date`] replaced by its `YYYY-MM-DD` text.
///
/// The cache writes JSON, which has no date type, and [`HVal`] reads a
/// serialized date back as a string. A check would then see a `datetime` on a
/// cold load and a string on a warm one. The cold path flattens dates on the
/// way in, and both loads then hand the function the same tree.
fn flatten_dates(value: HVal) -> HVal {
    match value {
        HVal::Date(text) => HVal::Str(text),
        HVal::Array(items) => HVal::Array(items.into_iter().map(flatten_dates).collect()),
        HVal::Dict(map) => HVal::Dict(
            map.into_iter()
                .map(|(key, item)| (key, flatten_dates(item)))
                .collect(),
        ),
        other => other,
    }
}
