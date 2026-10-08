//! Resolve `mt rm`'s variadic `target` argument into the set of task ids to
//! retire — split out of `cmd/rm/mod.rs` to stay under the file-size gate.

use std::collections::HashSet;

use anyhow::Result;
use mindtape::{Filter, Readiness, Task, select};

use crate::diag::{self, Diag};
use crate::util;

/// Resolve the variadic `targets` into the set of task ids to retire.
///
/// Each entry resolves as [`resolve_one`] says — a reference, else a filter
/// parse + [`select`]; every entry's matches are unioned, deduplicated by id —
/// the same task named twice, or named once directly and again by a filter,
/// retires once. An empty `targets` applies the configured `[rm] default_filter`.
///
/// # Errors
/// A lone entry that neither resolves as a reference nor parses as a filter
/// is that entry's own error. With several, each prints on its own and a batch
/// summary ends the run.
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
            Err(err) => problems.push(Diag::from(err)),
        }
    }
    unresolved(problems, targets.len())?;
    Ok(ids)
}

/// The ids the configured `[rm] default_filter` selects — today's zero-target
/// behaviour, applied when `targets` is empty.
fn default_targets(project: &util::Project, readiness: &Readiness) -> Result<HashSet<String>> {
    match project.config.rm.default_filter.as_deref() {
        Some(expr) => {
            let filter = Filter::parse(expr).map_err(|err| {
                Diag::failed("can't parse filter")
                    .rule("rm.default-filter")
                    .value(expr)
                    .cause(&err)
            })?;
            Ok(ids_of(filter_tasks(project, &filter, readiness, expr)?))
        }
        None => Ok(HashSet::new()),
    }
}

/// Resolve one target entry: a path or exact stem, then a filter term
/// ([`Filter::names_term`]), then a stem hash, then any other filter. Surfaces
/// the reference error when the input does not even parse as a filter (a
/// mistyped ref, not a filter).
///
/// A term goes before the stem hash because `mt rm` retires what it resolves:
/// `done` or `has` must stay a filter even where a task's stem hash spells it.
fn resolve_one(
    project: &util::Project,
    spec: &str,
    readiness: &Readiness,
) -> Result<HashSet<String>> {
    let is_stem = project.outcome.tasks.iter().any(|task| task.id == spec);
    if !is_stem && Filter::names_term(spec, &project.config) {
        let filter = parse(spec)?;
        return Ok(ids_of(filter_tasks(project, &filter, readiness, spec)?));
    }
    match util::resolve_task(project, spec) {
        Ok((task, _)) => Ok(std::iter::once(task.id).collect()),
        // A lone word that is no stem, hash or term can't be a filter: filters
        // that name a field or term were routed above, and any other bare word
        // is a field that doesn't exist. It names a task that isn't there.
        Err(ref_err) if is_bare_word(spec) => Err(ref_err),
        Err(ref_err) => {
            let filter = Filter::parse(spec).map_err(|_| ref_err)?;
            Ok(ids_of(filter_tasks(project, &filter, readiness, spec)?))
        }
    }
}

/// Whether `spec` is one word of stem characters, with no operator, quote,
/// path separator or space that would make it an expression.
fn is_bare_word(spec: &str) -> bool {
    !spec.is_empty()
        && spec
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Task ids out of a resolved task list.
fn ids_of(tasks: Vec<Task>) -> HashSet<String> {
    tasks.into_iter().map(|task| task.id).collect()
}

/// `spec` as a filter, its failure worded for the command line.
fn parse(spec: &str) -> Result<Filter> {
    Filter::parse(spec).map_err(|err| diag::filter(err, Some(spec)).into())
}

/// Fail when any of the `total` entries did not resolve: the one problem as it
/// is, or every problem printed and a summary returned.
fn unresolved(mut problems: Vec<Diag>, total: usize) -> Result<()> {
    match problems.len() {
        0 => Ok(()),
        1 => problems.pop().map_or(Ok(()), |only| Err(only.into())),
        failed => {
            for problem in &problems {
                diag::emit(problem);
            }
            Err(Diag::batch("resolved", total.saturating_sub(failed), total).into())
        }
    }
}

/// Select every task matching `filter` over the full loaded set.
fn filter_tasks(
    project: &util::Project,
    filter: &Filter,
    readiness: &Readiness,
    expr: &str,
) -> Result<Vec<Task>> {
    select(
        project.outcome.tasks.clone(),
        Some(filter),
        &project.config,
        readiness,
    )
    .map_err(|err| diag::filter(err, Some(expr)).into())
}
