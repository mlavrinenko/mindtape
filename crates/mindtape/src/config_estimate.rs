//! `[tasks.estimate]` — the unit allow-list and whether an estimate is
//! required.
//!
//! An estimate is an opaque `unit -> quantity` dict (e.g. `(days: 5, hours: 5)`
//! or `(src-files: 3)`). Units are free labels, never converted or compared
//! across names; `units` only constrains which names are legal.
//!
//! Nothing here checks a quantity. A cap on a declared estimate, or a budget
//! the git working tree is held to, is a `[[tasks.rules]]` entry reading
//! `estimate.<unit>` and `changed(...)` in the filter language (see
//! [`crate::config_rules`]), so no estimate check runs unless a project wrote
//! or inherited one.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::config::Requirement;

/// `[tasks.estimate]` — the legal estimate units and whether one is required.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct EstimateConfig {
    /// Allowed unit names. Empty = any unit.
    pub units: Vec<String>,
    /// Whether an estimate must be set.
    pub required: Requirement,
    /// The estimate `mt check` files a spliced slot with, as `unit = quantity`
    /// pairs. A required estimate with no `example` leaves that slot
    /// unfillable.
    pub example: Option<BTreeMap<String, f64>>,
}

#[cfg(test)]
#[path = "config_estimate_tests.rs"]
mod tests;
