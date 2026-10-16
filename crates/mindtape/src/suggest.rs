//! Nearest-name suggestion for an unknown identifier against a known set
//! (`truth/tasks/suggest-a-nearest-match-on-an-unknown-view-name.typ`).
//!
//! [`nearest`] finds the closest known name by edit distance above a
//! similarity floor, and [`nearest_few`] the closest few, for a word a
//! dictionary lacks. [`Unknown`] holds the nearest name with the names to
//! choose from, so an error carries what the name could have been as data and
//! the CLI words it as `hint:` lines.

use strsim::normalized_damerau_levenshtein;

/// Below this normalized Damerau-Levenshtein similarity, in which a swap of
/// two letters is one edit (0.0 identical strings share nothing,
/// 1.0 identical), a candidate is not close enough to name — the far-miss
/// case falls back to the enumeration alone rather than pointing at an
/// unrelated name. 0.5 clears a single-character typo or transposition on a
/// short name (`"fpo"` vs `"foo"` scores 0.67) while excluding names that
/// merely share a substring (`"backlog"` vs `"blocked"` scores 0.43).
const SIMILARITY_FLOOR: f64 = 0.5;

/// Find the nearest match to `name` among `known`, if any clears
/// [`SIMILARITY_FLOOR`]. Ties resolve to the first-listed candidate: `known`
/// is scanned in order and a later, merely-equal score never displaces the
/// running best (replacement requires a strictly higher score), so the
/// result never varies between runs for the same candidate order.
#[must_use]
pub fn nearest<'a>(name: &str, known: &[&'a str]) -> Option<&'a str> {
    let mut best: Option<(&str, f64)> = None;
    for &candidate in known {
        let score = normalized_damerau_levenshtein(name, candidate);
        if score < SIMILARITY_FLOOR {
            continue;
        }
        let improves = best.is_none_or(|(_, best_score)| score > best_score);
        if improves {
            best = Some((candidate, score));
        }
    }
    best.map(|(candidate, _)| candidate)
}

/// Up to `limit` of `known` that clear [`SIMILARITY_FLOOR`] against `name`,
/// nearest first. Equal scores keep the order of `known`, so the answer never
/// varies between runs for the same candidate order.
#[must_use]
pub fn nearest_few<'a>(
    name: &str,
    known: impl IntoIterator<Item = &'a str>,
    limit: usize,
) -> Vec<&'a str> {
    let mut scored: Vec<(&str, f64)> = known
        .into_iter()
        .map(|candidate| (candidate, normalized_damerau_levenshtein(name, candidate)))
        .filter(|(_, score)| *score >= SIMILARITY_FLOOR)
        .collect();
    scored.sort_by(|(_, left), (_, right)| right.total_cmp(left));
    scored
        .into_iter()
        .take(limit)
        .map(|(candidate, _)| candidate)
        .collect()
}

/// [`nearest`], owned: the shape a library error carries. The one library
/// caller of [`nearest`]: clippy refuses every other.
#[must_use]
#[allow(clippy::disallowed_methods, reason = "the one sanctioned caller")]
pub fn nearest_owned(name: &str, known: &[&str]) -> Option<String> {
    nearest(name, known).map(str::to_owned)
}

/// What an unknown name could have been: the nearest known name, when one
/// clears [`SIMILARITY_FLOOR`], and the names to choose from. A library error
/// carries it as data; the CLI words it as `hint:` lines.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Unknown {
    /// The known name closest to the unknown one.
    pub nearest: Option<String>,
    /// Every known name, in listing order. Empty when the caller names none.
    pub names: Vec<String>,
}

impl Unknown {
    /// The nearest of `known` to `name`, and all of `known` as the names to
    /// choose from.
    #[must_use]
    pub fn among<'a>(name: &str, known: impl IntoIterator<Item = &'a str>) -> Self {
        let names: Vec<&str> = known.into_iter().collect();
        Self {
            nearest: nearest_owned(name, &names),
            names: names.into_iter().map(str::to_owned).collect(),
        }
    }

    /// The nearest of `known` to `name` alone, for a caller whose message
    /// already lists the legal values.
    #[must_use]
    pub fn near(name: &str, known: &[&str]) -> Self {
        Self {
            nearest: nearest_owned(name, known),
            names: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_finds_a_near_miss() {
        let known = ["backlog", "blocked", "default"];
        assert_eq!(nearest("backlgo", &known), Some("backlog"));
    }

    #[test]
    fn nearest_rejects_a_far_miss() {
        let known = ["backlog", "blocked", "default"];
        assert_eq!(nearest("zzz", &known), None);
    }

    #[test]
    fn nearest_ties_resolve_to_candidate_order() {
        // "abc" is equidistant from both "abx" and "axc" (one substitution
        // each); the first-listed candidate must win regardless of order.
        assert_eq!(nearest("abc", &["abx", "axc"]), Some("abx"));
        assert_eq!(nearest("abc", &["axc", "abx"]), Some("axc"));
    }

    #[test]
    fn nearest_prefers_the_higher_scoring_candidate() {
        // "fo" also clears the floor, so only "the strictly higher score
        // wins" picks "foo" either way the candidates are listed.
        assert_eq!(nearest("foo", &["fo", "foo"]), Some("foo"));
        assert_eq!(nearest("foo", &["foo", "fo"]), Some("foo"));
    }

    #[test]
    fn nearest_few_ranks_by_score_and_breaks_ties_in_candidate_order() {
        // One edit from "rote" scores 0.75 and "rotate" two, at 0.67, so
        // the four-way tie at 0.75 keeps candidate order.
        let known = ["rotate", "vote", "note", "rate", "rite"];
        assert_eq!(nearest_few("rote", known, 3), ["vote", "note", "rate"]);
        assert_eq!(
            nearest_few("rote", known, 9),
            ["vote", "note", "rate", "rite", "rotate"]
        );
    }

    #[test]
    fn nearest_few_drops_a_score_below_the_floor_and_past_the_limit() {
        // "ab" vs "abcd" scores exactly 0.5; "abc" vs "xyzc" scores 0.25.
        assert_eq!(nearest_few("ab", ["abcd"], 3), ["abcd"]);
        assert!(nearest_few("abc", ["xyzc"], 3).is_empty());
        assert!(nearest_few("rote", ["note"], 0).is_empty());
        assert_eq!(nearest_few("rote", ["note", "vote"], 1), ["note"]);
    }

    #[test]
    fn among_carries_the_nearest_and_every_name() {
        let unknown = Unknown::among("backlgo", ["backlog", "blocked"]);
        assert_eq!(unknown.nearest.as_deref(), Some("backlog"));
        assert_eq!(unknown.names, ["backlog", "blocked"]);
    }

    #[test]
    fn among_far_miss_has_no_nearest() {
        let unknown = Unknown::among("zzz", ["backlog", "blocked"]);
        assert_eq!(unknown.nearest, None);
        assert_eq!(unknown.names, ["backlog", "blocked"]);
    }

    #[test]
    fn near_carries_no_names() {
        let unknown = Unknown::near("backlgo", &["backlog", "blocked"]);
        assert_eq!(unknown.nearest.as_deref(), Some("backlog"));
        assert!(unknown.names.is_empty());
    }
}
