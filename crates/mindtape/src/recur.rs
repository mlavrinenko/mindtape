//! The `recur` field: model, `[tasks.recur]` config, parsing, and the pure
//! config-driven policy check.
//!
//! Mirrors the priority frameworks' discipline: the prelude only shapes a
//! `(rule, anchor, until, count, materialize)` dict via `every()`/`rrule()` —
//! no date math there. This module owns the marker parse and
//! [`check_policy`]; the pure next-date arithmetic lives in
//! [`crate::recur_dates`] (the same model/computation split as
//! `framework.rs`/`framework_eval.rs`). `mt recur` (`crates/mindtape-cli`) is
//! the only writer that ever acts on a [`Recur`] — see
//! `truth/tasks/recurring-tasks.typ` and `truth/tasks/command-semantics-cqrs.typ`.
//!
//! `rrule(...)` shares the same dict shape (a [`Rule::Rrule`] variant carrying
//! the raw rule string), gated by `[tasks.recur] rrule` and the `rrule`
//! cargo feature (`crates/mindtape/src/recur_rrule.rs`) — see
//! `truth/tasks/rrule-escape-hatch.typ`. `check_policy` grammar-checks the
//! raw string at parse time (feature on) and rejects `anchor: "completion"`
//! on it (date-anchored only); the expansion itself is
//! [`crate::recur_dates::advance_once`]'s concern.

use serde::Deserialize;
use typst_harvest::HVal;

/// A task's recurrence: the rule plus the policy riding along with it.
///
/// Every policy field is `None` when the task omits it — [`crate::recur_dates`]
/// resolves the effective value from `[tasks.recur]` rather than the prelude
/// baking a default in Typst.
#[derive(Debug, Clone, PartialEq)]
pub struct Recur {
    /// The period or rrule rule.
    pub rule: Rule,
    /// `"completion"` or `"due"`; `None` = the project's `default-anchor`.
    pub anchor: Option<String>,
    /// Series end date (`YYYY-MM-DD`); `None` = unbounded.
    pub until: Option<String>,
    /// Remaining occurrence budget; decremented on every `mt recur` advance
    /// (spawn's carried-forward instance, or mutate's rewritten counter).
    /// `None` = unbounded.
    pub count: Option<u32>,
    /// `"spawn"` or `"mutate"`; `None` = the project's `materialize` default.
    pub materialize: Option<String>,
}

/// The recurrence rule: a chrono-arithmetic period (optionally narrowed to a
/// weekday set) or the `rrule(...)` escape hatch.
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    /// `every(days: 3)`, `every(weeks: 1, on: (mon, wed, fri))`, …
    Period {
        /// The period unit.
        unit: PeriodUnit,
        /// The unit multiple (`weeks: 2` = every 2 weeks).
        value: u32,
        /// Non-empty when `on:` narrowed the period to specific weekdays; the
        /// next date is then the nearest matching weekday rather than a fixed
        /// `value`-unit jump. See [`crate::recur_dates::advance_once`].
        weekdays: Vec<chrono::Weekday>,
    },
    /// `rrule("FREQ=MONTHLY;BYDAY=1MO")` — parsed and gated, not yet expanded.
    Rrule {
        /// The raw rule string, verbatim.
        raw: String,
    },
}

/// One `every()` period unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodUnit {
    /// `days:`.
    Days,
    /// `weeks:`.
    Weeks,
    /// `months:`.
    Months,
    /// `years:`.
    Years,
}

impl PeriodUnit {
    /// Parse a marker unit name (`"days"`, `"weeks"`, `"months"`, `"years"`).
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "days" => Some(Self::Days),
            "weeks" => Some(Self::Weeks),
            "months" => Some(Self::Months),
            "years" => Some(Self::Years),
            _ => None,
        }
    }

    /// The config/marker unit name, singular-stripped for display by the
    /// caller when `value == 1` (mirrors the prelude's `_fmt-estimate`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Days => "days",
            Self::Weeks => "weeks",
            Self::Months => "months",
            Self::Years => "years",
        }
    }
}

