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
//! Split out of `rm_gitless.rs` to stay under the file-size gate; `mt mv`
//! reaches it through `rm`'s re-exports rather than owning a second copy.

use std::collections::HashSet;
use std::ops::Range;
use std::path::Path;

use anyhow::{Context, Result};
use mindtape::{Task, normalize_link};
use typst_cleanup::{Plan, Rewrite};

use crate::task_edit::task_with;

use super::plain_delete_plan;

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
        let source = std::fs::read_to_string(&abs)
            .with_context(|| format!("read {}", task.file.display()))?;
        let (edits, links) = inbound_edits(&source, matched, to);
        if links == 0 {
            continue;
        }
        let contents = typst_edit::apply(&source, edits)
            .map_err(|err| anyhow::anyhow!(err.to_string()))
            .with_context(|| format!("rewrite links in {}", task.file.display()))?;
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
        deletions: plain_delete_plan(tasks, eligible, root).deletions,
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
            let mut cursor = end;
            while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
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
    let mut cursor = start;
    while cursor > 0 && bytes.get(cursor - 1).is_some_and(u8::is_ascii_whitespace) {
        cursor -= 1;
    }
    if cursor > 0 && bytes.get(cursor - 1) == Some(&b'+') {
        let mut back = cursor - 1;
        while back > 0 && bytes.get(back - 1).is_some_and(u8::is_ascii_whitespace) {
            back -= 1;
        }
        start = back;
    }
    let mut end = range.end;
    let mut ahead = end;
    while bytes.get(ahead).is_some_and(u8::is_ascii_whitespace) {
        ahead += 1;
    }
    if bytes.get(ahead) == Some(&b'+') {
        let mut forward = ahead + 1;
        while bytes.get(forward).is_some_and(u8::is_ascii_whitespace) {
            forward += 1;
        }
        end = forward;
    }
    Range { start, end }
}

#[cfg(test)]
mod tests {
    use super::retarget_raw;

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
}
