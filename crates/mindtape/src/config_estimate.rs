//! `[tasks.estimate]` — the unit allow-list, per-unit bounds, and the
//! working-tree drift policy.
//!
//! An estimate is an opaque `unit -> quantity` dict (e.g. `(days: 5, hours: 5)`
//! or `(files: 3)`). Units are free labels, never converted or compared across
//! names; `units` only constrains which names are legal.
//!
//! On top of that allow-list sit two lists of per-unit bounds, `max` and `min`.
//! A quantity outside the declared range is a *validation error*, not a warning:
//! the point of a bounded estimate is that the task is split **before** the work
//! starts, and a check that only complains afterwards has already lost. Terminal
//! tasks are exempt, for the same reason `required = "!terminal"` relaxes there —
//! a finished task cannot be split, and re-litigating its estimate would only
//! freeze the backlog.
//!
//! Each entry carries an optional `when` filter and `message`, so a ceiling can
//! bind on judgment rather than on size alone:
//!
//! ```toml
//! [[tasks.estimate.max]]
//! when = "difficulty >= 3"
//! files = 5
//! message = "judgment work stays small; split it rather than widen the cap"
//!
//! [[tasks.estimate.max]]
//! files = 15
//! ```
//!
//! Resolution is per unit: a unit's bound is the first entry, in config order,
//! whose `when` matches the task AND that names the unit. An entry silent on a
//! unit leaves it to the entries below, so a gated `files` ceiling never
//! unbounds an `hours` one the fallback declares. The single-table spelling
//! (`[tasks.estimate.max] files = 5`) still parses, as one unconditional entry.
//!
//! `[tasks.estimate.drift]` configures the complementary run-time check
//! ([`crate::estimate_drift`]): while the work runs, `mt check` compares the git
//! working tree against the budget the in-flight tasks declare and warns — never
//! errors — when it has outgrown it.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::config::{Config, Requirement};
use crate::filter::Filter;
use crate::model::Task;
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
    /// Upper bounds (`[[tasks.estimate.max]] files = 5`), first matching entry
    /// per unit wins. A unit no matching entry names is unbounded above. A
    /// single table is accepted as one unconditional entry.
    #[serde(deserialize_with = "one_or_many")]
    #[schemars(schema_with = "crate::config_schema::estimate_bounds_schema")]
    pub max: Vec<EstimateBound>,
    /// Lower bounds (`[[tasks.estimate.min]] files = 1`), resolved like `max`.
    /// A unit no matching entry names is unbounded below.
    #[serde(deserialize_with = "one_or_many")]
    #[schemars(schema_with = "crate::config_schema::estimate_bounds_schema")]
    pub min: Vec<EstimateBound>,
    /// Working-tree drift policy for `mt check`.
    pub drift: DriftConfig,
}

/// One `[[tasks.estimate.max]]` / `[[tasks.estimate.min]]` entry: per-unit
/// quantities, gated by `when` and explained by `message`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct EstimateBound {
    /// Filter expression selecting the tasks this entry bounds. Absent means
    /// every task.
    #[serde(default)]
    pub when: Option<String>,
    /// Why the bound sits where it does; replaces the default advice in a
    /// breach report.
    #[serde(default)]
    pub message: Option<String>,
    /// The bound per unit name (`files = 5`).
    #[serde(flatten)]
    pub units: BTreeMap<String, f64>,
}

/// Accept `max`/`min` as an array of tables or, for the single-table spelling,
/// one table read as a one-entry list. A visitor rather than an untagged enum,
/// so a wrong-typed quantity reports its own error instead of "did not match
/// any variant".
fn one_or_many<'de, D>(deserializer: D) -> Result<Vec<EstimateBound>, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoundsVisitor;
    impl<'de> serde::de::Visitor<'de> for BoundsVisitor {
        type Value = Vec<EstimateBound>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a table of per-unit bounds, or an array of them")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            let one =
                EstimateBound::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
            Ok(vec![one])
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
            Vec::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
        }
    }
    deserializer.deserialize_any(BoundsVisitor)
}

/// Which side of the range an entry list bounds.
#[derive(Clone, Copy)]
enum Side {
    Max,
    Min,
}

/// One entry with its `when` parsed.
struct CompiledBound<'a> {
    spec: &'a EstimateBound,
    /// `None` when the entry has no `when`; `Some(None)` when it has one that
    /// failed to parse — see [`EstimateBounds::compile`].
    when: Option<Option<Filter>>,
}

