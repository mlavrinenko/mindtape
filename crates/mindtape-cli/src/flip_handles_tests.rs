//! Unit tests for the flip handle report's one decision: given a task's
//! handle on each side of the batch, what line does it earn.
//!
//! `move_line` is the whole policy — the four scope bullets of
//! `truth/tasks/print-the-refreshed-handle-after-a-flip.typ` reduce to it — and
//! it is pure, so it is pinned here rather than through a spawned `mt`. That
//! the two sides are actually the pre- and post-write default views, and that
//! the line reaches stdout, is `tests/flip_handle_e2e.rs`'s half.
#![allow(clippy::unwrap_used)]

use super::move_line;

/// The reported case: an ordinal that moved under a caller still holding the
/// old one. Both handles appear, so the line answers "what do I type now"
/// without a second command.
#[test]
fn a_moved_ordinal_reports_both_sides() {
    let line = move_line("alpha", Some("1o"), Some("3o")).unwrap();
    assert_eq!(line, "handle alpha: 1o → 3o");
}

/// The silent case, and the common one: a same-status re-stamp leaves the view
/// untouched. Noise here would train the reader to skip the line that matters.
#[test]
fn an_unmoved_handle_says_nothing() {
    assert_eq!(move_line("alpha", Some("1o"), Some("1o")), None);
}

/// A task the flip filtered OUT of the default view — `mt done` under the
/// config floor's `!terminal` filter — has no ordinal left to print, so the
/// line names the filename stem, which still addresses it.
#[test]
fn leaving_the_view_names_the_stem() {
    let line = move_line("alpha", Some("1o"), None).unwrap();
    assert!(
        line.starts_with("handle alpha: 1o → none"),
        "expected the stem and the vacated handle: {line}"
    );
    assert!(
        line.contains("left the default view") && line.contains("stem still addresses it"),
        "expected the line to say what still addresses the task: {line}"
    );
}

/// The mirror: a flip that pulls a task back INTO the view hands out a handle
/// the caller had no way to know. `none` on the left keeps the shape uniform.
#[test]
fn entering_the_view_reports_the_new_handle() {
    let line = move_line("alpha", None, Some("2o")).unwrap();
    assert_eq!(line, "handle alpha: none → 2o (now in the default view)");
}

/// A task outside the default view on both sides never had a handle to go
/// stale, so it gets the same silence an unmoved one does — the report is
/// about a handle that changed, not about every task the batch touched.
#[test]
fn absent_on_both_sides_says_nothing() {
    assert_eq!(move_line("alpha", None, None), None);
}
