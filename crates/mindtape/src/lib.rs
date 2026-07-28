//! `MindTape` domain library: a per-file, daemon-free task tracker over Typst.
//!
//! Tasks are one-per-file Typst artifacts (`#show: task.with(...)`). `MindTape`
//! discovers them under the configured roots, live-evaluates each via
//! [`typst_harvest`], projects the harvested marker into a [`Task`], and offers
//! a filter-expression query language and config-driven validation.
//!
//! The CLI is a thin wrapper over this crate; other tools (e.g. speconaut) can
//! depend on it directly.

pub mod add_slots;
mod body_headers;
pub mod canonical;
pub mod condition;
pub mod config;
mod config_add;
pub mod config_comments;
mod config_estimate;
mod config_format;
mod config_go;
pub mod config_hooks;
mod config_layers;
pub mod config_limits;
mod config_links_validate;
mod config_ls;
mod config_required;
mod config_roots;
mod config_schema;
mod config_status;
mod config_views;
mod date;
mod date_pin;
pub mod derive;
pub mod emit;
pub mod estimate_drift;
mod expr;
pub mod filter;
mod filter_links;
mod filter_views;
pub mod format;
pub mod framework;
mod framework_eval;
pub mod gate;
pub mod git;
pub mod graph;
pub mod import_map;
pub mod link;
pub mod link_scan;
pub mod loader;
mod merge;
pub mod model;
pub mod naming;
pub mod numeric;
pub mod ready;
pub mod recur;
pub mod recur_dates;
#[cfg(feature = "rrule")]
mod recur_rrule;
// Without the feature, the same module name resolves to a stub with the same
// surface — so every caller stays one always-compiled function, with no
// `#[cfg]` twin that a mutation sweep can only ever see as dead code.
#[cfg(not(feature = "rrule"))]
#[path = "recur_rrule_off.rs"]
mod recur_rrule;
pub mod sort;
pub mod status_value;
pub mod template;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod validate;
mod validate_links;
mod validate_recur;
pub mod worktrees;

pub use add_slots::{RequiredSlot, required_core_slots};
pub use canonical::{CanonicalTask, SCHEMA_VERSION};
pub use condition::{Condition, ConditionEnv, ConditionError};
pub use config::{
    AddConfig, Collector, ColumnConfig, Config, ConfigError, DeriveConfig, DeriveTags, DriftConfig,
    EstimateConfig, Facet, FormatConfig, GoConfig, HyperlinkChoice, ImportView, InteractiveMode,
    LS_COLUMNS, LocalPackage, LsRule, PriorityConfig, ReadyConfig, RootConfig, RootSpec,
    TemplatesConfig, TypstConfig, ViewSpec, Visibility, config_path, embedded_defaults,
    load_config, parse_config, parse_import_view, validate_views,
};
pub use config_comments::{CommentsConfig, CommentsError, CommentsPolicy, check_comments};
pub use config_layers::{
    LayerSource, load_config_layered, load_config_layered_from, merged_config_table,
};
pub use config_limits::{LimitBreach, LimitError, LimitSeverity, LimitSpec, check_limits};
pub use config_schema::{SCHEMA_URL, config_schema_json};
pub use date::{days_ago_iso, format_iso, parse_ymd, today_iso, today_ymd};
pub use date_pin::{DateOverrideError, init_pinned_today};
pub use derive::RootDeriver;
pub use emit::emit_task;
pub use estimate_drift::{
    ChangeSet, DriftKind, DriftWarning, InFlightBudget, check_estimate_drift, working_tree_changes,
};
pub use filter::{Filter, FilterError};
pub use format::format_typst;
pub use framework::{Category, Framework, Input, InputKind};
pub use gate::{CompiledRule, GateError, check_headers, compile_rules};
pub use git::current_branch;
pub use graph::{Cycle, Edge, edges, find_referrer, resolve_edge, toposort};
pub use import_map::{
    EntryKind, ImportReport, MapOptions, Mapped, ReportEntry, map_canonical, slug_from_title,
    stem_for_title,
};
pub use link_scan::{TargetSpan, link_target_spans};
pub use loader::{LoadError, LoadOutcome, find_root, load_project};
pub use model::{Status, Task, is_history_pointer, normalize_link};
pub use numeric::{Anchor, Band, InputValue, NumCell, NumValue, NumericField, ScaleKind};
pub use ready::{Readiness, ReadyState};
pub use sort::{default_order, default_order_with, difficulty_value, sort_by_specs};
pub use status_value::{STATUS_SHORTCUTS, escape_markup, status_value};
pub use template::{FillError, RawValue, SlotKind, SlotSpec, discover_slots, fill};
pub use validate::{Location, Violation, validate};
pub use worktrees::{WorktreeEntry, WorktreeError, WorktreeRegistry};

/// Retain the tasks matching `filter`; `None` keeps them all. `config` resolves
/// framework-scored priorities for numeric comparisons; `readiness` answers the
/// `ready` / `blocked` predicates and must be computed over the full set before
/// any narrowing (pass [`Readiness::empty`] when no set context applies).
///
/// # Errors
/// Returns [`FilterError`] if the filter references an unknown field.
pub fn select(
    tasks: Vec<Task>,
    filter: Option<&Filter>,
    config: &Config,
    readiness: &Readiness,
) -> Result<Vec<Task>, FilterError> {
    let Some(filter) = filter else {
        return Ok(tasks);
    };
    let mut kept = Vec::new();
    for task in tasks {
        if filter.matches_with(&task, config, readiness)? {
            kept.push(task);
        }
    }
    Ok(kept)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
