//! The config-aware middle layer between the canonical task-JSON schema
//! (`canonical.rs`) and a project's `.mindtape/config.toml` policy —
//! `truth/tasks/canonical-json-to-task-mapping.typ`.
//!
//! The jq adapter upstream has already done the *structural* map (foreign
//! tracker → canonical, config-blind); [`map_canonical`] does the *policy*
//! map (canonical → valid-for-this-project), the only part that needs the
//! loaded [`Config`]. Pure over `(&CanonicalTask, &Config, &MapOptions)`: no
//! IO, no source writing. `mt import` (a later CLI task) deserializes JSON
//! into [`CanonicalTask`] via its existing `serde::Deserialize`, calls this,
//! then hands the resulting [`Task`] to `emit::emit_task`.
//!
//! Every policy check reports a [`ReportEntry`] rather than panicking or
//! silently discarding data — an out-of-legend status, an unknown priority
//! framework, an out-of-allow-list tag, still land on the mapped [`Task`]
//! (or, for extras violating the `x-<ns>` discipline, are dropped and
//! reported) so the CLI can decide fail vs skip vs continue per record.

mod body;
mod enum_fields;
mod extras;
mod priority;
mod report;
mod status;
mod stem;

use crate::canonical::CanonicalTask;
use crate::config::Config;
use crate::link::Link;
use crate::model::Task;

pub use crate::canonical::json_to_hval;
pub use report::{EntryKind, ImportReport, MapOptions, ReportEntry};
pub use stem::{slug_from_title, stem_for_title};

/// The result of mapping one canonical record.
#[derive(Debug, Clone)]
pub struct Mapped {
    /// The reconciled task. `id`/`file` are placeholders — the canonical
    /// record's own `id` and an empty path — since this layer never decides
    /// a destination file; the CLI assigns the real stem (see
    /// [`stem_for_title`]) and path before writing.
    pub task: Task,
    /// What happened while mapping: defaults applied, values dropped or
    /// remapped, and policy violations.
    pub report: ImportReport,
}

/// Reverse-project a canonical task-JSON record onto a [`Task`], reconciling
/// it against `config`'s policy (status legend, priority/difficulty scale
/// and framework strictness, `kind` enum, tag allow-list, the `x-<ns>`
/// extras discipline, and the body-header "every file passes `mt check`"
/// guarantee). See the module doc for the overall contract.
#[must_use]
pub fn map_canonical(canonical: &CanonicalTask, config: &Config, opts: &MapOptions) -> Mapped {
    let mut entries = Vec::new();

    let status = status::map_status(canonical.status.as_ref(), config, opts, &mut entries);
    let priority = priority::map_priority(canonical.priority.as_ref(), config, &mut entries);
    let difficulty = priority::map_difficulty(canonical.difficulty.as_ref(), config, &mut entries);
    enum_fields::check_kind(canonical.kind.as_deref(), config, &mut entries);
    let tags = enum_fields::map_tags(canonical.tags.clone(), config, opts, &mut entries);
    let body = body::map_body(canonical, config, &mut entries);
    let (extras, carried_extras) = extras::map_extras(&canonical.extras, &mut entries);
    let links = canonical.links.iter().map(map_link).collect();

    let task = Task {
        id: canonical.id.clone(),
        file: std::path::PathBuf::new(),
        title: canonical.title.clone(),
        status,
        priority,
        difficulty,
        kind: canonical.kind.clone(),
        estimate: canonical.estimate.clone(),
        links,
        tags,
        derived_tags: Vec::new(),
        due: canonical.due.clone(),
        start: canonical.start.clone(),
        reviewed: canonical.reviewed.clone(),
        recur: None,
        extras,
        body,
    };

    Mapped {
        task,
        report: ImportReport {
            entries,
            carried_extras,
        },
    }
}

fn map_link(link: &crate::canonical::CanonicalLink) -> Link {
    Link {
        task: link.task.clone(),
        kind: link.kind.clone(),
        note: link.note.clone(),
    }
}

#[cfg(test)]
#[path = "import_map_tests.rs"]
mod tests;
