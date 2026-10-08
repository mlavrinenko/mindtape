//! Date helpers: today's date, relative thresholds for the staleness
//! predicate, and a configurable display formatter.
//!
//! "Today" is the machine's LOCAL calendar date, not UTC (see
//! `truth/tasks/decide-whether-today-is-utc-or-local.typ`): a task filed at
//! 01:00 on the 29th is stamped the 29th, whatever the offset. Only
//! [`local_epoch_days`] knows about zones — everything downstream is a plain
//! `YYYY-MM-DD` civil date with no offset attached, and every comparison is
//! date-to-date within that one calendar.
//!
//! "Today" is pinnable for reproducible output (see
//! `truth/tasks/pinnable-clock-for-reproducible-output.typ`):
//! [`crate::init_pinned_today`] (re-exported at the crate root, backed by
//! `crate::date_pin`) resolves `$MINDTAPE_NOW` once per process and caches the
//! result; `today_epoch_days`, below, is the single point every helper here
//! reads through — no other function in this module touches the clock directly.
//!
//! # Files
//!
//! - `pin.rs`: the env-driven override for "today", resolved once per process.

pub(crate) mod pin;

use chrono::Datelike as _;

/// Today's local date as `YYYY-MM-DD`.
#[must_use]
pub fn today_iso() -> String {
    let (year, month, day) = today_ymd();
    format!("{year:04}-{month:02}-{day:02}")
}

/// Today's local date as `(year, month, day)`.
#[must_use]
pub fn today_ymd() -> (i32, u8, u8) {
    epoch_days_to_ymd(today_epoch_days())
}

