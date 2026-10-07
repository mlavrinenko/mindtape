//! `mt ls` attribution: [`Applied`] — which source won each facet of a
//! resolved listing — and the dim `applied:` trailer it renders as under the
//! table.
//!
//! Attribution is per facet because selection and presentation come from
//! different places: an explicit filter argument answers "which tasks" while a
//! matching `[[ls.rules]]` entry goes on answering "how do I read this branch's
//! listing", so one listing routinely owes its facets to two sources at once
//! and the trailer has to name both
//! (`truth/tasks/ls-rules-lose-their-sort-and-columns-facets-under-an-explicit-filter.typ`).
//! [`crate::cmd::ls::rules`] does the resolving; this file records and renders the
//! outcome.

/// The source label meaning "nothing applied here — the built-in default".
pub const MATCH_ALL: &str = "match-all";
/// The source label for an explicit CLI argument: the filter positional,
/// `--sort` or `--columns`. A rule's own `filter`/`sort`/`columns` override is
/// attributed to the rule instead, so a listing whose filter came from the
/// command line and whose sort came from a rule override cannot report both as
/// one source.
pub const MANUAL: &str = "manual";

/// Source attribution for [`crate::cmd::ls::rules::Bundle`], driving the human
/// trailer and the `mt ls --json` envelope. Each facet's `_from` is a view
/// name, a rule's label, [`MANUAL`], or [`MATCH_ALL`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Applied {
    /// The `[[ls.rules]]` entry that matched, by label; `None` when none did.
    /// Set whether or not the rule went on to contribute a facet — a rule that
    /// matched and was then overridden everywhere is still what the listing
    /// answered to, and [`Applied::rule_superseded`] is how the trailer says so
    /// rather than leaving it indistinguishable from a rule that never matched.
    pub rule: Option<String>,
    /// The matched rule contributed at least one facet and none of them
    /// survived the layers stacked on top ([`Applied::note_supersession`]).
    pub rule_superseded: bool,
    pub filter_from: String,
    pub filter_value: String,
    pub sort_from: String,
    pub sort_value: Vec<String>,
    pub columns_from: String,
    pub columns_value: Vec<String>,
}

impl Applied {
    pub(crate) fn match_all() -> Self {
        Self {
            filter_from: MATCH_ALL.to_owned(),
            sort_from: MATCH_ALL.to_owned(),
            columns_from: MATCH_ALL.to_owned(),
            ..Self::default()
        }
    }

    /// The three facet sources, in trailer order. Taken as a snapshot before
    /// the explicit-filter and flag layers run, so [`Applied::note_supersession`]
    /// can tell what the matched rule still owns afterwards.
    pub(crate) fn sources(&self) -> [String; 3] {
        [
            self.filter_from.clone(),
            self.sort_from.clone(),
            self.columns_from.clone(),
        ]
    }

    /// Record whether the matched rule's contribution survived: `before` is
    /// [`Applied::sources`] as the rule left them, `self` carries the final
    /// ones. Superseded means the rule supplied at least one facet and every
    /// one of them has since been overwritten — a rule that supplied nothing in
    /// the first place is not superseded, it is simply empty.
    pub(crate) fn note_supersession(&mut self, before: &[String; 3]) {
        if self.rule.is_none() {
            return;
        }
        let after = self.sources();
        let supplied = before.iter().filter(|from| *from != MATCH_ALL);
        let survivors = before
            .iter()
            .zip(&after)
            .filter(|(from, now)| *from != MATCH_ALL && from == now);
        self.rule_superseded = supplied.count() > 0 && survivors.count() == 0;
    }

    /// Whether nothing worth mentioning applied: no rule fired and every
    /// facet is still at the built-in default. The trailer stays silent for
    /// this case; the JSON envelope still reports it (`"rule": null`, every
    /// facet `"from": "match-all"`).
    #[must_use]
    pub fn is_trivial(&self) -> bool {
        self.rule.is_none()
            && self.filter_from == MATCH_ALL
            && self.sort_from == MATCH_ALL
            && self.columns_from == MATCH_ALL
    }
}

/// The human trailer line, grouped by winning source in first-appearance
/// order (filter, then sort, then columns), with a matched-but-overridden rule
/// named at the end; `None` when [`Applied::is_trivial`].
#[must_use]
pub fn trailer(applied: &Applied) -> Option<String> {
    if applied.is_trivial() {
        return None;
    }
    let parts: Vec<String> = group_by_source(applied)
        .into_iter()
        .map(|(source, facets)| format!("{source} ({})", facets.join(", ")))
        .collect();
    let mut line = parts.join(" → ");
    if applied.rule_superseded {
        let note = format!("{} superseded", applied.rule.as_deref().unwrap_or_default());
        line = if line.is_empty() {
            note
        } else {
            format!("{line} · {note}")
        };
    }
    (!line.is_empty()).then(|| format!("applied: {line}"))
}

/// The facets each source won, sources in first-appearance order; a facet
/// still at [`MATCH_ALL`] has no source and is left out.
fn group_by_source(applied: &Applied) -> Vec<(&str, Vec<&str>)> {
    let facets = [
        ("filter", applied.filter_from.as_str()),
        ("sort", applied.sort_from.as_str()),
        ("columns", applied.columns_from.as_str()),
    ];
    let mut grouped: Vec<(&str, Vec<&str>)> = Vec::new();
    for (facet, from) in facets {
        if from == MATCH_ALL {
            continue;
        }
        if let Some(entry) = grouped.iter_mut().find(|(source, _)| *source == from) {
            entry.1.push(facet);
        } else {
            grouped.push((from, vec![facet]));
        }
    }
    grouped
}

#[cfg(test)]
#[path = "rules_applied_ls_rules_tests_applied.rs"]
mod tests;
