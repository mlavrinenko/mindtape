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
pub mod config;
mod config_add;
mod config_go;
mod config_layers;
mod config_ls;
mod config_roots;
mod config_status;
mod config_views;
mod date;
pub mod derive;
pub mod filter;
mod filter_links;
mod filter_views;
pub mod framework;
mod framework_eval;
pub mod gate;
pub mod graph;
pub mod link;
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
pub use config::{
    AddConfig, Collector, ColumnConfig, Config, ConfigError, DeriveConfig, DeriveTags, GoConfig,
    InteractiveMode, LS_COLUMNS, LocalPackage, PriorityConfig, ReadyConfig, RootConfig, RootSpec,
    TemplatesConfig, TypstConfig, ViewSpec, Visibility, config_path, embedded_defaults,
    load_config, parse_config, validate_views,
};
pub use config_layers::{LayerSource, load_config_layered, merged_config_table};
pub use date::{days_ago_iso, format_iso, parse_ymd, today_iso, today_ymd};
pub use derive::RootDeriver;
pub use filter::{Filter, FilterError};
pub use framework::{Category, Framework, Input, InputKind};
pub use gate::{CompiledRule, GateError, check_headers, compile_rules};
pub use graph::{Cycle, find_referrer, resolve_edge, toposort};
pub use loader::{LoadError, LoadOutcome, find_root, load_project};
pub use model::{Status, Task, is_history_pointer, normalize_link};
pub use numeric::{Anchor, Band, InputValue, NumCell, NumValue, NumericField, ScaleKind};
pub use ready::{Readiness, ReadyState};
pub use sort::{default_order, default_order_with, difficulty_value, sort_by_specs};
pub use status_value::{STATUS_SHORTCUTS, status_value};
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
