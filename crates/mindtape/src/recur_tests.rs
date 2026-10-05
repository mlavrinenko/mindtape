#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use chrono::Weekday;

use super::*;

pub(super) fn dict(pairs: Vec<(&str, HVal)>) -> HVal {
    HVal::Dict(pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}

pub(super) fn period_rule(unit: &str, value: i64, on: Option<HVal>) -> HVal {
    let mut fields = vec![
        ("kind", HVal::Str("period".to_owned())),
        ("unit", HVal::Str(unit.to_owned())),
        ("value", HVal::Int(value)),
    ];
    fields.push(("on", on.unwrap_or(HVal::Null)));
    dict(fields)
}

pub(super) fn every_val(rule: HVal, policy: Vec<(&str, HVal)>) -> HVal {
    let mut fields = vec![("rule", rule)];
    for (k, v) in policy {
        fields.push((k, v));
    }
    dict(fields)
}

/// [`parse_recur`] without caring about shape errors — most tests here feed
/// it a well-shaped dict; shape-error tests call `parse_recur` directly.
pub(super) fn parse(value: Option<&HVal>) -> Option<Recur> {
    parse_recur(value, &mut Vec::new())
}

#[test]
fn parse_recur_absent_is_none() {
    assert_eq!(parse(None), None);
}

#[test]
fn parse_recur_simple_weekly() {
    let val = every_val(period_rule("weeks", 1, None), vec![]);
    let recur = parse(Some(&val)).unwrap();
    assert_eq!(
        recur.rule,
        Rule::Period {
            unit: PeriodUnit::Weeks,
            value: 1,
            weekdays: vec![],
            start: None,
        }
    );
    assert_eq!(recur.anchor, None);
    assert_eq!(recur.until, None);
    assert_eq!(recur.count, None);
    assert_eq!(recur.materialize, None);
}

#[test]
fn parse_recur_weekday_set() {
    let on = HVal::Array(vec![
        HVal::Str("mon".to_owned()),
        HVal::Str("wed".to_owned()),
        HVal::Str("fri".to_owned()),
    ]);
    let val = every_val(period_rule("weeks", 1, Some(on)), vec![]);
    let recur = parse(Some(&val)).unwrap();
    let Rule::Period { weekdays, .. } = recur.rule else {
        panic!("expected a period rule");
    };
    assert_eq!(weekdays, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
}

#[test]
fn parse_recur_full_policy() {
    let val = every_val(
        period_rule("weeks", 1, None),
        vec![
            ("anchor", HVal::Str("due".to_owned())),
            ("until", HVal::Date("2027-01-01".to_owned())),
            ("count", HVal::Int(10)),
            ("materialize", HVal::Str("mutate".to_owned())),
        ],
    );
    let recur = parse(Some(&val)).unwrap();
    assert_eq!(recur.anchor.as_deref(), Some("due"));
    assert_eq!(recur.until.as_deref(), Some("2027-01-01"));
    assert_eq!(recur.count, Some(10));
    assert_eq!(recur.materialize.as_deref(), Some("mutate"));
}

#[test]
fn parse_recur_rrule_rule() {
    let rule = dict(vec![
        ("kind", HVal::Str("rrule".to_owned())),
        ("raw", HVal::Str("FREQ=MONTHLY;BYDAY=1MO".to_owned())),
    ]);
    let val = every_val(rule, vec![]);
    let recur = parse(Some(&val)).unwrap();
    assert_eq!(
        recur.rule,
        Rule::Rrule {
            raw: "FREQ=MONTHLY;BYDAY=1MO".to_owned(),
        }
    );
}

#[test]
fn parse_weekday_roundtrip_all() {
    use chrono::Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
    for (name, expected) in [
        ("mon", Mon),
        ("tue", Tue),
        ("wed", Wed),
        ("thu", Thu),
        ("fri", Fri),
        ("sat", Sat),
        ("sun", Sun),
    ] {
        assert_eq!(parse_weekday(name), Some(expected));
    }
}

fn recur(rule: Rule) -> Recur {
    Recur {
        rule,
        anchor: None,
        until: None,
        count: None,
        materialize: None,
    }
}

fn weekly() -> Rule {
    Rule::Period {
        unit: PeriodUnit::Weeks,
        value: 1,
        weekdays: vec![],
        start: None,
    }
}

#[test]
fn check_policy_disabled_reports_one_problem() {
    let config = RecurConfig {
        enabled: false,
        ..RecurConfig::default()
    };
    let problems = check_policy(&recur(weekly()), &config);
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("disabled"));
}

#[test]
fn check_policy_default_config_is_clean() {
    assert!(check_policy(&recur(weekly()), &RecurConfig::default()).is_empty());
}

#[test]
fn check_policy_rejects_unit_outside_allow_list() {
    let config = RecurConfig {
        units: vec!["days".to_owned()],
        ..RecurConfig::default()
    };
    let problems = check_policy(&recur(weekly()), &config);
    assert!(problems.iter().any(|p| p.contains("period unit")));
}

#[test]
fn check_policy_rejects_anchor_outside_allow_list() {
    let mut r = recur(weekly());
    r.anchor = Some("due".to_owned());
    let config = RecurConfig {
        anchors: vec!["completion".to_owned()],
        ..RecurConfig::default()
    };
    let problems = check_policy(&r, &config);
    assert!(problems.iter().any(|p| p.contains("anchor")));
}

#[test]
fn check_policy_rejects_unknown_materialize() {
    let mut r = recur(weekly());
    r.materialize = Some("bogus".to_owned());
    let problems = check_policy(&r, &RecurConfig::default());
    assert!(problems.iter().any(|p| p.contains("materialize")));
}

#[test]
fn check_policy_rejects_weekday_set_on_non_weekly_unit() {
    let rule = Rule::Period {
        unit: PeriodUnit::Days,
        value: 1,
        weekdays: vec![Weekday::Mon],
        start: None,
    };
    let problems = check_policy(&recur(rule), &RecurConfig::default());
    assert!(problems.iter().any(|p| p.contains("weeks()")));
}

#[test]
fn check_policy_rejects_start_without_a_weekday_set() {
    let rule = Rule::Period {
        unit: PeriodUnit::Weeks,
        value: 2,
        weekdays: vec![],
        start: chrono::NaiveDate::from_ymd_opt(2026, 6, 1),
    };
    let problems = check_policy(&recur(rule), &RecurConfig::default());
    assert!(problems.iter().any(|p| p.contains("`start:` requires")));
}

#[test]
fn check_policy_rejects_rrule_when_config_flag_off() {
    let r = recur(Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    });
    let problems = check_policy(&r, &RecurConfig::default());
    assert!(problems.iter().any(|p| p.contains("rrule")));
}

