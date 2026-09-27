//! Presence, staleness, changed-path count, and substring-search arms of the filter language's
//! function dispatch (`has`/`miss`/`contains`/`stale`/`changed`), split out of
//! `filter.rs` (file-size gate). Each function here is pure over `Task` (plus
//! `Config` where a field needs one to resolve, e.g. a framework priority) —
//! none of them touch the `Ctx` that threads `ready`/`blocked`/`in_view`
//! recursion state through the evaluator, unlike the dispatcher itself
//! (`filter::function`), which stays put and calls into these.

use evalexpr::{DefaultNumericTypes, Node};

use crate::config::Config;
use crate::expr::{call_args, extract_identifier, extract_literal};
use crate::fields::{self, FieldValue};
use crate::filter::{FilterError, field_value};
use crate::git_state::{PathFilter, StateSet};
use crate::model::Task;
use crate::numeric::format_num;

pub(crate) fn present(task: &Task, name: &str, config: &Config) -> Result<bool, FilterError> {
    match name {
        "tags" => Ok(!task.tags.is_empty() || !task.derived_tags.is_empty()),
        "estimate" => Ok(!task.estimate.is_empty()),
        slot if slot.starts_with("x-") => Ok(task.extras.contains_key(slot)),
        other => Ok(!matches!(
            field_value(task, other, config)?,
            FieldValue::Missing
        )),
    }
}

/// `estimate.<unit>`: one unit of the estimate dict, read as a number, for
/// any unit name — mindtape knows none of them. `None` when `name` is not an
/// estimate unit at all. An absent unit is [`FieldValue::Missing`], like any
/// absent numeric field; a unit outside a declared `[tasks.estimate] units`
/// is an unknown field, so a saved expression naming one fails at load.
pub(crate) fn estimate_unit(
    task: &Task,
    name: &str,
    config: &Config,
) -> Option<Result<FieldValue, FilterError>> {
    let unit = name.strip_prefix("estimate.")?;
    let declared = &config.tasks.estimate.units;
    if !declared.is_empty() && !declared.iter().any(|known| known == unit) {
        return Some(Err(FilterError::UnknownField(name.to_owned())));
    }
    Some(Ok(task
        .estimate
        .get(unit)
        .map_or(FieldValue::Missing, |value| {
            FieldValue::Num(*value)
        })))
}