/// `[tasks.recur]` — the recurring-tasks feature gate and policy floor.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RecurConfig {
    /// Whether the recurrence feature is enabled. `false` rejects any
    /// authored `recur` field outright, at both `mt check` and `mt recur`.
    pub enabled: bool,
    /// Allowed period units. Empty = any.
    pub units: Vec<String>,
    /// Allowed anchors. Empty = any.
    pub anchors: Vec<String>,
    /// Anchor used when a task's `recur` omits one.
    pub default_anchor: String,
    /// Materialize model used when a task's `recur` omits one.
    pub materialize: String,
    /// Whether the `rrule(...)` escape hatch is accepted. Off by default;
    /// also requires `MindTape` to be built with the `rrule` cargo feature —
    /// missing either gate rejects an authored `rrule(...)` with a clear
    /// error message.
    pub rrule: bool,
    /// Runaway cap: `mt recur --advance N` refuses `N` above this.
    pub max_advance: u32,
}

impl Default for RecurConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            units: vec![
                "days".to_owned(),
                "weeks".to_owned(),
                "months".to_owned(),
                "years".to_owned(),
            ],
            anchors: vec!["completion".to_owned(), "due".to_owned()],
            default_anchor: "completion".to_owned(),
            materialize: "spawn".to_owned(),
            rrule: false,
            max_advance: 52,
        }
    }
}

/// Parse the `recur` marker field into a [`Recur`], or `None` when absent or
/// malformed (an authored `recur` only ever comes from `every()`/`rrule()`,
/// which always shape it correctly; a hand-edited malformed dict is treated
/// as absent, the same tolerant style as the rest of the model) — except for
/// `on:`, whose shape is checked and reported into `errors` even when the
/// rest of the rule parses cleanly; see [`crate::model::ShapeError`].
#[must_use]
pub fn parse_recur(
    value: Option<&HVal>,
    errors: &mut Vec<crate::model::ShapeError>,
) -> Option<Recur> {
    let dict = value?.as_dict()?;
    let rule = parse_rule(dict.get("rule"), errors)?;
    Some(Recur {
        rule,
        anchor: str_field(dict, "anchor"),
        until: str_field(dict, "until"),
        count: dict.get("count").and_then(as_u32),
        materialize: str_field(dict, "materialize"),
    })
}

fn str_field(dict: &std::collections::BTreeMap<String, HVal>, key: &str) -> Option<String> {
    dict.get(key)
        .filter(|v| !v.is_null())
        .and_then(HVal::as_str)
        .map(ToOwned::to_owned)
}

fn parse_rule(value: Option<&HVal>, errors: &mut Vec<crate::model::ShapeError>) -> Option<Rule> {
    let dict = value?.as_dict()?;
    match dict.get("kind").and_then(HVal::as_str)? {
        "period" => parse_period_rule(dict, errors),
        "rrule" => Some(Rule::Rrule {
            raw: dict.get("raw").and_then(HVal::as_str)?.to_owned(),
        }),
        _ => None,
    }
}

fn parse_period_rule(
    dict: &std::collections::BTreeMap<String, HVal>,
    errors: &mut Vec<crate::model::ShapeError>,
) -> Option<Rule> {
    let unit = dict
        .get("unit")
        .and_then(HVal::as_str)
        .and_then(PeriodUnit::parse)?;
    let value = dict.get("value").and_then(as_u32)?;
    let weekdays = parse_weekdays(dict.get("on"), errors);
    Some(Rule::Period {
        unit,
        value,
        weekdays,
    })
}

/// Parse `on:`'s weekday set. Absent/`none` is a plain unnarrowed period; a
/// present value that isn't an array reports a
/// [`crate::model::ShapeError`] rather than silently narrowing to no
/// weekdays at all.
fn parse_weekdays(
    value: Option<&HVal>,
    errors: &mut Vec<crate::model::ShapeError>,
) -> Vec<chrono::Weekday> {
    let Some(value) = value.filter(|v| !v.is_null()) else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        errors.push(crate::model::shape_error(
            "recur",
            "an array for `on:`",
            value,
        ));
        return Vec::new();
    };
    items
        .iter()
        .filter_map(HVal::as_str)
        .filter_map(parse_weekday)
        .collect()
}

fn as_u32(value: &HVal) -> Option<u32> {
    if let Some(int) = value.as_i64() {
        return u32::try_from(int).ok();
    }
    // A period `value`/`count` is authored as a small non-negative literal;
    // the round-trip through `f64` (Typst's untyped-number path) is exact for
    // every realistic magnitude.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let rounded = value.as_f64().map(|num| num.round() as u32);
    rounded
}

