//! Width-aware string helpers using Unicode display width.
//!
//! A double-width character (e.g. CJK ideographs, many emoji) counts as
//! width 2 in a fixed-width terminal cell grid. All functions here operate on
//! *display* width rather than byte or `char` count.

use unicode_width::UnicodeWidthChar;

const ELLIPSIS: char = '…';
const ELLIPSIS_WIDTH: usize = 1;

/// Compute the display width of `s`.
///
/// Each `char` contributes its Unicode display width (0, 1, or 2 cells).
#[must_use]
pub fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
        .sum()
}

/// Truncate `s` to at most `max_width` display cells, appending `…` if truncated.
///
/// The ellipsis occupies 1 display cell. The function never splits a
/// multi-cell grapheme — it stops at the last char that fits whole.
///
/// # Panics
/// Does not panic; returns an empty string when `max_width` is 0.
#[must_use]
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    if display_width(s) <= max_width {
        return s.to_owned();
    }
    if max_width == 0 {
        return String::new();
    }
    // Reserve 1 cell for the ellipsis.
    let budget = max_width.saturating_sub(ELLIPSIS_WIDTH);
    let mut out = String::new();
    let mut used = 0_usize;
    for ch in s.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > budget {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push(ELLIPSIS);
    out
}

/// Pad `s` on the right with spaces until its display width reaches `width`.
///
/// If `s` already meets or exceeds `width`, it is returned unchanged.
#[must_use]
pub fn pad_to_width(s: &str, width: usize) -> String {
    let current = display_width(s);
    if current >= width {
        return s.to_owned();
    }
    let mut out = s.to_owned();
    let pad = width - current;
    out.extend(std::iter::repeat_n(' ', pad));
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{display_width, pad_to_width, truncate_to_width};

    #[test]
    fn ascii_width_equals_char_count() {
        assert_eq!(display_width("hello"), 5);
    }

    #[test]
    fn double_width_char_counts_as_two() {
        // CJK character 世 has display width 2.
        assert_eq!(display_width("世"), 2);
        assert_eq!(display_width("世界"), 4);
    }

    #[test]
    fn emoji_hammer_counts_as_two() {
        // 🔨 is a double-width emoji.
        assert_eq!(display_width("🔨"), 2);
    }

    #[test]
    fn no_truncation_when_fits() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
        assert_eq!(truncate_to_width("hello", 5), "hello");
    }

    #[test]
    fn truncates_ascii() {
        let out = truncate_to_width("hello world", 8);
        // budget = 7 → "hello w…"
        assert_eq!(out, "hello w…");
        assert_eq!(display_width(&out), 8);
    }

    #[test]
    fn does_not_split_double_width_char() {
        // "世界X" = width 5; limit 4 → budget 3 → fits "世" (2) but not "界" (2+2=4>3)
        // so result = "世…" (width 3)
        let out = truncate_to_width("世界X", 4);
        assert_eq!(out, "世…");
        assert_eq!(display_width(&out), 3);
    }

    #[test]
    fn truncate_max_width_zero_returns_empty() {
        assert_eq!(truncate_to_width("hello", 0), "");
    }

    #[test]
    fn pad_to_width_adds_spaces() {
        let out = pad_to_width("hi", 5);
        assert_eq!(out, "hi   ");
        assert_eq!(display_width(&out), 5);
    }

    #[test]
    fn pad_does_not_truncate_wider_string() {
        let out = pad_to_width("hello world", 5);
        assert_eq!(out, "hello world");
    }

    #[test]
    fn pad_double_width_pads_by_display_width() {
        // "世界" has display width 4; padding to 7 should add 3 spaces.
        let out = pad_to_width("世界", 7);
        assert_eq!(display_width(&out), 7);
        assert!(out.ends_with("   "));
    }

    #[test]
    fn truncate_double_width_at_exact_budget() {
        // "AB🔨" = 2+2 = 4 cells. Truncate to 4 → fits → no ellipsis.
        assert_eq!(truncate_to_width("AB🔨", 4), "AB🔨");
    }

    #[test]
    fn truncate_drops_whole_double_width_char() {
        // "AB🔨C" = 5 cells. Truncate to 4 → budget 3 → "AB" (2) + 🔨 won't fit (4>3) → "AB…"
        let out = truncate_to_width("AB🔨C", 4);
        assert_eq!(out, "AB…");
        assert_eq!(display_width(&out), 3);
    }
}
