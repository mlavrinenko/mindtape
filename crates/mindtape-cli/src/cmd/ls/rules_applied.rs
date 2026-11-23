//! `mt ls` attribution: [`Applied`], which source won each facet of a resolved
//! listing, and the dim footer it renders as under the table:
//! `3 of 12 tasks · default view (!terminal && (miss(start) || start <= now()))`,
//! with `· 1 broken` when a file failed to load.
//!
//! Attribution is per facet because selection and presentation come from
//! different places. An explicit filter argument answers "which tasks", while a
//! matching `[[ls.rules]]` entry goes on answering "how do I read this branch's
//! listing". One listing then routinely owes its facets to two sources at once
//! (`truth/tasks/2026-09/ls-rules-lose-their-sort-and-columns-facets-under-an-explicit-filter.typ`).
//! The `mt ls --json` envelope carries every facet's source. The footer shows
//! the filter a view or a rule applied, or the view's description, and a rule
//! that set the sort. [`crate::cmd::ls::rules`] does the resolving, and this
//! file records and renders the outcome.

use crate::diag::{Command, hint_line};
use crate::render::width::display_width;

/// The source label meaning "nothing applied here, the built-in default".
pub const MATCH_ALL: &str = "match-all";
/// The source label for an explicit CLI argument: the filter positional,
/// `--sort` or `--columns`. The code attributes a rule's own
/// `filter`/`sort`/`columns` override to the rule instead. A listing whose filter
/// came from the command line and whose sort came from a rule override then
/// cannot report both as one source.
pub const MANUAL: &str = "manual";

/// The indent a wrapped footer line continues under.
const CONTINUATION: &str = "  ";

/// Source attribution for [`crate::cmd::ls::rules::Bundle`], driving the human
/// footer and the `mt ls --json` envelope. Each facet's `_from` is a view
/// name, a rule's label, [`MANUAL`], or [`MATCH_ALL`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Applied {
    /// The `[[ls.rules]]` entry that matched, by label. `None` when none did.
    /// Set whether the rule went on to contribute a facet. A rule that matched
    /// and then lost every facet is still what the listing answered to.
    /// [`Applied::rule_superseded`] is how the JSON envelope says so, rather than
    /// leaving it indistinguishable from a rule that never matched.
    pub rule: Option<String>,
    /// The matched rule contributed at least one facet and none of them
    /// survived the layers stacked on top ([`Applied::note_supersession`]).
    pub rule_superseded: bool,
    pub filter_from: String,
    pub filter_value: String,
    /// The description of the view that supplied the filter, when it has one.
    /// [`footer`] shows it in place of the filter, which is longer and says
    /// less about why the view exists. A rule's own filter has none.
    pub filter_description: Option<String>,
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

    /// The three facet sources, filter, sort and columns. The code takes this
    /// snapshot before the explicit-filter and flag layers run, so
    /// [`Applied::note_supersession`] can tell what the matched rule still owns
    /// afterwards.
    pub(crate) fn sources(&self) -> [String; 3] {
        [
            self.filter_from.clone(),
            self.sort_from.clone(),
            self.columns_from.clone(),
        ]
    }

    /// Record whether the matched rule's contribution survived: `before` is
    /// [`Applied::sources`] as the rule left them, `self` carries the final
    /// ones. Superseded means the rule supplied at least one facet and every one
    /// of them has since been overwritten. A rule that supplied nothing in the
    /// first place is not superseded, it is simply empty.
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

/// The lines under a table. They lead with the rows shown of the project's
/// tasks, then say where those rows came from and a rule that set the sort. The
/// filter's source shows the filter it applied, or the view's description. They
/// end with the task files that failed to load. The parts join with ` · `:
/// `3 of 12 tasks · inbox view (priority > 5) · release sort`. The count leads
/// because it is the shortest part and the one a reader looks for first. A
/// wrapped line then keeps it on the line the table ends on. A line wider than
/// `width` wraps at a ` · ` separator, or at a space inside a part that is
/// still too long, and each continuation line starts with [`CONTINUATION`].
/// `None` leaves the line whole, since a pipe has no width. The footer doesn't
/// say which facet came from where, or that the command line overrode a rule.
/// The `mt ls --json` envelope carries both. A listing nothing shaped gets the
/// count alone.
#[must_use]
pub fn footer(
    applied: &Applied,
    shown: usize,
    total: usize,
    broken: usize,
    width: Option<usize>,
) -> String {
    let mut parts = vec![format!("{shown} of {}", mindtape::count(total, "task"))];
    parts.extend(source_parts(applied, filter_part(applied)));
    parts.extend(broken_part(broken));
    wrap(&parts.join(" · "), width).join("\n")
}