fn parse_weekday(name: &str) -> Option<chrono::Weekday> {
    use chrono::Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
    match name {
        "mon" => Some(Mon),
        "tue" => Some(Tue),
        "wed" => Some(Wed),
        "thu" => Some(Thu),
        "fri" => Some(Fri),
        "sat" => Some(Sat),
        "sun" => Some(Sun),
        _ => None,
    }
}

/// Policy problems for `recur` against `config`: the feature gate, the
/// unit/anchor allow-lists, materialize's fixed vocabulary, the
/// weekday-set/period-unit pairing, and the `rrule` feature flag.
///
/// Pure and [`crate::model::Task`]-free — [`crate::validate_recur`] wraps each
/// string into a [`crate::validate::Violation`] for `mt check`; `mt recur`
/// calls this directly so a misconfigured task fails before any file is
/// touched.
#[must_use]
pub fn check_policy(recur: &Recur, config: &RecurConfig) -> Vec<String> {
    let mut out = Vec::new();
    if !config.enabled {
        out.push("recurrence is disabled ([tasks.recur] enabled = false)".to_owned());
        return out;
    }
    check_rule(&recur.rule, recur.anchor.as_deref(), config, &mut out);
    check_choice(recur.anchor.as_deref(), &config.anchors, "anchor", &mut out);
    check_materialize(recur.materialize.as_deref(), &mut out);
    out
}

fn check_rule(rule: &Rule, anchor: Option<&str>, config: &RecurConfig, out: &mut Vec<String>) {
    match rule {
        Rule::Period { unit, weekdays, .. } => {
            check_choice(Some(unit.as_str()), &config.units, "period unit", out);
            if !weekdays.is_empty() && !matches!(unit, PeriodUnit::Weeks) {
                out.push("an `on:` weekday set requires a weeks() period".to_owned());
            }
        }
        Rule::Rrule { raw } => check_rrule_rule(raw, anchor, config, out),
    }
}

/// The `rrule(...)` gates, in order: the config flag, the compiled-in cargo
/// feature, the date-anchored-only restriction (RFC 5545 has no
/// completion-relative primitive — `truth/tasks/rrule-escape-hatch.typ`),
/// then the rule's own grammar (parse-time, behind the feature).
fn check_rrule_rule(raw: &str, anchor: Option<&str>, config: &RecurConfig, out: &mut Vec<String>) {
    if !config.rrule {
        out.push("rrule(...) is disabled ([tasks.recur] rrule = false)".to_owned());
        return;
    }
    if !cfg!(feature = "rrule") {
        out.push(
            "rrule(...) requires mindtape to be built with the `rrule` cargo feature (not \
             compiled into this binary)"
                .to_owned(),
        );
        return;
    }
    if anchor == Some("completion") {
        out.push(
            "rrule(...) is date-anchored only: `anchor: \"completion\"` is not supported (RFC \
             5545 has no completion-relative primitive); omit `anchor` or use `anchor: \"due\"`"
                .to_owned(),
        );
    }
    check_rrule_syntax(raw, out);
}

/// Grammar-check the raw rule. `crate::recur_rrule` is the real engine with
/// the `rrule` feature on and an always-`Ok` stub without it, so this stays
/// one always-compiled function either way.
fn check_rrule_syntax(raw: &str, out: &mut Vec<String>) {
    if let Err(message) = crate::recur_rrule::validate_rule(raw) {
        out.push(message);
    }
}

fn check_choice(value: Option<&str>, allowed: &[String], label: &str, out: &mut Vec<String>) {
    let Some(value) = value else { return };
    if !allowed.is_empty() && !allowed.iter().any(|a| a == value) {
        out.push(format!("{label} `{value}` is not allowed by [tasks.recur]"));
    }
}

fn check_materialize(value: Option<&str>, out: &mut Vec<String>) {
    if let Some(value) = value
        && !matches!(value, "spawn" | "mutate")
    {
        out.push(format!("materialize `{value}` must be `spawn` or `mutate`"));
    }
}

#[cfg(test)]
#[path = "recur_tests.rs"]
mod tests;
