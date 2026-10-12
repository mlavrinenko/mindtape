//! The `recur` field: model, `[tasks.recur]` config, parsing, and the pure
//! config-driven policy check.
//!
//! Like the priority frameworks, the prelude only shapes a
//! `(rule, anchor, until, count, materialize)` dict via `every()`/`rrule()`,
//! with no date math. This module owns the marker parse and [`check_policy`];
//! the next-date arithmetic lives in [`crate::recur::dates`]. Only `mt recur`
//! acts on a [`Recur`]; see `truth/tasks/recurring-tasks.typ`.
//!
//! `rrule(...)` shares the dict shape (a [`Rule::Rrule`] variant carrying the
//! raw rule string), gated by `[tasks.recur] rrule`; see
//! `truth/tasks/rrule-escape-hatch.typ`. `check_policy` grammar-checks the raw
//! string at parse time and rejects `anchor: "completion"` on it, since it is
//! date-anchored only.
//!
//! # Files
//!
//! - `canonical.rs`: the canonical task-JSON shape, [`CanonicalRecur`] and [`CanonicalRule`].
//! - `dates.rs`: pure next-occurrence date arithmetic.
//! - `rrule.rs`: the `rrule(...)` expansion engine.

pub mod dates;
pub(crate) mod rrule;

mod canonical;

use serde::Deserialize;
use typst_harvest::HVal;

pub use canonical::{CanonicalRecur, CanonicalRule};

/// A task's recurrence: the rule plus the policy riding along with it.
///
/// Every policy field is `None` when the task omits it — [`crate::recur::dates`]
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
        /// `value`-unit jump. See [`crate::recur::dates::advance_once`].
        weekdays: Vec<chrono::Weekday>,
        /// `start:` — a date whose Monday-to-Sunday week is an on-week of a
        /// multi-week weekday series; every `value`-th week after it is too.
        /// `None` = the global series epoch
        /// ([`crate::recur::dates`]'s `SERIES_EPOCH`).
        start: Option<chrono::NaiveDate>,
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

/// `[tasks.recur]` — whether recurring tasks are enabled, and the values a
/// `recur` field may take.
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
    /// Whether a `recur` field may use `rrule(...)`. Off by default. Without
    /// it, `mt` rejects an `rrule(...)` with an error.
    pub rrule: bool,
    /// Largest `N` that `mt recur --advance N` accepts.
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

/// What a `recur:` field accepts, for the reports below.
const RECUR_EXPECTED: &str = "a recurrence like `every(weeks: 2)`";
/// What one entry of a period's `on:` set accepts.
const WEEKDAY_EXPECTED: &str = "a weekday name (mon, tue, wed, thu, fri, sat, sun)";

/// Parse the `recur` marker field into a [`Recur`], or `None` when absent.
///
/// An authored `recur` only ever comes from `every()`/`rrule()`, which shape
/// the dict correctly — so anything this cannot read is a value the author
/// passed through one of those helpers (`every(weeks: "two")`, an `on:` entry
/// that is not a weekday) or a hand-edited dict. Either way it is reported
/// into `errors` as a [`crate::model::marker::ShapeError`] rather than read as no
/// recurrence at all. Reports name the AUTHORED position — `recur.on[1]`,
/// `recur.weeks` — since that is what the author can count to in their own
/// source; only a shape no helper can produce is named by its marker path
/// (`recur.rule`).
#[must_use]
pub fn parse_recur(
    value: Option<&HVal>,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) -> Option<Recur> {
    let value = value?;
    let Some(dict) = value.as_dict() else {
        errors.push(crate::model::marker::shape_error(
            "recur",
            RECUR_EXPECTED,
            value,
        ));
        return None;
    };
    let rule = parse_rule(dict, errors)?;
    Some(Recur {
        rule,
        anchor: crate::model::marker::dict_str(dict, "recur", "anchor", errors),
        until: crate::model::marker::dict_str(dict, "recur", "until", errors),
        count: parse_count(dict, errors),
        materialize: crate::model::marker::dict_str(dict, "recur", "materialize", errors),
    })
}

fn parse_rule(
    outer: &std::collections::BTreeMap<String, HVal>,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) -> Option<Rule> {
    let Some(value) = outer.get("rule").filter(|v| !v.is_null()) else {
        errors.push(crate::model::marker::shape_message(
            "recur",
            &format!("expected {RECUR_EXPECTED}, found one with no `rule`"),
        ));
        return None;
    };
    let Some(dict) = value.as_dict() else {
        errors.push(crate::model::marker::shape_error(
            "recur.rule",
            "a dict",
            value,
        ));
        return None;
    };
    match crate::model::marker::dict_str_required(dict, "recur.rule", "kind", errors)?.as_str() {
        "period" => parse_period_rule(dict, errors),
        "rrule" => Some(Rule::Rrule {
            raw: crate::model::marker::dict_str_required(dict, "recur.rule", "raw", errors)?,
        }),
        other => {
            errors.push(crate::model::marker::shape_message(
                "recur.rule.kind",
                &format!("expected `period` or `rrule`, found `{other}`"),
            ));
            None
        }
    }
}

fn parse_period_rule(
    dict: &std::collections::BTreeMap<String, HVal>,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) -> Option<Rule> {
    let name = crate::model::marker::dict_str_required(dict, "recur.rule", "unit", errors)?;
    let Some(unit) = PeriodUnit::parse(&name) else {
        errors.push(crate::model::marker::shape_message(
            "recur.rule.unit",
            &format!("expected one of days, weeks, months, years, found `{name}`"),
        ));
        return None;
    };
    let value = period_value(dict, &name, errors)?;
    let weekdays = parse_weekdays(dict.get("on"), errors);
    let start = parse_start(dict, errors);
    Some(Rule::Period {
        unit,
        value,
        weekdays,
        start,
    })
}

