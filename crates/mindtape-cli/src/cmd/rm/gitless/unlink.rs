//! The inbound-edge walk: which *other* task files name a task this run is
//! taking away from where it stands, and how each of those edges is spliced so
//! none of them is left dangling.
//!
//! One walk serves both commands that need it, because they differ only in
//! where the edge ends up ([`Retarget`]): `mt rm`'s `unlink` strategy strips
//! the edge — removal is the null target — and `mt mv` points it at the file
//! stem the task is renamed to. Locating an inbound edge, splicing it and
//! counting it are the same work either way.
//!
//! Split out of `cmd/rm/gitless/mod.rs` to stay under the file-size gate; `mt mv`
//! reaches it through `rm`'s re-exports rather than owning a second copy.

use std::collections::HashSet;
use std::ops::Range;
use std::path::Path;

use crate::diag::Result;
use mindtape::{Task, normalize_link};
use typst_cleanup::{Plan, Rewrite};

use crate::diag::Diag;
use crate::task_edit::task_with;

use super::super::artifacts;

/// Where an inbound edge to a matched task points once the run is over.
#[derive(Debug, Clone, Copy)]
pub enum Retarget<'a> {
    /// Nowhere: strip the edge whole, operator and all — `mt rm`'s `unlink`
    /// strategy, and the null target the retarget generalises.
    Drop,
    /// At this file stem. Each raw target keeps its own directory prefix and
    /// extension — `mt mv` renames in place, so only the last path component
    /// changes and `"../sub/old.typ"` becomes `"../sub/new.typ"`.
    Stem(&'a str),
}

/// Rewrite every inbound edge naming a task in `matched`, across the task files
/// that are *not* themselves matched — a matched task is the one going away or
/// moving, and its own file is the caller's business, not this walk's.
///
/// Returns one [`Rewrite`] per file whose source actually changed, sorted by
/// relative path for a stable report. Nothing is written.
///
/// # Errors
/// A referring file that cannot be read, or a spliced source `typst_edit`
/// refuses to reassemble, fails the run.
pub fn inbound_rewrites(
    tasks: &[Task],
    matched: &HashSet<String>,
    root: &Path,
    to: Retarget<'_>,
) -> Result<Vec<Rewrite>> {
    let mut rewrites: Vec<Rewrite> = Vec::new();
    for task in tasks {
        if matched.contains(&task.id) {
            continue;
        }
        let abs = root.join(&task.file);
        let source =
            std::fs::read_to_string(&abs).map_err(|err| Diag::cannot_read(&task.file, &err))?;
        let (edits, links) = inbound_edits(&source, matched, to);
        if links == 0 {
            continue;
        }
        let contents = typst_edit::apply(&source, edits).map_err(|err| {
            Diag::failed("can't rewrite links")
                .value(task.file.display().to_string())
                .cause(&err)
        })?;
        rewrites.push(Rewrite {
            abs,
            rel: task.file.clone(),
            contents,
            links,
        });
    }
    rewrites.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(rewrites)
}

/// The edits one referring source needs, and how many inbound edges they cover.
/// The count, not the edit count, is what the report quotes: stripping merges
/// adjacent ranges, so one edit can carry several edges away.
fn inbound_edits(
    source: &str,
    matched: &HashSet<String>,
    to: Retarget<'_>,
) -> (Vec<typst_edit::Edit>, usize) {
    match to {
        Retarget::Drop => {
            let (ranges, stripped) = unlink_edits(source, matched);
            let edits = ranges
                .into_iter()
                .map(|range| typst_edit::Edit::new(range, String::new()))
                .collect();
            (edits, stripped)
        }
        Retarget::Stem(stem) => {
            let edits = stem_edits(source, matched, stem);
            let count = edits.len();
            (edits, count)
        }
    }
}

/// Point every target literal naming a matched task at `stem`, in place: the
/// quoted literal is replaced and nothing around it moves, so a `related(…)`
/// note, an `edge(…)` kind and the sibling edges of a sum survive byte for
/// byte.
fn stem_edits(source: &str, matched: &HashSet<String>, stem: &str) -> Vec<typst_edit::Edit> {
    mindtape::link_target_spans(source)
        .into_iter()
        .filter(|span| matched.contains(&normalize_link(&span.raw)))
        .map(|span| {
            let raw = retarget_raw(&span.raw, stem);
            typst_edit::Edit::new(span.range, format!("\"{raw}\""))
        })
        .collect()
}

/// `raw` with its file stem replaced by `stem`, keeping the directory prefix
/// and the extension it was authored with: `"../sub/old.typ"` with stem `new`
/// is `"../sub/new.typ"`, and a bare `"old"` is `"new"`.
///
/// String work rather than [`Path`] joining, because a link target is authored
/// with `/` separators on every platform and rebuilding it through `Path` would
/// emit `\` on Windows.
#[must_use]
pub fn retarget_raw(raw: &str, stem: &str) -> String {
    let (dir, name) = raw
        .rfind('/')
        .map_or(("", raw), |slash| raw.split_at(slash + 1));
    // A leading dot opens a hidden file, not an extension; a stem may itself
    // carry dots (`foo.task-260101.typ`), so only the last one counts.
    let ext = name
        .rfind('.')
        .filter(|dot| *dot > 0)
        .map_or("", |dot| name.split_at(dot).1);
    format!("{dir}{stem}{ext}")
}

