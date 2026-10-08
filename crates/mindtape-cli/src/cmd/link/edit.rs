//! `mt link`'s pure half: render edges as prelude helper calls, and reconcile
//! them with the `links:` a task already carries.
//!
//! No IO and no project here — a source string, the task's evaluated edges, and
//! the edges the caller asked for go in; one [`typst_edit::Edit`] comes out (or
//! `None`, when the file already says this). The same `task_edit::{task_with,
//! upsert_named_arg}` + `typst_edit::apply` shape `cmd/flip/mod.rs` and `cmd/recur/edit.rs`
//! use to rewrite a single named argument.

use std::collections::{HashMap, HashSet};

use crate::diag::Result;
use mindtape::link::{Link, is_builtin_kind};
use mindtape::{escape_markup, is_history_pointer, normalize_link};
use typst_edit::Edit;

use crate::diag::{Diag, Hint};
use crate::task_edit::{task_with, upsert_named_arg};

/// One edge to write: its kind, its target as the caller authored it, and an
/// optional note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeSpec {
    /// Edge kind — a built-in or a configured `[[tasks.links.kinds]]` name.
    pub kind: String,
    /// Target, as the literal string that goes into the file. The planner
    /// settles the `.typ` extension before an edge reaches here
    /// ([`crate::cmd::link::plan`]), because whether a target *has* one is a
    /// property of the kind, not of the edge: a `local-task-path` kind's
    /// target is a task file, while a kind pointing outside the backlog names
    /// a URL or a commit that appending `.typ` would corrupt.
    pub target: String,
    /// Free-form note, rendered as the helper's trailing content block.
    pub note: Option<String>,
}

impl EdgeSpec {
    /// The identity two edges are deduped on: the kind plus the target's
    /// filename stem, so `other` and `other.typ` are one edge and re-running
    /// the same command writes nothing the second time.
    fn key(&self) -> (&str, String) {
        (self.kind.as_str(), normalize_link(&self.target))
    }
}

/// How a run reconciles the edges it was given with the ones already there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Add what is missing; leave everything else alone (the default).
    Merge,
    /// Make the given edges the task's complete edge set (`--replace`).
    Replace,
}

/// One task's pending rewrite: the edit itself, plus what it does, for the
/// plan/report line.
#[derive(Debug)]
pub struct LinkEdit {
    /// The `links:` argument rewrite, ready for `typst_edit::apply`.
    pub edit: Edit,
    /// Edges this write adds that the task did not already carry.
    pub added: Vec<EdgeSpec>,
    /// Edges the task carried that this write drops (`--replace` only).
    pub removed: usize,
}

/// Render `edges` as a `links:` value built from the prelude's own typed
/// helpers, `+`-joined.
///
/// A built-in kind uses its named helper (`depends-on("a.typ")`); every other
/// configured kind goes through the generic `edge("kind", "a.typ")`, because
/// the prelude binds one helper per built-in and nothing per custom kind — a
/// custom kind written as a bare call would be an unknown identifier and the
/// file would not eval.
///
/// Consecutive note-less edges of one kind collapse into a single variadic
/// call, which is the shape the DSL page documents and a human writes. A noted
/// edge always stands alone: the trailing block is the note for *every* target
/// in its call, so folding a noted edge into a run would attach its note to
/// the others.
#[must_use]
pub fn render_edges(edges: &[EdgeSpec]) -> String {
    let mut calls: Vec<String> = Vec::new();
    let mut run: Vec<&EdgeSpec> = Vec::new();
    for edge in edges {
        let extends = edge.note.is_none()
            && run
                .first()
                .is_some_and(|first| first.kind == edge.kind && first.note.is_none());
        if !extends {
            flush(&mut run, &mut calls);
        }
        run.push(edge);
    }
    flush(&mut run, &mut calls);
    calls.join(" + ")
}

/// Render the accumulated run as one helper call and clear it.
fn flush(run: &mut Vec<&EdgeSpec>, out: &mut Vec<String>) {
    if let Some(first) = run.first() {
        let targets: Vec<String> = run.iter().map(|edge| quote(&edge.target)).collect();
        let args = targets.join(", ");
        let mut call = if is_builtin_kind(&first.kind) {
            format!("{}({args})", first.kind)
        } else {
            format!("edge({}, {args})", quote(&first.kind))
        };
        if let Some(note) = first
            .note
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty())
        {
            call.push_str(&format!("[{}]", escape_markup(note)));
        }
        out.push(call);
    }
    run.clear();
}

/// Quote `text` as a Typst string literal. Backslash and the closing quote are
/// the only two characters that can end the literal early; a stem never holds
/// either, which is exactly why escaping them costs nothing and forgetting to
/// would fail loud only on the one target that does.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        if ch == '\\' || ch == '"' {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

