//! The `unlink` strategy: delete the eligible task files and strip every
//! inbound edge that references one from the surviving files. No git, no
//! tombstones — split out of `rm_gitless.rs` to stay under the file-size gate.

use std::collections::HashSet;
use std::ops::Range;
use std::path::Path;

use anyhow::{Context, Result};
use mindtape::{Task, normalize_link};
use typst_cleanup::{Plan, Rewrite};

use crate::task_edit::task_with;

use super::plain_delete_plan;

/// The `unlink` strategy's plan: delete the eligible task files, and strip
/// every inbound edge that references one from the surviving files. No git,
/// no tombstones.
///
/// # Errors
/// A survivor file that cannot be read, or a stripped source that cannot be
/// spliced back together, fails the run.
pub fn unlink_plan(tasks: &[Task], eligible: &HashSet<String>, root: &Path) -> Result<Plan> {
    let mut rewrites: Vec<Rewrite> = Vec::new();
    for task in tasks {
        if eligible.contains(&task.id) {
            continue;
        }
        let abs = root.join(&task.file);
        let source = std::fs::read_to_string(&abs)
            .with_context(|| format!("read {}", task.file.display()))?;
        let (ranges, links) = unlink_edits(&source, eligible);
        if ranges.is_empty() {
            continue;
        }
        let edits: Vec<typst_edit::Edit> = ranges
            .into_iter()
            .map(|range| typst_edit::Edit::new(range, String::new()))
            .collect();
        let contents =
            typst_edit::apply(&source, edits).map_err(|err| anyhow::anyhow!(err.to_string()))?;
        rewrites.push(Rewrite {
            abs,
            rel: task.file.clone(),
            contents,
            links,
        });
    }
    rewrites.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(Plan {
        deletions: plain_delete_plan(tasks, eligible, root).deletions,
        rewrites,
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
