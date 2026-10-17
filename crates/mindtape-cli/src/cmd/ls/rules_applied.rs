//! `mt ls` attribution: [`Applied`] — which source won each facet of a
//! resolved listing — and the dim footer it renders as under the table:
//! `default view · 3 of 12 tasks`.
//!
//! Attribution is per facet because selection and presentation come from
//! different places: an explicit filter argument answers "which tasks" while a
//! matching `[[ls.rules]]` entry goes on answering "how do I read this branch's
//! listing", so one listing routinely owes its facets to two sources at once
//! (`truth/tasks/ls-rules-lose-their-sort-and-columns-facets-under-an-explicit-filter.typ`).
//! The `mt ls --json` envelope carries every facet's source; the footer names
//! only the rows' source and a rule that set the sort.
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
/// footer and the `mt ls --json` envelope. Each facet's `_from` is a view
/// name, a rule's label, [`MANUAL`], or [`MATCH_ALL`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Applied {
    /// The `[[ls.rules]]` entry that matched, by label; `None` when none did.
    /// Set whether or not the rule went on to contribute a facet — a rule that
    /// matched and was then overridden everywhere is still what the listing
    /// answered to, and [`Applied::rule_superseded`] is how the JSON envelope
    /// says so rather than leaving it indistinguishable from a rule that never
    /// matched.
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

    /// The three facet sources: filter, sort, columns. Taken as a snapshot before
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
}

/// The line under a table: where the listing's rows came from, a rule that
/// set the sort, then the rows shown of the project's tasks, joined with
/// ` · `: `inbox view · release sort · 3 of 12 tasks`. It doesn't say which
/// facet came from where, or that the command line overrode a rule; the
/// `mt ls --json` envelope carries both. A listing nothing shaped gets the
/// count alone.
#[must_use]
pub fn footer(applied: &Applied, shown: usize, total: usize) -> String {
    let mut parts = footer_sources(applied);
    parts.push(format!("{shown} of {}", mindtape::count(total, "task")));
    parts.join(" · ")
}

/// [`footer`]'s parts before the count: `["inbox view", "release sort"]`,
/// empty when nothing shaped the listing.
#[must_use]
pub fn footer_sources(applied: &Applied) -> Vec<String> {
    let rule = applied.rule.as_deref();
    let mut parts: Vec<String> = Vec::new();
    if let Some(label) = source_label(rule, &applied.filter_from) {
        parts.push(label);
    }
    let sort = applied.sort_from.as_str();
    if rule == Some(sort) && sort != applied.filter_from {
        parts.push(format!("{sort} sort"));
    }
    if parts.is_empty() && rule == Some(applied.columns_from.as_str()) {
        parts.push(format!("{} rule", applied.columns_from));
    }
    parts
}

/// The filter's source: `your filter` for what the user typed, `release rule`
/// for the matched rule, `inbox view` for a view, nothing for the built-in
/// match-all.
fn source_label(rule: Option<&str>, from: &str) -> Option<String> {
    match from {
        MATCH_ALL => None,
        MANUAL => Some("your filter".to_owned()),
        _ if rule == Some(from) => Some(format!("{from} rule")),
        _ => Some(format!("{from} view")),
    }
}

#[cfg(test)]
#[path = "rules_applied_ls_rules_tests_applied.rs"]
mod tests;