impl CompiledBound<'_> {
    /// Whether this entry applies to `task`. A `when` that cannot be parsed or
    /// evaluated excludes the task, the reading `[[tasks.rules]] when` takes: a
    /// gate that cannot say whether it applies has not said that it does.
    fn applies(&self, task: &Task, config: &Config) -> bool {
        match &self.when {
            None => true,
            Some(None) => false,
            Some(Some(filter)) => filter.matches(task, config).unwrap_or(false),
        }
    }
}

/// `[tasks.estimate]`'s `max` and `min` entries, compiled once for a
/// validation pass.
pub struct EstimateBounds<'a> {
    max: Vec<CompiledBound<'a>>,
    min: Vec<CompiledBound<'a>>,
}

impl<'a> EstimateBounds<'a> {
    /// Parse every entry's `when`. A malformed one is rejected at config load
    /// ([`validate_bounds`]), so a parse failure here only covers a hand-built
    /// config and drops that entry.
    #[must_use]
    pub fn compile(estimate: &'a EstimateConfig) -> Self {
        let compile = |entries: &'a [EstimateBound]| {
            entries
                .iter()
                .map(|spec| CompiledBound {
                    spec,
                    when: spec.when.as_deref().map(|src| Filter::parse(src).ok()),
                })
                .collect()
        };
        Self {
            max: compile(&estimate.max),
            min: compile(&estimate.min),
        }
    }

    /// Bound violations for `task`'s authored estimate, one message per
    /// breached bound, in unit order. Empty when every declared quantity sits
    /// inside its unit's range — or names a unit no matching entry covers.
    #[must_use]
    pub fn problems(&self, task: &Task, config: &Config) -> Vec<String> {
        let mut out = Vec::new();
        for (unit, &quantity) in &task.estimate {
            for (side, entries) in [(Side::Max, &self.max), (Side::Min, &self.min)] {
                let Some((spec, bound)) = entries.iter().find_map(|entry| {
                    let bound = entry.spec.units.get(unit)?;
                    entry.applies(task, config).then_some((entry.spec, *bound))
                }) else {
                    continue;
                };
                let breached = match side {
                    Side::Max => quantity > bound,
                    Side::Min => quantity < bound,
                };
                if breached {
                    out.push(breach_message(side, unit, quantity, bound, spec));
                }
            }
        }
        out
    }
}

/// Message for a breached bound: the quantity against the bound, the entry's
/// `when` when it has one — so the reader sees why this ceiling and not
/// another applies — and the entry's `message`, else the default action. The
/// default names the move rather than the breach: over is a split, under a fold.
fn breach_message(
    side: Side,
    unit: &str,
    quantity: f64,
    bound: f64,
    spec: &EstimateBound,
) -> String {
    let (relation, verb, default_advice) = match side {
        Side::Max => ("over", "allowed", "split the task before starting the work"),
        Side::Min => ("under", "required", "fold it into a larger task"),
    };
    let gate = spec
        .when
        .as_deref()
        .map(|when| format!(" when `{when}`"))
        .unwrap_or_default();
    format!(
        "{} {unit} is {relation} the {} {verb}{gate}; {}",
        format_num(quantity),
        format_num(bound),
        spec.message.as_deref().unwrap_or(default_advice),
    )
}

/// Check every `max`/`min` entry at config load, returning one message per
/// problem, prefixed with its dotted TOML path: a `when` that does not parse
/// or dry-run, an entry naming no unit (it bounds nothing), and a unit outside
/// a non-empty `units` allow-list (no task may carry it, so it never fires).
pub(crate) fn validate_bounds(config: &Config) -> Vec<String> {
    let estimate = &config.tasks.estimate;
    let probe = Task::default();
    let mut problems = Vec::new();
    for (key, entries) in [("max", &estimate.max), ("min", &estimate.min)] {
        for (index, entry) in entries.iter().enumerate() {
            let path = format!("tasks.estimate.{key}[{index}]");
            if let Some(when) = &entry.when
                && let Some(problem) = crate::filter::expr_problem(when, &probe, config)
            {
                problems.push(format!("{path}.when: {problem}"));
            }
            if entry.units.is_empty() {
                problems.push(format!(
                    "{path}: bounds no unit; name at least one, e.g. `files = 5`"
                ));
            }
            for unit in entry.units.keys() {
                if !estimate.units.is_empty() && !estimate.units.contains(unit) {
                    problems.push(format!(
                        "{path}.{unit}: `{unit}` is not an allowed estimate unit"
                    ));
                }
            }
        }
    }
    problems
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