/// `changed("<states>", "<glob>"…)`: how many distinct paths git reports
/// changed in `states` that the globs admit. `dry` validates the arguments and
/// counts zero without reading git — the config-load dry run.
pub(crate) fn changed_count(
    children: &[Node<DefaultNumericTypes>],
    config: &Config,
    dry: bool,
) -> Result<f64, FilterError> {
    let args = call_args(children);
    let Some((states_node, glob_nodes)) = args.split_first() else {
        return Err(FilterError::Invalid(
            "changed() needs its git states: changed(\"staged+worktree\", \"<glob>\"…)".to_owned(),
        ));
    };
    let invalid = |err: String| FilterError::Invalid(format!("changed(): {err}"));
    let states =
        StateSet::parse(&extract_literal(states_node).map_err(invalid)?).map_err(invalid)?;
    let globs = glob_nodes
        .iter()
        .map(|node| extract_literal(node).map_err(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    let admitted = PathFilter::new(&globs).map_err(invalid)?;
    if dry {
        return Ok(0.0);
    }
    let git_status = config.git.status().map_err(FilterError::Git)?;
    Ok(git_status
        .paths(states)
        .filter(|path| admitted.admits(path))
        .count() as f64)
}

pub(crate) fn is_stale(task: &Task, days: u64) -> bool {
    let threshold = crate::date::days_ago_iso(days);
    if let Some(rev) = &task.reviewed {
        rev.as_str() < threshold.as_str()
    } else if let Some(ref_date) = task.status.as_ref().and_then(|s| s.date.as_deref()) {
        ref_date < threshold.as_str()
    } else {
        true
    }
}

fn contains_ci(value: &FieldValue, needle: &str) -> bool {
    let haystack = match value {
        FieldValue::Str(text) => text.clone(),
        FieldValue::Num(num) => format_num(*num),
        FieldValue::Missing => return false,
    };
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Shipped default `contains()` haystack when no field is named, and no
/// `[ls] contains-fields` is configured — the prose surface. Tags have
/// `has_tag()` and every scalar field has a comparison operator, so neither
/// belongs in an implicit text sweep. Deliberately narrow: a default applied
/// uniformly across every project inherits the strictest constraint of
/// anything that could land in it, so widening this for everyone is the
/// wrong move — a project opts in via `[ls] contains-fields` instead (see
/// [`default_contains_fields`]).
const DEFAULT_CONTAINS_FIELDS: [&str; 3] = ["title", "id", fields::BODY_FIELD];

/// The default `contains()` haystack: the project's `[ls] contains-fields`
/// when set, else [`DEFAULT_CONTAINS_FIELDS`] unchanged. Config load already
/// validated a configured list against [`fields::is_contains_field`]
/// ([`crate::config_ls::deserialize_contains_fields`]), so every name here is
/// trusted.
fn default_contains_fields(config: &Config) -> Vec<String> {
    if config.ls.contains_fields.is_empty() {
        DEFAULT_CONTAINS_FIELDS
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    } else {
        config.ls.contains_fields.clone()
    }
}

/// `contains(needle)` / `contains(needle, field, ...)`: case-insensitive
/// substring, needle first. Zero or more bare field identifiers follow; none
/// named searches [`DEFAULT_CONTAINS_FIELDS`].
///
/// Names are resolved and validated in full (`collect` into a `Result<Vec<_>,
/// _>`, short-circuiting only on the first invalid one) before any is matched
/// against the needle — an `||` fold straight over resolved values would
/// short-circuit on the first hit, letting a bogus name past an
/// already-matching one through unvalidated.
pub(crate) fn contains_call(
    children: &[Node<DefaultNumericTypes>],
    task: &Task,
    config: &Config,
) -> Result<bool, FilterError> {
    let args = call_args(children);
    let Some((needle_node, field_nodes)) = args.split_first() else {
        return Err(FilterError::Invalid(
            "contains() needs a needle: contains(\"substr\") or contains(\"substr\", field, ...)"
                .to_owned(),
        ));
    };
    let needle = extract_literal(needle_node).map_err(FilterError::Invalid)?;

    let names: Vec<String> = if field_nodes.is_empty() {
        default_contains_fields(config)
    } else {
        field_nodes
            .iter()
            .map(|node| extract_identifier(node).map_err(FilterError::Invalid))
            .collect::<Result<Vec<_>, _>>()?
    };
    let values = names
        .iter()
        .map(|name| contains_value(task, name, config))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(values.iter().any(|value| contains_ci(value, &needle)))
}

/// Resolve `name`'s `contains()` haystack on `task`: `body` reads
/// [`Task::body`] directly (marker-free), `note` joins every `links[].note`
/// ([`joined_notes`]), `tags` joins authored and derived tags
/// ([`Task::all_tags`]); anything else defers to [`field_value`]. Errors
/// outside [`fields::is_contains_field`]'s vocabulary.
fn contains_value(task: &Task, name: &str, config: &Config) -> Result<FieldValue, FilterError> {
    if !fields::is_contains_field(name) {
        return Err(FilterError::UnknownField(name.to_owned()));
    }
    if name == fields::BODY_FIELD {
        return Ok(FieldValue::Str(task.body.clone()));
    }
    if name == fields::NOTE_FIELD {
        return Ok(FieldValue::Str(joined_notes(task)));
    }
    if name == fields::TAGS_FIELD {
        return Ok(FieldValue::Str(task.all_tags().join(" ")));
    }
    field_value(task, name, config)
}

/// Every `links[].note` on `task`, space-joined so a needle cannot spuriously
/// span the boundary between two adjacent notes.
fn joined_notes(task: &Task) -> String {
    task.links
        .iter()
        .filter_map(|link| link.note.as_deref())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
#[path = "filter_changed_tests.rs"]
mod changed_tests;