// The cargo-feature-absent and cargo-feature-present paths through
// `check_rrule_rule` are mutually exclusive at compile time — `cfg(feature =
// "rrule")` picks which of the next two tests exists in a given build.

#[cfg(not(feature = "rrule"))]
#[test]
fn check_policy_rejects_rrule_when_cargo_feature_absent() {
    let r = recur(Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    });
    let config = RecurConfig {
        rrule: true,
        ..RecurConfig::default()
    };
    let problems = check_policy(&r, &config);
    assert!(problems.iter().any(|p| p.contains("cargo feature")));
}

#[cfg(feature = "rrule")]
#[test]
fn check_policy_allows_rrule_when_feature_on() {
    let r = recur(Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    });
    let config = RecurConfig {
        rrule: true,
        ..RecurConfig::default()
    };
    assert!(check_policy(&r, &config).is_empty());
}

#[cfg(feature = "rrule")]
#[test]
fn check_policy_rejects_rrule_with_completion_anchor() {
    let mut r = recur(Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    });
    r.anchor = Some("completion".to_owned());
    let config = RecurConfig {
        rrule: true,
        ..RecurConfig::default()
    };
    let problems = check_policy(&r, &config);
    assert!(problems.iter().any(|p| p.contains("date-anchored only")));
}

#[cfg(feature = "rrule")]
#[test]
fn check_policy_allows_rrule_with_explicit_due_anchor() {
    let mut r = recur(Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    });
    r.anchor = Some("due".to_owned());
    let config = RecurConfig {
        rrule: true,
        ..RecurConfig::default()
    };
    assert!(check_policy(&r, &config).is_empty());
}

#[cfg(feature = "rrule")]
#[test]
fn check_policy_rejects_malformed_rrule_quoting_it() {
    let r = recur(Rule::Rrule {
        raw: "FREQ=NOPE".to_owned(),
    });
    let config = RecurConfig {
        rrule: true,
        ..RecurConfig::default()
    };
    let problems = check_policy(&r, &config);
    assert!(problems.iter().any(|p| p.contains("invalid rrule")));
    assert!(problems.iter().any(|p| p.contains("\"FREQ=NOPE\"")));
}

#[test]
fn period_unit_as_str_roundtrips_parse() {
    for unit in [
        PeriodUnit::Days,
        PeriodUnit::Weeks,
        PeriodUnit::Months,
        PeriodUnit::Years,
    ] {
        assert_eq!(PeriodUnit::parse(unit.as_str()), Some(unit));
    }
}

#[test]
fn recur_config_default_is_permissive_but_on() {
    let config = RecurConfig::default();
    assert!(config.enabled);
    assert!(!config.rrule);
    assert_eq!(config.default_anchor, "completion");
    assert_eq!(config.materialize, "spawn");
}
