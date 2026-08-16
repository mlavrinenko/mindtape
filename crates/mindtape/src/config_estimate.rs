//! `[tasks.estimate]` — the unit allow-list, per-unit bounds, and the
//! working-tree drift policy.
//!
//! An estimate is an opaque `unit -> quantity` dict (e.g. `(days: 5, hours: 5)`
//! or `(files: 3)`). Units are free labels, never converted or compared across
//! names; `units` only constrains which names are legal.
//!
//! On top of that allow-list sit two per-unit tables, `max` and `min`
//! (`[tasks.estimate.max] files = 5`). A quantity outside the declared range is
//! a *validation error*, not a warning: the point of a bounded estimate is that
//! the task is split **before** the work starts, and a check that only complains
//! afterwards has already lost. Terminal tasks are exempt, for the same reason
//! `required = "!terminal"` relaxes there — a finished task cannot be split, and
//! re-litigating its estimate would only freeze the backlog.
//!
//! `[tasks.estimate.drift]` configures the complementary run-time check
//! ([`crate::estimate_drift`]): while the work runs, `mt check` compares the git
//! working tree against the budget the in-flight tasks declare and warns — never
//! errors — when it has outgrown it.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::config::Requirement;
use crate::numeric::format_num;

/// Estimate unit the drift check reads as a file budget when
/// `[tasks.estimate.drift] unit` is unset. A file count is the unit the check
/// can actually observe: it is plannable up front and checkable against git
/// while the work runs, unlike a predicted duration.
const DEFAULT_DRIFT_UNIT: &str = "files";

/// Filter naming the tasks the drift check treats as in flight, when
/// `[tasks.estimate.drift] in-flight` is unset. Matches the shipped legend's
/// working state; a project that names its in-flight status differently (or has
/// several) overrides it with any filter expression.
const DEFAULT_IN_FLIGHT: &str = "status == \"wip\"";

/// Lines allowed per declared file before the secondary soft line warning
/// fires. Deliberately generous — a line total is distorted by generated files,
/// lockfiles and formatting passes, so this is a smoke alarm for a diff that has
/// clearly stopped matching its task, never a budget of its own. `0` disables it.
const DEFAULT_LINE_MULTIPLE: usize = 200;

/// Advice reported with a drift warning that declares no `message`.
///
/// The message *is* the intervention, so it names the action rather than
/// restating the breach the report already prints: a warning that only says the
/// working tree is too big tells an agent nothing about what to do next.
pub(crate) const DEFAULT_DRIFT_ADVICE: &str =
    "split the remaining work into a new task and finish the part in hand";

/// `[tasks.estimate]` — estimate units, bounds, and drift policy.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct EstimateConfig {
    /// Allowed unit names. Empty = any unit.
    pub units: Vec<String>,
    /// Whether an estimate must be set.
    pub required: Requirement,
    /// Per-unit upper bounds (`[tasks.estimate.max] files = 5`). A unit with no
    /// entry is unbounded above.
    pub max: BTreeMap<String, f64>,
    /// Per-unit lower bounds (`[tasks.estimate.min] files = 1`). A unit with no
    /// entry is unbounded below.
    pub min: BTreeMap<String, f64>,
    /// Working-tree drift policy for `mt check`.
    pub drift: DriftConfig,
}

impl EstimateConfig {
    /// Bound violations for one authored estimate, one message per breached
    /// bound, in unit order. Empty when every declared quantity sits inside its
    /// unit's range — or names a unit no bound covers.
    #[must_use]
    pub fn bound_problems(&self, estimate: &BTreeMap<String, f64>) -> Vec<String> {
        let mut out = Vec::new();
        for (unit, quantity) in estimate {
            if let Some(max) = self.max.get(unit)
                && quantity > max
            {
                out.push(over_message(unit, *quantity, *max));
            }
            if let Some(min) = self.min.get(unit)
                && quantity < min
            {
                out.push(under_message(unit, *quantity, *min));
            }
        }
        out
    }
}

/// Message for an estimate over its unit's `max`. Names the split, because that
/// is the only response: the task is too big to start as written.
fn over_message(unit: &str, quantity: f64, max: f64) -> String {
    format!(
        "{} {unit} is over the {} allowed; split the task before starting the work",
        format_num(quantity),
        format_num(max),
    )
}

/// Message for an estimate under its unit's `min`.
fn under_message(unit: &str, quantity: f64, min: f64) -> String {
    format!(
        "{} {unit} is under the {} required; fold it into a larger task",
        format_num(quantity),
        format_num(min),
    )
}

/// `[tasks.estimate.drift]` — how `mt check` compares the working tree against
/// the budget the in-flight tasks declare.
///
/// Inert until some in-flight task actually declares an estimate in `unit`, so
/// a project that never uses a file budget never sees the check.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DriftConfig {
    /// Estimate unit read as the file budget. Default `files`.
    pub unit: String,
    /// Filter expression naming the tasks in flight. The working tree carries
    /// one diff and nothing in it says which file belongs to which task, so this
    /// is what attribution rests on. Default `status == "wip"`.
    pub in_flight: String,
    /// Replaces the default advice in a drift report.
    pub message: Option<String>,
    /// Lines allowed per declared file before the secondary soft line warning.
    /// `0` disables it. Default 200.
    pub line_multiple: usize,
    /// Glob patterns naming paths the file count never counts — generated
    /// output nobody hand-edits, so touching it should never cost part of a
    /// budget spent on the source that produced it. Matched against the path
    /// git reports, repo-root-relative (the same root `mt` runs against, not
    /// the task root), with the same glob syntax as `[tasks.roots]
    /// derive-tags`: `*` matches within one path segment, `**` crosses
    /// directory boundaries. Empty = nothing excluded.
    pub exclude: Vec<String>,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            unit: DEFAULT_DRIFT_UNIT.to_owned(),
            in_flight: DEFAULT_IN_FLIGHT.to_owned(),
            message: None,
            line_multiple: DEFAULT_LINE_MULTIPLE,
            exclude: Vec::new(),
        }
    }
}

impl DriftConfig {
    /// The advice a drift report carries: the configured `message`, else
    /// [`DEFAULT_DRIFT_ADVICE`].
    #[must_use]
    pub fn advice(&self) -> &str {
        self.message.as_deref().unwrap_or(DEFAULT_DRIFT_ADVICE)
    }
}

#[cfg(test)]
#[path = "config_estimate_tests.rs"]
mod tests;
