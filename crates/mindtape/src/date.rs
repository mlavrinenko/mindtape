//! Date helpers using only `std`: today's date, relative thresholds for the
//! staleness predicate, and a configurable display formatter. No external
//! crates; UTC only (close enough for backlog-rot detection).
//!
//! "Today" is pinnable for reproducible output (see
//! `truth/tasks/pinnable-clock-for-reproducible-output.typ`):
//! [`crate::init_pinned_today`] (re-exported at the crate root, backed by
//! `crate::date_pin`) resolves `$MINDTAPE_NOW` once per process and caches the
//! result; `today_epoch_days`, below, is the single point every helper here
//! reads through — no other function in this module touches the clock directly.

/// Today's date as `YYYY-MM-DD` (UTC).
#[must_use]
pub fn today_iso() -> String {
    let (year, month, day) = today_ymd();
    format!("{year:04}-{month:02}-{day:02}")
}

/// Today's date as `(year, month, day)` (UTC).
#[must_use]
pub fn today_ymd() -> (i32, u8, u8) {
    epoch_days_to_ymd(today_epoch_days())
}

/// The date `count` days before today as `YYYY-MM-DD` (UTC).
#[must_use]
pub fn days_ago_iso(count: u64) -> String {
    let (year, month, day) = epoch_days_to_ymd(today_epoch_days().saturating_sub(count));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Today, as days since the Unix epoch: the process's pinned override if
/// [`crate::date_pin::init_pinned_today`] resolved one, else the system
/// clock. The only place in this module that reads a clock.
fn today_epoch_days() -> u64 {
    if let Some(days) = crate::date_pin::pinned_epoch_days() {
        return days;
    }
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400
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

/// Format a `YYYY-MM-DD` ISO date string using a display format template.
///
/// Tokens are substituted longest-first so that e.g. `MMMM` is not consumed
/// by `MM`. Unrecognised characters pass through literally.
///
/// | Token | Output |
/// |-------|--------|
/// | `YYYY` | 4-digit year |
/// | `YY`   | 2-digit year (last two digits) |
/// | `MMMM` | full month name (January … December) |
/// | `MMM`  | abbreviated month (Jan … Dec) |
/// | `MM`   | zero-padded month (01 … 12) |
/// | `M`    | bare month (1 … 12) |
/// | `DD`   | zero-padded day (01 … 31) |
/// | `D`    | bare day (1 … 31) |
///
/// Returns the input string unchanged when it cannot be parsed as `YYYY-MM-DD`.
#[must_use]
pub fn format_iso(iso: &str, fmt: &str) -> String {
    let Some((year, month, day)) = parse_ymd(iso) else {
        return iso.to_owned();
    };
    let mut out = String::with_capacity(fmt.len());
    let mut rest = fmt;
    while !rest.is_empty() {
        if let Some(tail) = rest.strip_prefix("YYYY") {
            out.push_str(&format!("{year:04}"));
            rest = tail;
        } else if let Some(tail) = rest.strip_prefix("YY") {
            out.push_str(&format!("{:02}", year.unsigned_abs() % 100));
            rest = tail;
        } else if let Some(tail) = rest.strip_prefix("MMMM") {
            out.push_str(month_name_full(month));
            rest = tail;
        } else if let Some(tail) = rest.strip_prefix("MMM") {
            out.push_str(month_name_abbr(month));
            rest = tail;
        } else if let Some(tail) = rest.strip_prefix("MM") {
            out.push_str(&format!("{month:02}"));
            rest = tail;
        } else if let Some(tail) = rest.strip_prefix('M') {
            out.push_str(&format!("{month}"));
            rest = tail;
        } else if let Some(tail) = rest.strip_prefix("DD") {
            out.push_str(&format!("{day:02}"));
            rest = tail;
        } else if let Some(tail) = rest.strip_prefix('D') {
            out.push_str(&format!("{day}"));
            rest = tail;
        } else {
            let mut chars = rest.chars();
            if let Some(ch) = chars.next() {
                out.push(ch);
                rest = chars.as_str();
            }
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
/// Uses Howard Hinnant's civil calendar algorithm.
/// <https://howardhinnant.github.io/date_algorithms.html>
fn epoch_days_to_ymd(epoch_days: u64) -> (i32, u8, u8) {
    let shifted = i64::try_from(epoch_days).unwrap_or(0) + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
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
    use super::{
        days_ago_iso, days_from_civil, epoch_days_to_ymd, format_iso, today_iso, today_ymd,
    };

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
}
