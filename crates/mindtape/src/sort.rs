//! Default list ordering, shared by `list` (when the caller gives no `--sort`)
//! and by handle resolution.
//!
//! Handles map an ordinal to a task against "the default view", so that view
//! must order tasks identically to what `mindtape list` prints — otherwise the
//! ordinal a user reads off the list resolves to a different task. Both sides
//! call [`default_order`] to stay in lock-step.

use std::cmp::Ordering;

use crate::config::Config;
use crate::model::Task;
use crate::ready::{Readiness, ReadyState};

/// Sort `tasks` into the default order with no readiness signal (every task
/// settled). See [`default_order_with`].
pub fn default_order(tasks: &mut [Task], config: &Config) {
    default_order_with(tasks, config, &Readiness::empty());
}

/// Sort `tasks` into the default order: ready work first (ready, then blocked,
/// then settled per `readiness`), then status lifecycle (the configured
/// `[[tasks.status]]` index), then priority descending, then due ascending, with
/// the id as a stable final tiebreak. `readiness` must be computed over the full
/// loaded set (see [`Readiness::compute`]).
pub fn default_order_with(tasks: &mut [Task], config: &Config, readiness: &Readiness) {
    tasks.sort_by(|left, right| compare_default(left, right, config, readiness));
}

fn compare_default(left: &Task, right: &Task, config: &Config, readiness: &Readiness) -> Ordering {
    ready_rank(left, readiness)
        .cmp(&ready_rank(right, readiness))
        .then_with(|| status_index(left, config).cmp(&status_index(right, config)))
        .then_with(|| priority_desc(left, right, config))
        .then_with(|| due_asc(left, right))
        .then_with(|| left.id.cmp(&right.id))
}

/// Sort rank for a task's readiness: ready (0) before blocked (1) before settled
/// (2), so actionable work floats to the top of the default view.
#[must_use]
pub fn ready_rank(task: &Task, readiness: &Readiness) -> u8 {
    match readiness.state(task) {
        ReadyState::Ready => 0,
        ReadyState::Blocked => 1,
        ReadyState::Settled => 2,
    }
}

/// Position of a task's status in the configured lifecycle; unknown or missing
/// status sorts last.
#[must_use]
pub fn status_index(task: &Task, config: &Config) -> usize {
    match &task.status {
        Some(status) => config
            .tasks
            .status
            .entries
            .iter()
            .position(|entry| entry.value == status.name)
            .unwrap_or(usize::MAX),
        None => usize::MAX,
    }
}

/// A task's priority resolved to a number on its scale, if set and resolvable.
#[must_use]
pub fn priority_value(task: &Task, config: &Config) -> Option<f64> {
    task.priority
        .as_ref()
        .and_then(|value| config.tasks.priority.resolve(value))
}

fn priority_desc(left: &Task, right: &Task, config: &Config) -> Ordering {
    cmp_num_desc(priority_value(left, config), priority_value(right, config))
}

fn due_asc(left: &Task, right: &Task) -> Ordering {
    cmp_opt_asc(left.due.as_deref(), right.due.as_deref())
}

/// Compare numbers descending (higher first); a missing value sorts last.
fn cmp_num_desc(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(high), Some(low)) => low.partial_cmp(&high).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Compare options ascending with present values first; a missing value last.
fn cmp_opt_asc<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(early), Some(late)) => early.cmp(&late),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
#[path = "sort_tests.rs"]
mod tests;
