//! `MindTape` domain library: a per-file, daemon-free task tracker over Typst.
//!
//! Tasks are one-per-file Typst artifacts (`#show: task.with(...)`). `MindTape`
//! discovers them under the configured roots, live-evaluates each via
//! [`typst_harvest`], projects the harvested marker into a [`Task`], and offers
//! a filter-expression query language and config-driven validation.
//!
//! The CLI is a thin wrapper over this crate; other tools (e.g. speconaut) can
//! depend on it directly.

mod body_headers;
pub mod config;
mod date;
pub mod filter;
pub mod framework;
mod framework_eval;
pub mod gate;
pub mod graph;
pub mod loader;
pub mod model;
pub mod numeric;
pub mod ready;
pub mod sort;
pub mod validate;

pub use config::{Config, ConfigError, PriorityConfig, config_path, load_config, parse_config};
pub use date::{days_ago_iso, today_iso, today_ymd};
pub use filter::{Filter, FilterError};
pub use framework::{Category, Framework, Input, InputKind};
pub use gate::{CompiledRule, GateError, check_headers, compile_rules};
pub use graph::{Cycle, toposort};
pub use loader::{LoadError, LoadOutcome, find_root, load_project};
pub use model::{Status, Task, is_history_pointer, normalize_link};
pub use numeric::{Anchor, Band, InputValue, NumCell, NumValue, NumericField, ScaleKind};
pub use ready::{Readiness, ReadyState};
pub use sort::{default_order, default_order_with, difficulty_value, sort_by_specs};
pub use validate::{Location, Violation, validate};

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