/// The rewrite one task needs to carry `desired`, given the edges it already
/// holds (`existing` — the evaluated `links:` value).
///
/// [`Mode::Merge`] appends only the edges `existing` lacks and leaves the
/// authored value byte-for-byte alone. That asymmetry is deliberate: a note in
/// the source is Typst content and the model hands it back as plain text, so
/// re-rendering an edge the caller never named would quietly flatten its
/// markup. [`Mode::Replace`] writes `desired` as the whole value and accepts
/// that cost, which is what it was asked for.
///
/// `Ok(None)` means the file already says this — a merge with nothing new, or
/// a replace whose set and notes already match — so re-runs are no-ops.
///
/// # Errors
/// Returns an error when `source` holds no `task.with(…)` call, or when a
/// replace would drop a SWHID history pointer (see [`refuse_tombstones`]).
pub fn links_edit(
    source: &str,
    existing: &[Link],
    desired: &[EdgeSpec],
    mode: Mode,
) -> Result<Option<LinkEdit>> {
    let call = task_with(source)?;
    let have: HashSet<(&str, String)> = existing
        .iter()
        .map(|link| (link.kind.as_str(), normalize_link(&link.task)))
        .collect();
    let added: Vec<EdgeSpec> = desired
        .iter()
        .filter(|edge| !have.contains(&edge.key()))
        .cloned()
        .collect();

    let (value, removed) = match mode {
        Mode::Merge => {
            if added.is_empty() {
                return Ok(None);
            }
            let rendered = render_edges(&added);
            let value = match call.named("links") {
                Some(arg) => format!("{} + {rendered}", arg.value),
                None => rendered,
            };
            (value, 0)
        }
        Mode::Replace => {
            refuse_tombstones(source)?;
            let want: HashSet<(&str, String)> = desired.iter().map(EdgeSpec::key).collect();
            let removed = existing
                .iter()
                .filter(|link| !want.contains(&(link.kind.as_str(), normalize_link(&link.task))))
                .count();
            if added.is_empty() && removed == 0 && notes_match(existing, desired) {
                return Ok(None);
            }
            // An empty edge set is a legitimate `--replace` outcome ("this task
            // links to nothing"), and the empty array says so in the DSL's own
            // vocabulary — deleting the argument would need a second, unrelated
            // rewrite of the call's separators to avoid leaving a dangling key.
            let rendered = if desired.is_empty() {
                "()".to_owned()
            } else {
                render_edges(desired)
            };
            (rendered, removed)
        }
    };

    Ok(Some(LinkEdit {
        edit: upsert_named_arg(source, &call, "links", &value)?,
        added,
        removed,
    }))
}

/// Whether every edge in `desired` that `existing` already carries carries the
/// same note. Without this a `--replace` that only rewords a note would report
/// itself a no-op: the key sets match, so nothing else in the comparison moves.
/// A missing note and a blank one are the same absence, matching what
/// `render_edges` writes and what the note rules read.
fn notes_match(existing: &[Link], desired: &[EdgeSpec]) -> bool {
    let by_key: HashMap<(&str, String), Option<&str>> = existing
        .iter()
        .map(|link| {
            (
                (link.kind.as_str(), normalize_link(&link.task)),
                note_text(link.note.as_deref()),
            )
        })
        .collect();
    desired.iter().all(|edge| match by_key.get(&edge.key()) {
        Some(note) => *note == note_text(edge.note.as_deref()),
        None => true,
    })
}

/// A note's comparable text: trimmed, with blank read as absent.
fn note_text(note: Option<&str>) -> Option<&str> {
    note.map(str::trim).filter(|text| !text.is_empty())
}

/// Refuse a `--replace` on a task whose `links:` holds a SWHID history pointer
/// — the tombstone `mt rm` mints in place of a retired task.
///
/// A tombstone names no loaded task, so `parse_links` drops it and it is absent
/// from `existing`; rewriting the value from `existing` alone would erase the
/// provenance `mt rm` went to a git round trip to record, with nothing
/// reporting the loss. Refusing is the honest answer: merge still works on such
/// a file, and so does editing it by hand.
///
/// # Errors
/// Returns an error naming the first pointer found.
fn refuse_tombstones(source: &str) -> Result<()> {
    for call in mindtape::link_edges(source) {
        for target in &call.targets {
            if is_history_pointer(&target.raw) {
                return Err(Diag::refused("--replace would drop a history pointer")
                    .value(target.raw.as_str())
                    .hint(Hint::new(
                        "merge instead by omitting --replace, or edit the file by hand",
                    )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
