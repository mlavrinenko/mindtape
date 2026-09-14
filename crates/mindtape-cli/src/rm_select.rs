//! Resolve `mt rm`'s variadic `target` argument into the set of task ids to
//! retire — split out of `rm.rs` to stay under the file-size gate.

use std::collections::HashSet;

use anyhow::{Context, Result, bail};
use mindtape::{Filter, Readiness, Task, select};

use crate::util;

/// Resolve the variadic `targets` into the set of task ids to retire.
///
/// Each entry resolves reference-first (path, handle, slug — slug wins on a
/// bare token), falling back to a filter parse + [`select`] when it is not a
/// reference; every entry's matches are unioned, deduplicated by id — the same
/// task named twice, or named once directly and again by a filter, retires
/// once. An empty `targets` applies the configured `[rm] default_filter`.
///
/// # Errors
/// Returns an error naming every entry that neither resolves as a reference
/// nor parses as a filter, or if a filter references an unknown field.
pub(super) fn select_targets(
    project: &util::Project,
    targets: &[String],
    readiness: &Readiness,
) -> Result<HashSet<String>> {
    if targets.is_empty() {
        return default_targets(project, readiness);
    }
    let mut ids = HashSet::new();
    let mut problems = Vec::new();
    for spec in targets {
        match resolve_one(project, spec, readiness) {
            Ok(found) => ids.extend(found),
            Err(err) => problems.push(format!("{spec}: {err:#}")),
        }
    }
    if !problems.is_empty() {
        bail!("{}", unresolved_message(&problems));
    }
    Ok(ids)
}

/// The ids the configured `[rm] default_filter` selects — today's zero-target
/// behaviour, applied when `targets` is empty.
fn default_targets(project: &util::Project, readiness: &Readiness) -> Result<HashSet<String>> {
    match project.config.rm.default_filter.as_deref() {
        Some(expr) => {
            let filter = Filter::parse(expr)
                .with_context(|| format!("parse [rm] default_filter {expr:?}"))?;
            Ok(ids_of(filter_tasks(project, &filter, readiness)?))
        }
        None => Ok(HashSet::new()),
    }
}

/// Resolve one target entry: reference-first (path, handle, slug — slug wins
/// on a bare token), falling back to a filter parse + [`select`] when it is
/// not a reference. Surfaces the reference error when the input does not even
/// parse as a filter (a mistyped ref, not a filter).
fn resolve_one(
    project: &util::Project,
    spec: &str,
    readiness: &Readiness,
) -> Result<HashSet<String>> {
    match util::resolve_task(project, spec) {
        Ok((task, _)) => Ok(std::iter::once(task.id).collect()),
        Err(ref_err) => {
            let filter = Filter::parse(spec).map_err(|_| ref_err)?;
            Ok(ids_of(filter_tasks(project, &filter, readiness)?))
        }
    }
}

/// Task ids out of a resolved task list.
fn ids_of(tasks: Vec<Task>) -> HashSet<String> {
    tasks.into_iter().map(|task| task.id).collect()
}

/// The bail message for one or more targets that failed to resolve: the lone
/// `<spec>: <reason>` line, or a numbered list when several entries failed.
fn unresolved_message(problems: &[String]) -> String {
    if let [only] = problems {
        return only.clone();
    }
    format!(
        "{} target(s) did not resolve:\n  {}",
        problems.len(),
        problems.join("\n  ")
    )
}

/// Select every task matching `filter` over the full loaded set.
fn filter_tasks(
    project: &util::Project,
    filter: &Filter,
    readiness: &Readiness,
) -> Result<Vec<Task>> {
    Ok(select(
        project.outcome.tasks.clone(),
        Some(filter),
        &project.config,
        readiness,
    )?)
}
