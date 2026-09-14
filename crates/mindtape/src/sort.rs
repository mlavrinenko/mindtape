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
use crate::fields::{self, FieldValue};
use crate::model::Task;
use crate::ready::{Readiness, ReadyState};

/// Whether `name` is accepted by `--sort` and `[query] default_sort`: every
/// [`crate::fields::FIELDS`] entry, plus `ready` (a computed rank, not a
/// `Task` field — see [`ready_rank`]). Derived rather than listed, so a field
/// or property wired into the registry is sortable by construction and no
/// second list has to remember it — the drift that left `assignee` and
/// `reviewed` silently unsortable before the registry existed.
///
/// Two of them read the dependency graph rather than the task's own
/// properties: `topo` (dependency level — ascending is a topological order)
/// and `blocks` (how many tasks it transitively unblocks). Both are levels,
/// not linear positions, so they tie and a following spec breaks the tie:
/// `--sort topo --sort priority:desc` is graph order with priority inside a
/// layer. See [`crate::dep_rank`].
#[must_use]
pub fn is_sort_field(name: &str) -> bool {
    name == "ready" || fields::get(name).is_some()
}

/// Validate a single sort spec (`"field"` or `"field:asc"` / `"field:desc"`).
///
/// # Errors
/// Returns a message string if [`is_sort_field`] rejects the field.
pub fn validate_sort_spec(spec: &str) -> Result<(), String> {
    let field = spec.split_once(':').map_or(spec, |(f, _)| f);
    if is_sort_field(field) {
        Ok(())
    } else {
        Err(unknown_sort_field(field))
    }
}

/// The `--sort` / `[query] default_sort` error for an unrecognised field,
/// carrying [`crate::fields::rename_hint`]'s clause when the name is one the
/// registry retired — so `--sort file` names `path` rather than only saying
/// `file` does not exist.
fn unknown_sort_field(field: &str) -> String {
    format!("unknown sort field: {field}{}", fields::rename_hint(field))
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
        if !is_sort_field(field) {
            return Err(unknown_sort_field(field));
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
///
/// `status`/`ready`/`priority`/`difficulty` need config- or readiness-aware
/// ordering that a raw field value can't express (legend position, readiness
/// band, a resolved numeric scale point) so they stay special-cased; every
/// other field — including any added to [`crate::fields::FIELDS`] later —
/// falls through to the shared registry.
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
        _ => cmp_field_value(
            &registry_value(left, field, config),
            &registry_value(right, field, config),
        ),
    }
}

/// A field's value via the shared [`crate::fields`] registry; `Missing` when
/// `field` isn't registered there — unreachable via `--sort`, since
/// `sort_by_specs` validates every spec first and the one sort name with no
/// registry entry, `ready`, is intercepted in [`compare_field`].
fn registry_value(task: &Task, field: &str, config: &Config) -> FieldValue {
    fields::get(field).map_or(FieldValue::Missing, |def| (def.get)(task, config))
}

/// Compare two resolved field values for `--sort`, ascending: numbers
/// numerically, strings lexicographically (ISO-correct for dates), a missing
/// value after every present one — the same missing-last convention
/// [`cmp_num_asc`] and [`cmp_opt_asc`] already use, so every field orders
/// missing values the same way (`sort_by_specs` reverses the whole comparison
/// for `:desc`, so there a missing value sorts first — same as `priority`/
/// `difficulty`, whose `cmp_num_asc` arms follow the identical rule). The two
/// values passed in are always resolved from the same field name, so they are
/// always the same variant when both are present.
fn cmp_field_value(left: &FieldValue, right: &FieldValue) -> Ordering {
    match (left, right) {
        (FieldValue::Missing, FieldValue::Missing) => Ordering::Equal,
        (FieldValue::Missing, _) => Ordering::Greater,
        (_, FieldValue::Missing) => Ordering::Less,
        (FieldValue::Num(low), FieldValue::Num(high)) => {
            low.partial_cmp(high).unwrap_or(Ordering::Equal)
        }
        (FieldValue::Str(left), FieldValue::Str(right)) => left.cmp(right),
        _ => Ordering::Equal,
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

#[cfg(test)]
#[path = "sort_compare_tests.rs"]
mod tests_compare;
