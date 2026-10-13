//! The pure, config-driven policy check on a parsed [`Recur`]: [`check_policy`].

use super::{PeriodUnit, Recur, RecurConfig, Rule};

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