/// The `unlink` strategy's plan: delete the eligible task files, and strip
/// every inbound edge that references one from the surviving files. No git,
/// no tombstones.
///
/// # Errors
/// A survivor file that cannot be read, or a stripped source that cannot be
/// spliced back together, fails the run.
pub fn unlink_plan(tasks: &[Task], eligible: &HashSet<String>, root: &Path) -> Result<Plan> {
    Ok(Plan {
        deletions: typst_cleanup::delete_plan(&artifacts(tasks, eligible, root).0).deletions,
        rewrites: inbound_rewrites(tasks, eligible, root, Retarget::Drop)?,
    })
}

/// Which byte ranges of `source` to strip to unlink every removed task: whole
/// edge-helper calls (`depends-on("gone.typ") + related(...)[note]`) and
/// whole `link("gone.typ")` calls, each extended across a `+` operator (and
/// its whitespace) so no dangling operator survives. Returns the merged ranges
/// and how many edges were stripped.
fn unlink_edits(source: &str, eligible: &HashSet<String>) -> (Vec<Range<usize>>, usize) {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for call in mindtape::link_edges(source) {
        if call
            .targets
            .iter()
            .any(|target| eligible.contains(&normalize_link(&target.raw)))
        {
            ranges.push(call.range.clone());
        }
    }
    for call in typst_edit::find_calls(source, "link") {
        let targeted = call
            .args
            .iter()
            .filter(|arg| arg.name.is_none())
            .any(|arg| {
                source.as_bytes().get(arg.value_range.start) == Some(&b'"')
                    && eligible.contains(&normalize_link(&arg.value))
            });
        if targeted {
            ranges.push(call.range.clone());
        }
    }
    let stripped = ranges.len();
    if ranges.is_empty() {
        return (Vec::new(), 0);
    }
    let mut removals: Vec<Range<usize>> = ranges
        .into_iter()
        .map(|range| sweep_operator(range, source))
        .collect();
    removals.sort_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for removal in removals {
        match merged.last_mut() {
            Some(prev) if removal.start <= prev.end => prev.end = prev.end.max(removal.end),
            _ => merged.push(removal),
        }
    }
    // Stripped edges cover the whole `links:` value: drop the named argument.
    if let Some(range) = whole_links_edit(source, &merged) {
        return (std::iter::once(range).collect(), stripped);
    }
    (merged, stripped)
}

/// When the stripped edges cover the whole `links:` value, the range of the
/// named argument itself (separator comma and all) — dropping the edges alone
/// would leave a dangling `links: ` key.
fn whole_links_edit(source: &str, merged: &[Range<usize>]) -> Option<Range<usize>> {
    if let Ok(call) = task_with(source)
        && let Some(value) = call.named("links")
    {
        let value = value.value_range.clone();
        let bytes = source.as_bytes();
        let whole = value.clone().all(|byte| {
            bytes.get(byte).is_none_or(u8::is_ascii_whitespace)
                || merged.iter().any(|range| range.contains(&byte))
        });
        if whole {
            let before = bytes
                .iter()
                .take(value.start)
                .rposition(|byte| *byte == b',' || *byte == b'(')
                .map_or(value.start, |pos| pos + 1);
            let mut end = merged
                .iter()
                .filter(|range| range.start < value.end)
                .map(|range| range.end)
                .max()
                .unwrap_or(value.end);
            let cursor = end + whitespace_from(bytes, end);
            if bytes.get(cursor) == Some(&b',') {
                end = cursor + 1;
            }
            return Some(before..end);
        }
    }
    None
}

/// Extend `range` across a `+` operator — `edge + range` or `range + edge` —
/// with its surrounding whitespace, so stripping the edge leaves no dangling
/// `+`. Both directions: a removed operand may sit anywhere in the sum.
fn sweep_operator(range: Range<usize>, source: &str) -> Range<usize> {
    let bytes = source.as_bytes();
    let mut start = range.start;
    let cursor = range.start - whitespace_before(bytes, range.start);
    if cursor > 0 && bytes.get(cursor - 1) == Some(&b'+') {
        let operator = cursor - 1;
        start = operator - whitespace_before(bytes, operator);
    }
    let ahead = range.end + whitespace_from(bytes, range.end);
    let end = if bytes.get(ahead) == Some(&b'+') {
        ahead + 1 + whitespace_from(bytes, ahead + 1)
    } else {
        range.end
    };
    Range { start, end }
}