/// The date `count` days before today as `YYYY-MM-DD`.
#[must_use]
pub fn days_ago_iso(count: u64) -> String {
    let (year, month, day) = epoch_days_to_ymd(today_epoch_days().saturating_sub(count));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Today, as days since the Unix epoch: the process's pinned override if
/// [`crate::date::pin::init_pinned_today`] resolved one, else the system clock
/// read in the machine's local timezone. The only place in this module that
/// reads a clock.
fn today_epoch_days() -> u64 {
    if let Some(days) = crate::date::pin::pinned_epoch_days() {
        return days;
    }
    local_epoch_days(&chrono::Local::now())
}

/// Days since the Unix epoch for the calendar date `at` falls on *in its own
/// timezone* — the local date, not the UTC one, whenever `at` carries a local
/// offset.
///
/// Generic over the zone (rather than taking `DateTime<Local>`) so the
/// offset-awareness can be asserted against fixed offsets in a unit test:
/// mutating `$TZ` in-process is neither safe nor parallel-safe, and reading
/// the real clock twice — once inside the conversion, once outside — races a
/// midnight rollover between the two reads.
fn local_epoch_days<Tz: chrono::TimeZone>(at: &chrono::DateTime<Tz>) -> u64 {
    let date = at.date_naive();
    let month = u8::try_from(date.month()).unwrap_or(1);
    let day = u8::try_from(date.day()).unwrap_or(1);
    u64::try_from(days_from_civil(date.year(), month, day)).unwrap_or(0)
}

/// Parse a `YYYY-MM-DD` string into `(year, month, day)`; `None` if malformed.
#[must_use]
pub fn parse_ymd(date: &str) -> Option<(i32, u8, u8)> {
    let mut parts = date.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((year, month, day))
}

/// One `{date:FMT}` token: what it is written as, what the docs say it
/// expands to, and how it expands a `(year, month, day)`.
#[derive(Debug, Clone, Copy)]
pub struct DateToken {
    /// The token as written in a format string, such as `YYYY`.
    pub token: &'static str,
    /// A short description for the docs, such as `four-digit year`.
    pub expands_to: &'static str,
    expand: fn(i32, u8, u8) -> String,
}

impl DateToken {
    /// The token's text for `(year, month, day)`.
    #[must_use]
    pub fn render(&self, year: i32, month: u8, day: u8) -> String {
        (self.expand)(year, month, day)
    }
}

/// The tokens `format_iso` reads, longest first so that `MMMM` is not
/// consumed by `MM`. `format_iso` matches against this table and the docs
/// render it, so a token and its description cannot disagree.
pub const DATE_TOKENS: [DateToken; 8] = [
    DateToken {
        token: "YYYY",
        expands_to: "four-digit year",
        expand: |y, _, _| format!("{y:04}"),
    },
    DateToken {
        token: "YY",
        expands_to: "two-digit year (last two digits)",
        expand: |y, _, _| format!("{:02}", y.unsigned_abs() % 100),
    },
    DateToken {
        token: "MMMM",
        expands_to: "full month name (January to December)",
        expand: |_, m, _| month_name_full(m).to_owned(),
    },
    DateToken {
        token: "MMM",
        expands_to: "abbreviated month name (Jan to Dec)",
        expand: |_, m, _| month_name_abbr(m).to_owned(),
    },
    DateToken {
        token: "MM",
        expands_to: "zero-padded month (01 to 12)",
        expand: |_, m, _| format!("{m:02}"),
    },
    DateToken {
        token: "M",
        expands_to: "bare month (1 to 12)",
        expand: |_, m, _| m.to_string(),
    },
    DateToken {
        token: "DD",
        expands_to: "zero-padded day (01 to 31)",
        expand: |_, _, d| format!("{d:02}"),
    },
    DateToken {
        token: "D",
        expands_to: "bare day (1 to 31)",
        expand: |_, _, d| d.to_string(),
    },
];

/// Format a `YYYY-MM-DD` ISO date string using a display format template.
///
/// Tokens are the entries of [`DATE_TOKENS`], matched in table order.
/// Unrecognised characters pass through literally. Returns the input string
/// unchanged when it cannot be parsed as `YYYY-MM-DD`.
#[must_use]
pub fn format_iso(iso: &str, fmt: &str) -> String {
    let Some((year, month, day)) = parse_ymd(iso) else {
        return iso.to_owned();
    };
    let mut out = String::with_capacity(fmt.len());
    let mut rest = fmt;
    while let Some(ch) = rest.chars().next() {
        if let Some((tok, tail)) = DATE_TOKENS
            .iter()
            .find_map(|t| rest.strip_prefix(t.token).map(|tail| (t, tail)))
        {
            out.push_str(&tok.render(year, month, day));
            rest = tail;
        } else {
            out.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
    }
    out
}

const MONTH_NAMES_FULL: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const MONTH_NAMES_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn month_name_full(month: u8) -> &'static str {
    MONTH_NAMES_FULL
        .get(usize::from(month).saturating_sub(1))
        .copied()
        .unwrap_or("???")
}

fn month_name_abbr(month: u8) -> &'static str {
    MONTH_NAMES_ABBR
        .get(usize::from(month).saturating_sub(1))
        .copied()
        .unwrap_or("???")
}

/// Convert days since Unix epoch (1970-01-01) to Gregorian `(year, month, day)`.
///
/// Uses Howard Hinnant's civil calendar algorithm, minus its negative-`shifted`
/// branch: `epoch_days` is unsigned, so `shifted` is at least `719_468` and the
/// era division never needs the round-towards-negative-infinity correction.
/// <https://howardhinnant.github.io/date_algorithms.html>
fn epoch_days_to_ymd(epoch_days: u64) -> (i32, u8, u8) {
    let shifted = i64::try_from(epoch_days).unwrap_or(0) + 719_468;
    let era = shifted / 146_097;
    let day_in_era = u64::try_from(shifted - era * 146_097).unwrap_or(0);
    let year_in_era =
        (day_in_era - day_in_era / 1460 + day_in_era / 36524 - day_in_era / 146_096) / 365;
    let base_year = i64::try_from(year_in_era).unwrap_or(0) + era * 400;
    let day_in_year = day_in_era - (365 * year_in_era + year_in_era / 4 - year_in_era / 100);
    let month_idx = (5 * day_in_year + 2) / 153;
    let day = u8::try_from(day_in_year - (153 * month_idx + 2) / 5 + 1).unwrap_or(1);
    let month = u8::try_from(if month_idx < 10 {
        month_idx + 3
    } else {
        month_idx - 9
    })
    .unwrap_or(1);
    let full_year = if u64::from(month) <= 2 {
        base_year + 1
    } else {
        base_year
    };
    (i32::try_from(full_year).unwrap_or(2000), month, day)
}

