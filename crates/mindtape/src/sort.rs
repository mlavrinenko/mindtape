//! Default list ordering and the configurable sort comparator, shared by `list`
//! (when the caller gives no `--sort`), handle resolution, and the sort field
//! definitions used by the `--sort` flag.
//!
//! Handles map an ordinal to a task against "the default view", so that view
//! must order tasks identically to what `mindtape list` prints — otherwise the
//! ordinal a user reads off the list resolves to a different task. Both sides
//! call [`default_order_with`] (which honours `config.query.default_sort`) to
//! stay in lock-step.

use std::cmp::Ordering;

use crate::config::Config;
use crate::model::Task;
use crate::ready::{Readiness, ReadyState};

/// All sort field names accepted by `--sort` and `[query] default_sort`.
pub const SORT_FIELDS: [&str; 11] = [
    "id",
    "title",
    "status",
    "priority",
    "difficulty",
    "kind",
    "due",
    "start",
    "file",
    "depth",
    "ready",
];

/// Validate a single sort spec (`"field"` or `"field:asc"` / `"field:desc"`).
///
/// # Errors
/// Returns a message string if the field is not in [`SORT_FIELDS`].
pub fn validate_sort_spec(spec: &str) -> Result<(), String> {
    let field = spec.split_once(':').map_or(spec, |(f, _)| f);
    if SORT_FIELDS.contains(&field) {
        Ok(())
    } else {
        Err(format!("unknown sort field: {field}"))
    }
}

/// A task's difficulty resolved to a number on its scale, if set and resolvable.
#[must_use]
pub fn difficulty_value(task: &Task, config: &Config) -> Option<f64> {
    task.difficulty
        .as_ref()
        .and_then(|value| config.tasks.difficulty.resolve(value))
}

/// Sort `tasks` by `specs` using a stable least-significant-first multi-key sort.
/// Each spec is `"field"`, `"field:asc"`, or `"field:desc"`.
///
/// # Errors
/// Returns a message string if any spec names an unknown field.
pub fn sort_by_specs(
    tasks: &mut [Task],
    specs: &[String],
    config: &Config,
    readiness: &Readiness,
) -> Result<(), String> {
    for spec in specs.iter().rev() {
        let (field, descending) = parse_spec(spec);
        if !SORT_FIELDS.contains(&field) {
            return Err(format!("unknown sort field: {field}"));
        }
        tasks.sort_by(|l, r| {
            let ord = compare_field(l, r, field, config, readiness);
            if descending { ord.reverse() } else { ord }
        });
    }
    Ok(())
}

fn parse_spec(spec: &str) -> (&str, bool) {
    match spec.split_once(':') {
        Some((field, "desc")) => (field, true),
        Some((field, _)) => (field, false),
        None => (spec, false),
    }
}

/// Compare two tasks on `field` ascending; callers reverse for descending specs.
fn compare_field(
    left: &Task,
    right: &Task,
    field: &str,
    config: &Config,
    readiness: &Readiness,
) -> Ordering {
    match field {
        "status" => status_index(left, config).cmp(&status_index(right, config)),
        "ready" => ready_rank(left, readiness).cmp(&ready_rank(right, readiness)),
        "priority" => cmp_num_asc(priority_value(left, config), priority_value(right, config)),
        "difficulty" => cmp_num_asc(
            difficulty_value(left, config),
            difficulty_value(right, config),
        ),
        "depth" => left.depth.cmp(&right.depth),
        _ => sort_string_key(left, field).cmp(&sort_string_key(right, field)),
    }
}

fn cmp_num_asc(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(low), Some(high)) => low.partial_cmp(&high).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn sort_string_key(task: &Task, field: &str) -> String {
    match field {
        "id" => task.id.clone(),
        "title" => task.title.clone(),
        "kind" => task.kind.clone().unwrap_or_default(),
        "due" => task.due.clone().unwrap_or_default(),
        "start" => task.start.clone().unwrap_or_default(),
        "file" => task.file_key(),
        _ => String::new(),
    }
}

/// Sort `tasks` into the default order with no readiness signal (every task
/// settled). See [`default_order_with`].
pub fn default_order(tasks: &mut [Task], config: &Config) {
    default_order_with(tasks, config, &Readiness::empty());
}

/// Sort `tasks` into the default order.
///
/// When `config.query.default_sort` is set, that chain (validated at load) is
/// used via the shared [`sort_by_specs`] comparator. Otherwise the shipped
/// built-in chain applies: ready work first (ready → blocked → settled), then
/// priority descending, status lifecycle, due ascending, id as stable tiebreak.
///
/// `readiness` must be computed over the full loaded set (see
/// [`Readiness::compute`]); pass [`Readiness::empty`] when no set context applies.
pub fn default_order_with(tasks: &mut [Task], config: &Config, readiness: &Readiness) {
    if !config.ls.default_sort.is_empty() {
        // Validated at config load — errors here are unreachable.
        let _ = sort_by_specs(tasks, &config.ls.default_sort, config, readiness);
        return;
    }
    tasks.sort_by(|left, right| compare_default(left, right, config, readiness));
}

fn compare_default(left: &Task, right: &Task, config: &Config, readiness: &Readiness) -> Ordering {
    ready_rank(left, readiness)
        .cmp(&ready_rank(right, readiness))
        .then_with(|| priority_desc(left, right, config))
        .then_with(|| status_index(left, config).cmp(&status_index(right, config)))
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