/// A period's `start:` date. One that is not a date reports against the
/// argument, `recur.start`, rather than reading as no start at all.
fn parse_start(
    dict: &std::collections::BTreeMap<String, HVal>,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) -> Option<chrono::NaiveDate> {
    let text = crate::model::marker::dict_str(dict, "recur", "start", errors)?;
    let date = crate::recur::dates::parse_iso(&text);
    if date.is_none() {
        errors.push(crate::model::marker::shape_message(
            "recur.start",
            &format!("expected a date like `on(2026, 6, 1)`, found `{text}`"),
        ));
    }
    date
}

/// A period's length, reported against the argument that carried it —
/// `every(weeks: "two")` reports `recur.weeks`, not the marker's `rule.value`.
fn period_value(
    dict: &std::collections::BTreeMap<String, HVal>,
    unit_name: &str,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) -> Option<u32> {
    let path = format!("recur.{unit_name}");
    let Some(value) = dict.get("value").filter(|v| !v.is_null()) else {
        errors.push(crate::model::marker::shape_message(
            &path,
            "expected a whole number, found none",
        ));
        return None;
    };
    let Some(num) = as_u32(value) else {
        errors.push(crate::model::marker::shape_error(
            &path,
            "a whole number",
            value,
        ));
        return None;
    };
    Some(num)
}

fn parse_count(
    dict: &std::collections::BTreeMap<String, HVal>,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) -> Option<u32> {
    let value = dict.get("count").filter(|v| !v.is_null())?;
    let Some(num) = as_u32(value) else {
        errors.push(crate::model::marker::shape_error(
            "recur.count",
            "a whole number",
            value,
        ));
        return None;
    };
    Some(num)
}

/// Parse `on:`'s weekday set. Absent/`none` is a plain unnarrowed period; a
/// present value that isn't an array, and every entry that isn't a weekday
/// name, report a [`crate::model::marker::ShapeError`] naming their position rather
/// than silently narrowing the period to fewer days than the author wrote.
fn parse_weekdays(
    value: Option<&HVal>,
    errors: &mut Vec<crate::model::marker::ShapeError>,
) -> Vec<chrono::Weekday> {
    let Some(value) = value.filter(|v| !v.is_null()) else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        errors.push(crate::model::marker::shape_error(
            "recur.on", "an array", value,
        ));
        return Vec::new();
    };
    let mut out = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let path = format!("recur.on[{index}]");
        let Some(name) = item.as_str() else {
            errors.push(crate::model::marker::shape_error(
                &path,
                WEEKDAY_EXPECTED,
                item,
            ));
            continue;
        };
        if let Some(day) = parse_weekday(name) {
            out.push(day);
        } else {
            errors.push(crate::model::marker::shape_message(
                &path,
                &format!("expected {WEEKDAY_EXPECTED}, found `{name}`"),
            ));
        }
    }
    out
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

/// The inverse of [`parse_weekday`] — `crate::emit`'s `on:` rendering shares
/// this rather than keeping its own copy of the name table.
pub(crate) fn weekday_name(day: chrono::Weekday) -> &'static str {
    use chrono::Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
    match day {
        Mon => "mon",
        Tue => "tue",
        Wed => "wed",
        Thu => "thu",
        Fri => "fri",
        Sat => "sat",
        Sun => "sun",
    }
}

/// Policy problems for `recur` against `config`: the feature gate, the
/// unit/anchor allow-lists, materialize's fixed vocabulary, the
/// weekday-set/period-unit pairing, and the `rrule` gate and grammar.
///
/// Pure and [`crate::model::Task`]-free — [`crate::validate::recur`] wraps each
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
        Rule::Period {
            unit,
            weekdays,
            start,
            ..
        } => {
            check_choice(Some(unit.as_str()), &config.units, "period unit", out);
            if !weekdays.is_empty() && !matches!(unit, PeriodUnit::Weeks) {
                out.push("an `on:` weekday set requires a weeks() period".to_owned());
            }
            if start.is_some() && weekdays.is_empty() {
                out.push("`start:` requires a weeks() period with an `on:` weekday set".to_owned());
            }
        }
        Rule::Rrule { raw } => check_rrule_rule(raw, anchor, config, out),
    }
}

/// The `rrule(...)` gates, in order: the config flag, the date-anchored-only
/// restriction (RFC 5545 has no completion-relative primitive —
/// `truth/tasks/rrule-escape-hatch.typ`), then the rule's own grammar
/// (parse-time).
fn check_rrule_rule(raw: &str, anchor: Option<&str>, config: &RecurConfig, out: &mut Vec<String>) {
    if !config.rrule {
        out.push("rrule(...) is disabled ([tasks.recur] rrule = false)".to_owned());
        return;
    }
    if anchor == Some("completion") {
        out.push(
            "rrule(...) is date-anchored only: `anchor: \"completion\"` is not supported (RFC \
             5545 has no completion-relative primitive); omit `anchor` or use `anchor: \"due\"`"
                .to_owned(),
        );
    }
    if let Err(message) = crate::recur::rrule::validate_rule(raw) {
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
mod tests;

#[cfg(test)]
mod tests_shape;
