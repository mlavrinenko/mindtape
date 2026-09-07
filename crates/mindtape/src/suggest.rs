//! Nearest-name suggestion for an unknown identifier against a known set
//! (`truth/tasks/suggest-a-nearest-match-on-an-unknown-view-name.typ`).
//!
//! Every other unknown-name error in the CLI enumerates the legal values
//! (`unknown column ...expected one of`, `unknown status ...configured
//! legend`, `unknown --set/-f key(s) ...known slots`) — an unknown view name
//! used to be the outlier, a bare `unknown view: <name>` with neither a
//! nearest match nor the list of views that do exist. [`nearest`] finds the
//! closest known name by edit distance above a similarity floor; [`hint`]
//! renders it plus the full enumeration as the trailing clause every
//! unknown-view error now appends.

use strsim::normalized_levenshtein;

/// Below this normalized similarity (0.0 identical strings share nothing,
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
        let score = normalized_levenshtein(name, candidate);
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

/// Render the trailing hint clause for an unknown `name` not found among
/// `known`: `; did you mean: <name>?; known <kind>s: a, b, c` when a near
/// match clears [`SIMILARITY_FLOOR`], `; known <kind>s: a, b, c` alone
/// otherwise. Returns an empty string when `known` is empty — nothing to
/// suggest and nothing to enumerate, so the caller's own message is left
/// unchanged. `kind` names what is being enumerated (`"view"` today; the
/// column / status / slot / tag / worktree sites are a follow-up), pluralized
/// with a trailing `s`.
#[must_use]
pub fn hint<'a, I>(kind: &str, name: &str, known: I) -> String
where
    I: IntoIterator<Item = &'a String>,
{
    let known: Vec<&str> = known.into_iter().map(String::as_str).collect();
    if known.is_empty() {
        return String::new();
    }
    let suggestion = nearest(name, &known)
        .map(|m| format!("did you mean: {m}?; "))
        .unwrap_or_default();
    format!("; {suggestion}known {kind}s: {}", known.join(", "))
}

/// Render just the nearest-match fragment as a trailing clause: `; did you
/// mean: <name>?` when a candidate clears [`SIMILARITY_FLOOR`], empty
/// otherwise (including when `known` is empty — nothing to suggest).
///
/// [`hint`] folds a suggestion together with a full enumeration for a caller
/// that has neither; a caller that already enumerates the legal values
/// (column / status / slot sites) appends this instead, so the enumeration is
/// never printed twice.
#[must_use]
pub fn suggestion(name: &str, known: &[&str]) -> String {
    nearest(name, known)
        .map(|m| format!("; did you mean: {m}?"))
        .unwrap_or_default()
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
    fn hint_near_miss_offers_a_suggestion_and_the_full_list() {
        let known = vec!["backlog".to_owned(), "blocked".to_owned()];
        let rendered = hint("view", "backlgo", &known);
        assert_eq!(
            rendered,
            "; did you mean: backlog?; known views: backlog, blocked"
        );
    }

    #[test]
    fn hint_far_miss_omits_the_suggestion() {
        let known = vec!["backlog".to_owned(), "blocked".to_owned()];
        let rendered = hint("view", "zzz", &known);
        assert_eq!(rendered, "; known views: backlog, blocked");
    }

    #[test]
    fn hint_empty_known_set_is_silent() {
        let known: Vec<String> = Vec::new();
        assert_eq!(hint("view", "anything", &known), "");
    }

    #[test]
    fn suggestion_near_miss_renders_the_clause_alone() {
        let known = ["backlog", "blocked", "default"];
        assert_eq!(suggestion("backlgo", &known), "; did you mean: backlog?");
    }

    #[test]
    fn suggestion_far_miss_is_empty() {
        let known = ["backlog", "blocked", "default"];
        assert_eq!(suggestion("zzz", &known), "");
    }

    #[test]
    fn suggestion_empty_known_set_is_empty() {
        assert_eq!(suggestion("anything", &[]), "");
    }
}
