//! What a flip did to the handles — the loop `flip` used to leave open.
//!
//! A handle (`mt ls`'s `#` column) is `<ordinal><check>`, and the ordinal is
//! the task's position in the default view, recomputed on every command rather
//! than handed out by the last listing. A flip re-sorts and re-filters that
//! view, so the handle the caller just typed is stale the moment the write
//! lands. Nothing else `flip` prints says so; the caller found out on the next
//! command, from a check mismatch naming a task they never meant.
//!
//! So the batch closes the loop where it opened. [`Handles::capture`] runs
//! twice — once before the writes, once after — and the batch prints one line
//! per landed flip whose handle actually moved. Twice, not once per target:
//! an ordinal is a position in ONE list, so a per-target recomputation would
//! buy nothing but N-1 redundant view builds, and the second capture rides the
//! project reload the post-flip revalidation already pays for.
//!
//! What "moved" means at the two edges, both decided here rather than left to
//! the caller:
//!
//! - A task the flip filtered OUT of the default view has no ordinal at all —
//!   `mt done` under the config floor's `!terminal` filter, so the common case
//!   rather than an edge. That is the loudest move there is, so it is reported,
//!   and the line names the filename stem: the handle is gone, the stem still
//!   addresses the task. A task filtered IN is the mirror, and reads the same.
//! - A task with no handle on either side never had one to go stale, so it is
//!   silent — the same silence a same-status re-stamp gets, and for the same
//!   reason: a line that says nothing changed teaches the reader to skip the
//!   line that says something did.
//!
//! A multi-target batch prints one line per moved target, in the order the
//! targets were given, each naming its own stem. The stem is the key rather
//! than the spec the caller typed, because that spec may itself be a handle —
//! one of the handles this batch just invalidated — and keying the answer on
//! the stale question is what the report exists to stop.

use std::collections::HashMap;

use anyhow::Result;

use crate::handle::{check_len, make_handles};
use crate::util::{Project, default_view};

/// Every default-view handle in a project, keyed by task id (the filename
/// stem). A task absent from the map is absent from the view.
pub(crate) struct Handles(HashMap<String, String>);

impl Handles {
    /// Build the default view once and index its handles by task id.
    ///
    /// The view is [`default_view`]'s — the same `[[ls.rules]]` cascade,
    /// filter and sort bare `mt ls` resolves — so the handles reported here
    /// are the handles the caller's next command will read off the list, not
    /// a second guess at them.
    ///
    /// # Errors
    /// Returns an error if the resolved filter or sort spec fails to parse.
    pub(crate) fn capture(project: &Project) -> Result<Self> {
        let tasks = default_view(project)?;
        Ok(Self(
            make_handles(&tasks, check_len(&project.config))
                .into_iter()
                .map(|(handle, task)| (task.id.clone(), handle))
                .collect(),
        ))
    }

    /// The handle task `id` carries in this view, or `None` when the view
    /// does not list it.
    fn get(&self, id: &str) -> Option<&str> {
        self.0.get(id).map(String::as_str)
    }
}

/// [`Handles::capture`], downgraded to a warning on failure.
///
/// A flip that already landed on disk must not be reported as failed because
/// the courtesy line behind it could not be computed, and it must not go into
/// the batch's problem list either — that list is what the exit status counts.
/// Warn and print nothing.
pub(crate) fn snapshot(project: &Project) -> Option<Handles> {
    match Handles::capture(project) {
        Ok(handles) => Some(handles),
        Err(err) => {
            eprintln!("warning: cannot recompute handles: {err:#}");
            None
        }
    }
}

/// Print the handle news for a landed batch: `before` is the pre-write view,
/// `after` the reloaded project, and `ids` the task ids that landed, in the
/// order their targets were given.
pub(crate) fn report(before: &Handles, after: &Project, ids: &[String]) {
    let Some(after) = snapshot(after) else { return };
    for id in ids {
        if let Some(line) = move_line(id, before.get(id), after.get(id)) {
            println!("{line}");
        }
    }
}

/// The one line task `id` earns, or `None` when its handle did not move.
///
/// Shape is uniform — `handle <stem>: <was> → <now>` — so a batch reads as a
/// column and the absent side is spelled `none` rather than omitted; a line
/// with a missing half would be read as a formatting slip, not as a fact.
fn move_line(id: &str, was: Option<&str>, now: Option<&str>) -> Option<String> {
    if was == now {
        return None;
    }
    match (was, now) {
        (Some(was), Some(now)) => Some(format!("handle {id}: {was} → {now}")),
        (None, Some(now)) => Some(format!(
            "handle {id}: none → {now} (now in the default view)"
        )),
        (Some(was), None) => Some(format!(
            "handle {id}: {was} → none (left the default view; the stem still addresses it)"
        )),
        // `was == now` already returned, so both-absent cannot reach here.
        (None, None) => None,
    }
}

#[cfg(test)]
#[path = "flip_handles_tests.rs"]
mod tests;
