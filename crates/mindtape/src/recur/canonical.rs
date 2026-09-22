//! `recur`'s canonical task-JSON shape — the marker dict verbatim (`rule.kind`,
//! `on[i]`, …), so a hand-edited canonical record reads the same as the
//! `every()`/`rrule()` call it came from. Split out from `recur.rs` to stay
//! under the file-size gate (`crate::emit`'s `format`/`links` split is the
//! same reasoning). See `crate::canonical`'s field contract.

use serde::{Deserialize, Serialize};

use super::{PeriodUnit, Recur, Rule, parse_weekday, weekday_name};

/// `recur:` in the canonical schema. See [`Recur::to_canonical`] and
/// [`CanonicalRecur::to_recur`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalRecur {
    /// The period or rrule rule.
    pub rule: CanonicalRule,
    /// `"completion"` or `"due"`; `None` = the project's `default-anchor`.
    pub anchor: Option<String>,
    /// Series end date (`YYYY-MM-DD`); `None` = unbounded.
    pub until: Option<String>,
    /// Remaining occurrence budget; `None` = unbounded.
    pub count: Option<u32>,
    /// `"spawn"` or `"mutate"`; `None` = the project's `materialize` default.
    pub materialize: Option<String>,
}

/// `recur.rule` in the canonical schema: a period or the `rrule(...)` raw
/// string, tagged by `kind` exactly as the marker dict is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CanonicalRule {
    /// `every(days: 3)`, `every(weeks: 1, on: (mon, wed, fri))`, …
    Period {
        /// `"days"`, `"weeks"`, `"months"`, or `"years"`.
        unit: String,
        /// The unit multiple.
        value: u32,
        /// Weekday names (`"mon"`, …); empty when unnarrowed.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        on: Vec<String>,
    },
    /// `rrule("FREQ=MONTHLY;BYDAY=1MO")`.
    Rrule {
        /// The raw rule string, verbatim.
        raw: String,
    },
}

impl Recur {
    /// Project into the canonical schema.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalRecur {
        CanonicalRecur {
            rule: match &self.rule {
                Rule::Period {
                    unit,
                    value,
                    weekdays,
                } => CanonicalRule::Period {
                    unit: unit.as_str().to_owned(),
                    value: *value,
                    on: weekdays
                        .iter()
                        .copied()
                        .map(|day| weekday_name(day).to_owned())
                        .collect(),
                },
                Rule::Rrule { raw } => CanonicalRule::Rrule { raw: raw.clone() },
            },
            anchor: self.anchor.clone(),
            until: self.until.clone(),
            count: self.count,
            materialize: self.materialize.clone(),
        }
    }
}

impl CanonicalRecur {
    /// Reverse into a [`Recur`]. An unrecognized `on:` weekday name is
    /// dropped rather than failing the whole record; an unrecognized `unit`
    /// falls back to [`PeriodUnit::Days`] — a hand-edited canonical record is
    /// the only way to reach either arm (this crate's own
    /// [`Recur::to_canonical`] never authors one), and `mt check`/`mt
    /// recur`'s config-driven [`super::check_policy`] is what reports a
    /// meaningful problem once the record lands as a task.
    #[must_use]
    pub fn to_recur(&self) -> Recur {
        Recur {
            rule: match &self.rule {
                CanonicalRule::Period { unit, value, on } => Rule::Period {
                    unit: PeriodUnit::parse(unit).unwrap_or(PeriodUnit::Days),
                    value: *value,
                    weekdays: on.iter().filter_map(|name| parse_weekday(name)).collect(),
                },
                CanonicalRule::Rrule { raw } => Rule::Rrule { raw: raw.clone() },
            },
            anchor: self.anchor.clone(),
            until: self.until.clone(),
            count: self.count,
            materialize: self.materialize.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_round_trips_through_canonical() {
        // Values deliberately differ from `canonical_tests.rs`'s
        // `every_recur_serializes_and_round_trips` (same property, one level
        // up the stack) so `check-dry` sees two fixtures, not one copy-pasted
        // twice.
        let recur = Recur {
            rule: Rule::Period {
                unit: PeriodUnit::Months,
                value: 3,
                weekdays: vec![],
            },
            anchor: Some("completion".to_owned()),
            until: None,
            count: Some(1),
            materialize: Some("spawn".to_owned()),
        };
        let canonical = recur.to_canonical();
        assert_eq!(canonical.to_recur(), recur);
    }

    #[test]
    fn rrule_round_trips_through_canonical() {
        let recur = Recur {
            rule: Rule::Rrule {
                raw: "FREQ=WEEKLY;BYDAY=FR".to_owned(),
            },
            anchor: Some("due".to_owned()),
            until: None,
            count: None,
            materialize: None,
        };
        let canonical = recur.to_canonical();
        assert_eq!(canonical.to_recur(), recur);
    }

    #[test]
    fn unrecognized_weekday_is_dropped_not_rejected() {
        let canonical = CanonicalRecur {
            rule: CanonicalRule::Period {
                unit: "weeks".to_owned(),
                value: 1,
                on: vec!["mon".to_owned(), "bogus".to_owned()],
            },
            anchor: None,
            until: None,
            count: None,
            materialize: None,
        };
        assert_eq!(
            canonical.to_recur().rule,
            Rule::Period {
                unit: PeriodUnit::Weeks,
                value: 1,
                weekdays: vec![chrono::Weekday::Mon],
            }
        );
    }

    #[test]
    fn unrecognized_unit_falls_back_to_days() {
        let canonical = CanonicalRecur {
            rule: CanonicalRule::Period {
                unit: "fortnights".to_owned(),
                value: 2,
                on: vec![],
            },
            anchor: None,
            until: None,
            count: None,
            materialize: None,
        };
        assert_eq!(
            canonical.to_recur().rule,
            Rule::Period {
                unit: PeriodUnit::Days,
                value: 2,
                weekdays: vec![],
            }
        );
    }
}
