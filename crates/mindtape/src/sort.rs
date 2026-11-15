//! Default list ordering and the configurable sort comparator, shared by `list`
//! (when the caller gives no `--sort`) and the sort field definitions used by
//! the `--sort` flag. [`default_order_with`] honours `config.ls.default_sort`.

use std::cmp::Ordering;

use crate::config::{CollectorSpec, Config};
use crate::graph::ready::{Readiness, ReadyState};
use crate::model::Task;
use crate::model::fields::{self as fields, FieldValue};
use crate::problem::Problem;
use crate::suggest::Unknown;

/// Whether `--sort` and `[ls] default-sort` accept `name`: every
/// [`crate::model::fields::FIELDS`] entry, every `x-<ns>` extension slot, plus
/// `ready` (a computed rank, not a `Task` field, see [`ready_rank`]). A scalar
/// value orders like a core field. A list or a dict orders as missing.
///
/// The registry derives the list rather than storing it, so a field or property
/// wired into the registry is sortable by construction. No second list has to
/// remember it, which is the drift that left `assignee` and `reviewed` silently
/// unsortable before the registry existed.
///
/// Two of them read the dependency graph rather than the task's own
/// properties: `topo` (dependency level, ascending is a topological order)
/// and `blocks` (how many tasks it transitively unblocks). Both are levels,
/// not linear positions, so they tie and a following spec breaks the tie:
/// `--sort topo --sort priority:desc` is graph order with priority inside a
/// layer. See [`crate::graph::dep_rank`].
#[must_use]
pub fn is_sort_field(name: &str) -> bool {
    name == "ready" || fields::get(name).is_some() || fields::is_extra(name)
}

/// [`is_sort_field`] over a project's collectors too: a scalar derived field
/// sorts, while a list or record does not.
#[must_use]
pub fn is_sort_field_with(name: &str, collectors: &[CollectorSpec]) -> bool {
    if fields::is_derived(name) {
        return fields::derived_type_in(collectors, name)
            .is_some_and(crate::config::CollectorType::is_scalar);
    }
    is_sort_field(name)
}

/// Validate a single sort spec (`"field"`, `"field:asc"`, or `"field:desc"`).
///
/// # Errors
/// Returns a message string if [`is_sort_field`] rejects the field, or if a
/// `:` suffix is anything but `asc`/`desc`. The function refuses a misspelled
/// direction rather than reading it as ascending.
pub fn validate_sort_spec(spec: &str) -> Result<(), Problem> {
    validate_sort_spec_with(spec, &[])
}

/// [`validate_sort_spec`] over a project's collectors too. The function
/// refuses a derived list or record with its type, since only a scalar orders.
///
/// # Errors
/// As [`validate_sort_spec`].
pub fn validate_sort_spec_with(spec: &str, collectors: &[CollectorSpec]) -> Result<(), Problem> {
    let (field, direction) = spec
        .split_once(':')
        .map_or((spec, None), |(field, direction)| (field, Some(direction)));
    if fields::is_derived(field) {
        match fields::derived_type_in(collectors, field) {
            Some(ty) if ty.is_scalar() => {}
            Some(ty) => {
                return Err(Problem::new(format!(
                    "sort field {} is {}; only a scalar sorts",
                    crate::config::quote(field),
                    ty.describe()
                )));
            }
            None => return Err(unknown_sort_field(field)),
        }
    } else if !is_sort_field(field) {
        return Err(unknown_sort_field(field));
    }
    match direction {
        None | Some("asc" | "desc") => Ok(()),
        Some(other) => Err(Problem::new(format!(
            "unknown sort direction: {}: use asc or desc",
            crate::config::quote(other)
        ))),
    }
}

/// The core field names `--sort` accepts besides the `x-<ns>` slots: every
/// registry field, then `ready`. What a did-you-mean for a misspelled field
/// searches.
#[must_use]
pub fn sort_fields() -> Vec<&'static str> {
    let mut names: Vec<&str> = fields::FIELDS.iter().map(|def| def.name).collect();
    names.push("ready");
    names
}

/// [`sort_fields`] plus every scalar derived field's name, for a did-you-mean
/// that can suggest one.
#[must_use]
pub fn sort_fields_with(collectors: &[CollectorSpec]) -> Vec<String> {
    let mut names: Vec<String> = sort_fields().into_iter().map(str::to_owned).collect();
    names.extend(
        collectors
            .iter()
            .filter(|spec| spec.returns.is_scalar())
            .map(|spec| format!("derived.{}", spec.name)),
    );
    names
}

