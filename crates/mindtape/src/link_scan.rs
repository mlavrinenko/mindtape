//! Parse-plane scan for a task's outbound link targets — the byte spans that
//! name another task by string literal, across both `link("…")` calls and the
//! `links:` edge helpers (`depends-on(...)`, `related(...)`, `edge(kind, ...)`, …).
//!
//! The cleanup path (`mt rm`) rewrites a retired task's inbound references into
//! SWHID tombstones. It locates them here on the parse plane (no eval, no
//! `World`), so it can splice the exact quoted string literal and leave
//! everything else — notes, sibling edges, prose — byte-for-byte intact.
//!
//! Unlike [`crate::parse_links`], which projects the *evaluated* `links:` value
//! into the model, this reads *source text* and reports ranges: a rewriter needs
//! spans, not values. And unlike a bare `typst_edit::find_link_targets` (which
//! sees only `link("…")`), it also covers the edge helpers, whose targets are
//! plain stem strings authored without `link(...)` — the exact gap that let
//! retiring a task leave inbound `links:` edges dangling.

use std::ops::Range;

use typst_edit::{Arg, find_calls, find_link_targets};

/// A located link target: the raw string as authored (e.g. `"other.typ"`,
/// unquoted and unescaped) and the byte range of its quoted string literal
/// (quotes included), ready to feed to `typst_edit::Edit::new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetSpan {
    /// Raw target string, unquoted and unescaped, as authored.
    pub raw: String,
    /// Byte range of the `"…"` literal in the source, quotes included.
    pub range: Range<usize>,
}

/// The `links:` edge helpers whose string arguments name a target task, mirroring
/// `lib/prelude.typ`. `edge` is the generic escape hatch for a custom kind; its
/// leading `kind` string is not a target and is skipped (see [`kind_args`]).
const EDGE_HELPERS: &[&str] = &[
    "depends-on",
    "parent",
    "recurred-from",
    "supersedes",
    "duplicates",
    "related",
    "edge",
];

/// How many leading positional arguments of `helper` are not targets: `edge`'s
/// first positional is the edge *kind*, so retiring a task named after a kind
/// never corrupts an `edge("kind", …)` call. The named helpers take targets from
/// the first positional onward.
fn kind_args(helper: &str) -> usize {
    usize::from(helper == "edge")
}

/// Every string-literal task target in `source`, across `link("…")` calls and
/// the `links:` edge helpers. Non-literal arguments (a computed value, a note
/// content block) never appear — only quoted literals are rewritable targets.
///
/// Order is not significant: the cleanup path maps each raw target to a retired
/// identity and hands the spans to `typst_edit::apply`, which sorts and checks
/// for overlap.
#[must_use]
pub fn link_target_spans(source: &str) -> Vec<TargetSpan> {
    let mut spans: Vec<TargetSpan> = find_link_targets(source)
        .into_iter()
        .map(|target| TargetSpan {
            raw: target.url,
            range: target.range,
        })
        .collect();
    for &helper in EDGE_HELPERS {
        let skip = kind_args(helper);
        for call in find_calls(source, helper) {
            let targets = call
                .args
                .iter()
                .filter(|arg| arg.name.is_none())
                .skip(skip)
                .filter_map(|arg| string_span(source, arg));
            spans.extend(targets);
        }
    }
    spans
}

/// A positional argument's span, kept only when it is a string literal (its
/// value bytes open with a `"`). A non-literal target — a variable, a computed
/// value — is not rewritable and is dropped; the prelude helpers only ever take
/// plain strings.
fn string_span(source: &str, arg: &Arg) -> Option<TargetSpan> {
    (source.as_bytes().get(arg.value_range.start) == Some(&b'"')).then(|| TargetSpan {
        raw: arg.value.clone(),
        range: arg.value_range.clone(),
    })
}

/// A located edge helper call: the helper name, the whole call's byte range
/// (a trailing note content block included), and its string-literal targets.
/// Enough to strip an inbound edge wholesale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeCall {
    /// The helper name (`depends-on`, `related`, `edge`, …).
    pub name: String,
    /// Byte range of the whole call in the source, note block included.
    pub range: Range<usize>,
    /// The call's string-literal targets (the `edge` kind argument excluded).
    pub targets: Vec<TargetSpan>,
}

/// Every edge helper call in `source`, in document order. The `unlink` rm
/// strategy's view: whole calls (not just target spans), so an inbound edge
/// can be stripped in one edit.
#[must_use]
pub fn link_edges(source: &str) -> Vec<EdgeCall> {
    let mut calls: Vec<EdgeCall> = EDGE_HELPERS
        .iter()
        .flat_map(|&helper| {
            let skip = kind_args(helper);
            find_calls(source, helper)
                .into_iter()
                .map(move |call| EdgeCall {
                    name: call.name.clone(),
                    range: call.range.clone(),
                    targets: call
                        .args
                        .iter()
                        .filter(|arg| arg.name.is_none())
                        .skip(skip)
                        .filter_map(|arg| string_span(source, arg))
                        .collect(),
                })
        })
        .collect();
    calls.sort_by(|a, b| a.range.start.cmp(&b.range.start));
    calls
}

#[cfg(test)]
#[path = "link_scan_tests.rs"]
mod tests;
