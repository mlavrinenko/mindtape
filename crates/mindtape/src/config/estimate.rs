//! `[tasks.estimate]`: the unit allow-list and whether a task needs an
//! estimate.
//!
//! An estimate is an opaque `unit -> quantity` dict (such as
//! `(days: 5, hours: 5)` or `(src-files: 3)`). Units are free labels, never
//! converted or compared across names. `units` only constrains which names are
//! legal.
//!
//! The rules own every quantity check. A cap on a declared estimate, or a
//! budget the git working tree must meet, is a `[[tasks.rules]]` entry reading
//! `estimate.<unit>` and `changed(...)` in the filter language (see
//! [`crate::rules`]). An estimate check runs only when a project wrote or
//! inherited one.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::Requirement;

/// The legal estimate units, and whether a task must carry one.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct EstimateConfig {
    /// Allowed unit names. Empty = any unit.
    pub units: Vec<String>,
    /// Whether a task must carry an estimate.
    #[schemars(extend("x-expression" = "filter"))]
    pub required: Requirement,
    /// The estimate `mt check` files a spliced slot with, as `unit = quantity`
    /// pairs. A required estimate with no `example` leaves that slot
    /// unfillable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<BTreeMap<String, f64>>,
    /// The units that take `count(n, ..queries)`, each naming the path set,
    /// a glob or an `@name` set, its matched files must fall in. A unit
    /// missing here takes only a plain number.
    // How a count resolves: `crate::estimate_queries`.
    pub queries: BTreeMap<String, String>,
}

#[cfg(test)]
#[path = "estimate_tests.rs"]
mod tests;
