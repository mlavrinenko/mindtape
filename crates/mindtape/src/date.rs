//! Date helpers using only `std`: today's date and relative thresholds for the
//! staleness predicate. No external crates; UTC only (close enough for
//! backlog-rot detection).

/// Today's date as `YYYY-MM-DD` (UTC).
#[must_use]
pub fn today_iso() -> String {
    let (year, month, day) = today_ymd();
    format!("{year:04}-{month:02}-{day:02}")
}

/// Today's date as `(year, month, day)` (UTC).
#[must_use]
pub fn today_ymd() -> (i32, u8, u8) {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    epoch_days_to_ymd(secs / 86400)
}

/// The date `count` days before today as `YYYY-MM-DD` (UTC).
#[must_use]
pub fn days_ago_iso(count: u64) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (year, month, day) = epoch_days_to_ymd((secs / 86400).saturating_sub(count));
    format!("{year:04}-{month:02}-{day:02}")
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

#[cfg(test)]
mod tests {
    use super::{days_ago_iso, epoch_days_to_ymd, today_iso, today_ymd};

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
}