/// The number of ASCII-whitespace bytes immediately before `end`.
fn whitespace_before(bytes: &[u8], end: usize) -> usize {
    bytes
        .get(..end)
        .unwrap_or_default()
        .iter()
        .rev()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count()
}

/// The number of ASCII-whitespace bytes immediately from `from` on.
fn whitespace_from(bytes: &[u8], from: usize) -> usize {
    bytes
        .get(from..)
        .unwrap_or_default()
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::collections::HashSet;

    use super::{retarget_raw, sweep_operator, task_with, unlink_edits, whole_links_edit};

    fn eligible(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    #[test]
    fn retargeting_keeps_the_relative_prefix_and_the_extension() {
        assert_eq!(retarget_raw("old.typ", "new"), "new.typ");
        assert_eq!(retarget_raw("../sub/old.typ", "new"), "../sub/new.typ");
    }

    #[test]
    fn retargeting_a_bare_stem_invents_no_extension() {
        assert_eq!(retarget_raw("old", "new"), "new");
        assert_eq!(retarget_raw("sub/old", "new"), "sub/new");
    }

    #[test]
    fn retargeting_replaces_the_whole_stem_dots_included() {
        assert_eq!(retarget_raw("old.task-260101.typ", "new"), "new.typ");
    }

    #[test]
    fn retargeting_a_hidden_target_reads_the_leading_dot_as_a_name() {
        assert_eq!(retarget_raw(".hidden", "new"), "new");
        assert_eq!(retarget_raw("../.hidden", "new"), "../new");
    }

    /// The wrapping whitespace on either side of an edge belongs to the
    /// operator sweep: removing `edge` from `a + edge` must take the ` + `
    /// too, whatever whitespace the author used, or a dangling operator is
    /// left behind.
    #[test]
    fn sweep_attaches_a_leading_operator_and_its_whitespace() {
        assert_eq!(sweep_operator(4..8, "a + edge"), 1..8);
    }

    #[test]
    fn sweep_attaches_a_trailing_operator_and_its_whitespace() {
        assert_eq!(sweep_operator(0..4, "edge + b"), 0..7);
    }

    /// The whitespace after a trailing `+` is counted from the byte after the
    /// operator, so each side can carry a different run length.
    #[test]
    fn sweep_counts_whitespace_on_only_one_side_of_a_trailing_operator() {
        assert_eq!(sweep_operator(0..4, "edge+  b"), 0..7);
        assert_eq!(sweep_operator(0..4, "edge +b"), 0..6);
    }

    #[test]
    fn sweep_reads_a_leading_operator_without_whitespace_before_it() {
        assert_eq!(sweep_operator(3..7, "a+ edge"), 1..7);
    }

    /// The scan stops at the start of the source rather than reading past it;
    /// an all-whitespace prefix (or one that reaches index 0 before a `+`) is
    /// where an inclusive bound would underflow.
    #[test]
    fn sweep_stops_at_the_start_of_the_source_without_underflow() {
        assert_eq!(sweep_operator(2..2, "  "), 2..2);
        assert_eq!(sweep_operator(4..4, "  + "), 0..4);
    }

    /// Two removed edges in one sum either side of a `+` overlap after the
    /// operator sweep, so they splice as one range rather than two adjacent
    /// edits.
    #[test]
    fn overlapping_removals_merge_into_one_range() {
        let eligible = eligible(&["a", "b"]);
        let (ranges, stripped) = unlink_edits(
            "#let x = depends-on(\"a.typ\") + depends-on(\"b.typ\")",
            &eligible,
        );
        assert_eq!(ranges.len(), 1, "{ranges:?}");
        assert_eq!(stripped, 2);
    }

    /// Removals with a kept edge between them are not merged, or the kept edge
    /// would be deleted with them.
    #[test]
    fn separated_removals_stay_separate() {
        let eligible = eligible(&["a", "b"]);
        let (ranges, stripped) = unlink_edits(
            "#let x = depends-on(\"a.typ\") + depends-on(\"keep.typ\") + depends-on(\"b.typ\")",
            &eligible,
        );
        assert_eq!(ranges.len(), 2, "{ranges:?}");
        assert_eq!(stripped, 2);
    }

    /// The whole-`links:` end is the end of the merged edits that start inside
    /// the value; an edit beginning exactly at the value's end is outside it
    /// and must not stretch the removal any further.
    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn whole_links_edit_extends_the_end_from_ranges_inside_the_value_only() {
        let source = "#show: task.with(title: \"T\", links: \"abc\")\n";
        let value = task_with(source)
            .unwrap()
            .named("links")
            .unwrap()
            .value_range
            .clone();
        let runs_past_the_value = vec![value.start..value.end + 1];
        let starts_at_the_value_end = vec![value.start..value.end, value.end..value.end + 2];
        assert_eq!(
            whole_links_edit(source, &runs_past_the_value).map(|range| range.end),
            Some(value.end + 1)
        );
        assert_eq!(
            whole_links_edit(source, &starts_at_the_value_end).map(|range| range.end),
            Some(value.end)
        );
    }
}