/// The `--sort` / `[ls] default-sort` error for an unrecognised field.
fn unknown_sort_field(field: &str) -> Problem {
    Problem::new(format!(
        "unknown sort field: {}",
        crate::config::quote(field)
    ))
    .with(Unknown::near(field, &sort_fields()))
}

/// A task's difficulty resolved to a number on its scale, if set and resolvable.
#[must_use]
pub fn difficulty_value(task: &Task, config: &Config) -> Option<f64> {
    task.difficulty
        .as_ref()
        .and_then(|value| config.tasks.difficulty.resolve(value))
}

/// Sort `tasks` by `specs` with a stable multi-key sort that applies the last
/// spec first. Each spec is `"field"`, `"field:asc"`, or `"field:desc"`.
///
/// # Errors
/// Returns a message string if any spec references an unknown field.
pub fn sort_by_specs(
    tasks: &mut [Task],
    specs: &[String],
    config: &Config,
    readiness: &Readiness,
) -> Result<(), String> {
    for spec in specs.iter().rev() {
        validate_sort_spec_with(spec, &config.tasks.collectors).map_err(|problem| problem.text)?;
        let (field, descending) = parse_spec(spec);
        tasks.sort_by(|l, r| {
            let ord = compare_field(l, r, field, config, readiness);
            if descending { ord.reverse() } else { ord }
        });
    }
    Ok(())
}

/// Split a validated spec into its field and direction. Every caller runs
/// [`validate_sort_spec`] first, so a suffix here is `asc` or `desc`. The
/// wildcard arm only keeps the function total.
fn parse_spec(spec: &str) -> (&str, bool) {
    match spec.split_once(':') {
        Some((field, "desc")) => (field, true),
        Some((field, _)) => (field, false),
        None => (spec, false),
    }
}

/// Compare two tasks on `field` ascending. Callers reverse the result for a
/// descending spec.
///
/// `status`, `ready`, `priority` and `difficulty` need config- or
/// readiness-aware ordering that a raw field value can't express: legend
/// position, readiness band, a resolved numeric scale point. They stay
/// special-cased. Every other field falls through to the shared registry,
/// including any field added to [`crate::model::fields::FIELDS`] later.
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

/// A field's value from the shared [`crate::model::fields`] registry, or an
/// `x-<ns>` slot's value from [`crate::model::fields::extra`]. Returns `Missing`
/// when `field` is neither. `--sort` never reaches that case, because
/// `sort_by_specs` validates every spec first and [`compare_field`] intercepts
/// the one other sort name, `ready`.
fn registry_value(task: &Task, field: &str, config: &Config) -> FieldValue {
    if fields::is_derived(field) {
        return fields::derived(task, field, config).unwrap_or(FieldValue::Missing);
    }
    if fields::is_extra(field) {
        return fields::extra(task, field);
    }
    fields::get(field).map_or(FieldValue::Missing, |def| (def.get)(task, config))
}

/// Compare two resolved field values for `--sort`, ascending: numbers
/// numerically, strings lexicographically (ISO-correct for dates), and a
/// missing value after every present one. That is the missing-last convention
/// [`cmp_num_asc`] and [`cmp_opt_asc`] already use, and every field orders
/// missing values the same way. `sort_by_specs` reverses the whole comparison
/// for `:desc`. There a missing value sorts first, the same as `priority` and
/// `difficulty`, whose `cmp_num_asc` arms follow the identical rule.
///
/// Both values come from the same field name, and a core field's values hold
/// the same variant when both are present. An `x-<ns>` slot may hold a number
/// on one task and text or a list on another, and those tie.
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
/// When `config.ls.default_sort` holds a chain, [`sort_by_specs`] applies it
/// (validated at load). Otherwise the built-in chain applies: ready work first
/// (ready → blocked → settled), then priority descending, status lifecycle, due
/// ascending, id as stable tiebreak.
///
/// Compute `readiness` over the full loaded set (see [`Readiness::compute`]).
/// Pass [`Readiness::empty`] when no set context applies.
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
/// (2). The default view puts work you can start at the top.
#[must_use]
pub fn ready_rank(task: &Task, readiness: &Readiness) -> u8 {
    match readiness.state(task) {
        ReadyState::Ready => 0,
        ReadyState::Blocked => 1,
        ReadyState::Settled => 2,
    }
}

/// Position of a task's status in the configured lifecycle. An unknown or
/// missing status sorts last.
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

/// Compare numbers descending (higher first). A missing value sorts last.
fn cmp_num_desc(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(high), Some(low)) => low.partial_cmp(&high).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Compare options ascending with present values first. A missing value sorts
/// last.
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

#[cfg(test)]
#[path = "sort_compare_tests_comparators.rs"]
mod tests_compare_comparators;
