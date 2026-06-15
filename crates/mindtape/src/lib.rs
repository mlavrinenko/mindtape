//! `MindTape` domain library: a per-file, daemon-free task tracker over Typst.
//!
//! Tasks are one-per-file Typst artifacts (`#show: task.with(...)`). `MindTape`
//! discovers them under the configured roots, live-evaluates each via
//! [`typst_harvest`], projects the harvested marker into a [`Task`], and offers
//! a filter-expression query language and config-driven validation.
//!
//! The CLI is a thin wrapper over this crate; other tools (e.g. speconaut) can
//! depend on it directly.

pub mod config;
pub mod filter;
pub mod gate;
pub mod loader;
pub mod model;
pub mod validate;

pub use config::{Config, ConfigError, config_path, load_config, parse_config};
pub use filter::{Filter, FilterError};
pub use gate::{CompiledRule, GateError, check_headers, compile_rules};
pub use loader::{LoadError, LoadOutcome, find_root, load_project};
pub use model::{Status, Task, normalize_link};
pub use validate::{Location, Violation, validate};

/// Retain the tasks matching `filter`; `None` keeps them all.
///
/// # Errors
/// Returns [`FilterError`] if the filter references an unknown field.
pub fn select(tasks: Vec<Task>, filter: Option<&Filter>) -> Result<Vec<Task>, FilterError> {
    let Some(filter) = filter else {
        return Ok(tasks);
    };
    let mut kept = Vec::new();
    for task in tasks {
        if filter.matches(&task)? {
            kept.push(task);
        }
    }
    Ok(kept)
}