/// [`footer`]'s last part, `1 broken`, counting the task files that failed to
/// load. It is absent when every file loaded.
#[must_use]
pub fn broken_part(broken: usize) -> Option<String> {
    (broken > 0).then(|| format!("{broken} broken"))
}

/// The line under a listing that selected nothing: how to widen the filter to
/// every task, or where a saved view appears. `None` when a row showed, and when
/// the project has no task to widen to. The commands carry this run's `@name` or
/// `-w`, and the line pastes where it printed.
#[must_use]
pub fn empty_hint(shown: usize, total: usize) -> Option<String> {
    if shown > 0 || total == 0 {
        return None;
    }
    let every = Command::mt().word("ls").arg("");
    let views = Command::mt().word("config").word("show").word("--resolved");
    Some(hint_line(format!(
        "no task matches; run {every} for every task, or {views} for a saved view"
    )))
}

/// [`footer`]'s parts after the count: `["inbox view", "release sort"]`,
/// empty when nothing shaped the listing. The filter part shows its source
/// alone, since a set compares them across projects. [`footer`] adds the
/// filter or the view's description.
#[must_use]
pub fn footer_sources(applied: &Applied) -> Vec<String> {
    source_parts(applied, source_label(applied))
}

/// The parts after the count: `filter` when it gives a source, a rule that
/// set the sort, or a rule that only picked the columns.
fn source_parts(applied: &Applied, filter: Option<String>) -> Vec<String> {
    let mut parts: Vec<String> = filter.into_iter().collect();
    let rule = applied.rule.as_deref();
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
fn source_label(applied: &Applied) -> Option<String> {
    let from = applied.filter_from.as_str();
    match from {
        MATCH_ALL => None,
        MANUAL => Some("your filter".to_owned()),
        _ if applied.rule.as_deref() == Some(from) => Some(format!("{from} rule")),
        _ => Some(format!("{from} view")),
    }
}

/// [`source_label`] with what the source applied: the view's description when
/// it has one, else the filter itself. `default view (!terminal && ...)`,
/// `agent view (Open tasks an agent can take)`. A source with no filter to
/// show stays bare, and so does a filter the user typed.
fn filter_part(applied: &Applied) -> Option<String> {
    let label = source_label(applied)?;
    if applied.filter_from == MANUAL {
        return Some(label);
    }
    let detail = applied
        .filter_description
        .as_deref()
        .filter(|description| !description.is_empty())
        .unwrap_or(&applied.filter_value);
    if detail.is_empty() {
        return Some(label);
    }
    Some(format!("{label} ({detail})"))
}

/// `line` broken into lines no wider than `width` display cells, at the ` · `
/// separators first and then at spaces inside a part that is still too long.
/// Each continuation line starts with [`CONTINUATION`], and a `·` separator
/// stays with the part it introduces, so no line ends with one. `None` leaves
/// the line whole, since a pipe has no width to wrap at.
fn wrap(line: &str, width: Option<usize>) -> Vec<String> {
    let Some(width) = width else {
        return vec![line.to_owned()];
    };
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for (index, part) in line.split(" · ").enumerate() {
        let separator = if index == 0 { "" } else { "· " };
        for unit in units(part, separator, width) {
            let candidate = if current.is_empty() {
                unit.clone()
            } else {
                format!("{current} {unit}")
            };
            if !current.is_empty() && display_width(&candidate) > width {
                lines.push(std::mem::take(&mut current));
                current = format!("{CONTINUATION}{unit}");
            } else {
                current = candidate;
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// `part` as one unit when it fits a line, else its words, so a short part
/// such as the count never breaks. `separator` leads the part when it is not
/// the first, and stays with its first word.
fn units(part: &str, separator: &str, width: usize) -> Vec<String> {
    let piece = format!("{separator}{part}");
    if display_width(&piece) + display_width(CONTINUATION) <= width {
        return vec![piece];
    }
    let mut words: Vec<String> = part.split_whitespace().map(str::to_owned).collect();
    if let Some(first) = words.first_mut() {
        first.insert_str(0, separator);
    }
    words
}

#[cfg(test)]
#[path = "rules_applied_ls_rules_tests_applied.rs"]
mod tests;