/// Convert Gregorian `(year, month, day)` to days since the Unix epoch
/// (1970-01-01). The inverse of [`epoch_days_to_ymd`] (same Howard Hinnant
/// civil calendar algorithm and source). Negative for a date before
/// 1970-01-01; [`crate::date_pin`] clamps that to epoch 0 rather than
/// wrapping, since nothing in this domain predates it.
pub(crate) fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let year_of_era = u64::try_from(y - era * 400).unwrap_or(0); // [0, 399]
    let m = u64::from(month);
    let shifted_month = if m > 2 { m - 3 } else { m + 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + u64::from(day) - 1; // [0, 365]
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + i64::try_from(day_of_era).unwrap_or(0) - 719_468
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::DATE_TOKENS;

    use super::{
        days_ago_iso, days_from_civil, epoch_days_to_ymd, format_iso, local_epoch_days,
        today_epoch_days, today_iso, today_ymd,
    };

    /// 2026-07-28T22:00:00Z, the instant the deciding task was filed at: still
    /// the 28th in UTC, already the 29th three hours east.
    fn late_july_evening_utc() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc
            .with_ymd_and_hms(2026, 7, 28, 22, 0, 0)
            .single()
            .expect("22:00 UTC on 2026-07-28 is unambiguous")
    }

    /// The offset decides the calendar date, not the underlying instant: one
    /// moment, read three hours east and five hours west, is two different
    /// days. This is the assertion that pins "local, not UTC" — under the old
    /// UTC conversion both sides would answer the 28th.
    #[test]
    fn local_epoch_days_reads_the_offsets_own_calendar_date() {
        let instant = late_july_evening_utc();
        let east = chrono::FixedOffset::east_opt(3 * 3600).expect("+03:00 is a valid offset");
        let west = chrono::FixedOffset::west_opt(5 * 3600).expect("-05:00 is a valid offset");

        assert_eq!(
            local_epoch_days(&instant.with_timezone(&east)),
            u64::try_from(days_from_civil(2026, 7, 29)).expect("after the epoch")
        );
        assert_eq!(
            local_epoch_days(&instant.with_timezone(&west)),
            u64::try_from(days_from_civil(2026, 7, 28)).expect("after the epoch")
        );
    }

    /// UTC is just the zero-offset case of the same conversion.
    #[test]
    fn local_epoch_days_at_utc_is_the_utc_date() {
        assert_eq!(
            local_epoch_days(&late_july_evening_utc()),
            u64::try_from(days_from_civil(2026, 7, 28)).expect("after the epoch")
        );
    }

    /// The clock fallback: no `MINDTAPE_NOW` pin is set in this process (the
    /// pin is exercised out-of-process — see `date/pin_tests.rs` and
    /// `crates/mindtape-cli/tests/date_pin_e2e.rs`), so `today_epoch_days`
    /// must report the system clock. Bracketing it between two samples taken
    /// around the call is rollover-safe: if midnight falls inside the window,
    /// the answer is still one of the two ends.
    #[test]
    fn today_epoch_days_falls_back_to_the_system_clock() {
        let before = local_epoch_days(&chrono::Local::now());
        let days = today_epoch_days();
        let after = local_epoch_days(&chrono::Local::now());
        assert!(
            (before..=after).contains(&days),
            "today_epoch_days() = {days} outside the sampled window {before}..={after}"
        );
    }

    #[test]
    fn epoch_zero_is_1970_01_01() {
        assert_eq!(epoch_days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn known_date_2026_06_18() {
        // 2026-06-18 = day 20622 since Unix epoch
        assert_eq!(epoch_days_to_ymd(20_622), (2026, 6, 18));
    }

    #[test]
    fn days_from_civil_is_the_inverse_of_epoch_days_to_ymd() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2026, 6, 18), 20_622);
    }

    #[test]
    fn days_from_civil_before_epoch_is_negative() {
        assert!(days_from_civil(1969, 12, 31) < 0);
    }

    #[test]
    fn today_iso_looks_like_a_date() {
        let date = today_iso();
        let parts: Vec<&str> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(date.len(), 10);
    }

    #[test]
    fn today_ymd_roundtrips_to_iso() {
        let (year, month, day) = today_ymd();
        let iso = format!("{year:04}-{month:02}-{day:02}");
        assert_eq!(iso, today_iso());
    }

    #[test]
    fn days_ago_iso_is_before_today() {
        let ago = days_ago_iso(1);
        let today = today_iso();
        assert!(ago < today, "{ago} should be before {today}");
    }

    #[test]
    fn days_ago_iso_zero_equals_today() {
        assert_eq!(days_ago_iso(0), today_iso());
    }

    #[test]
    fn days_ago_saturates_at_epoch() {
        // Far in the past: should not panic
        let _ = days_ago_iso(u64::MAX);
    }

    #[test]
    fn every_token_in_the_table_is_read_by_format_iso() {
        for t in DATE_TOKENS {
            let out = format_iso("2026-03-07", t.token);
            assert_ne!(out, t.token, "{} passed through literally", t.token);
            assert_eq!(out, t.render(2026, 3, 7));
        }
    }

    #[test]
    fn tokens_are_ordered_longest_first() {
        for (i, a) in DATE_TOKENS.iter().enumerate() {
            for b in DATE_TOKENS.iter().skip(i + 1) {
                assert!(
                    !b.token.starts_with(a.token) || a.token == b.token,
                    "{} shadows {}",
                    a.token,
                    b.token
                );
            }
        }
    }

    #[test]
    fn format_iso_default_reproduces_iso() {
        assert_eq!(format_iso("2026-06-27", "YYYY-MM-DD"), "2026-06-27");
    }

    #[test]
    fn format_iso_reordered_dd_mm_yyyy() {
        assert_eq!(format_iso("2026-06-27", "DD.MM.YYYY"), "27.06.2026");
    }

    #[test]
    fn format_iso_abbreviated_month() {
        assert_eq!(format_iso("2026-06-27", "DD MMM YYYY"), "27 Jun 2026");
    }

    #[test]
    fn format_iso_full_month_name() {
        assert_eq!(format_iso("2026-06-27", "MMMM DD, YYYY"), "June 27, 2026");
    }

    #[test]
    fn format_iso_two_digit_year() {
        assert_eq!(format_iso("2026-06-27", "DD/MM/YY"), "27/06/26");
    }

    #[test]
    fn format_iso_bare_month_and_day() {
        assert_eq!(format_iso("2026-06-07", "M/D/YYYY"), "6/7/2026");
    }

    #[test]
    fn format_iso_invalid_input_passes_through() {
        assert_eq!(format_iso("not-a-date", "DD.MM.YYYY"), "not-a-date");
    }

    #[test]
    fn format_iso_mmmm_not_eaten_by_mm() {
        // MMMM must match before MM
        assert_eq!(format_iso("2026-01-15", "MMMM"), "January");
    }

    #[test]
    fn epoch_days_to_ymd_last_day_of_era() {
        // day_in_era = 146096 exercises -day_in_era/146096 (col 62 -/+)
        // and month_idx = 11 (col 173 - vs /).
        assert_eq!(epoch_days_to_ymd(11_016), (2000, 2, 29));
    }

    #[test]
    fn epoch_days_to_ymd_era_boundary_operators() {
        // day_in_era = 36524 exercises +day_in_era/36524 (col 41 +/- and +/*)
        // and -day_in_era/146096 rewritten as / (col 62 - vs /) — the
        // mutated operator changes year_in_era from 100 to 99.
        assert_eq!(epoch_days_to_ymd(47_541), (2100, 3, 1));
    }

    #[test]
    fn days_from_civil_before_year_1() {
        // Exercises the y < 0 branch in era calculation (col 191 - vs +, - vs /).
        assert_eq!(days_from_civil(0, 1, 1), -719_528);
    }

    #[test]
    fn days_from_civil_february() {
        // Month = 2 exercises the m > 2 vs m >= 2 boundary (col 194).
        // The >= mutant misroutes month=2 into the m - 3 arm, producing
        // u64 underflow (panic or wrapped garbage).
        assert_eq!(days_from_civil(1970, 2, 1), 31);
    }
}
