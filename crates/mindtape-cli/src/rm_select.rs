//! Resolve `mt rm`'s optional `target` argument into the set of task ids to
//! retire — split out of `rm.rs` to stay under the file-size gate.

use std::collections::HashSet;

use anyhow::{Context, Result};
use mindtape::{Filter, Readiness, Task, select};

use crate::util;

/// Resolve the optional `target` into the set of task ids to retire.
///
/// A `Some` target resolves reference-first (path, handle, slug — slug wins on a
/// bare token), falling back to a filter parse + [`select`] when it is not a
/// reference. A `None` target applies the configured `[rm] default_filter`.
///
/// # Errors
/// Returns an error if the reference does not resolve and the input is not a
/// valid filter, or if the filter references an unknown field.
pub(super) fn select_targets(
    project: &util::Project,
    target: Option<&str>,
    readiness: &Readiness,
) -> Result<HashSet<String>> {
    let tasks = match target {
        Some(spec) => match util::resolve_task(project, spec) {
            Ok((task, _)) => vec![task],
            // Not a reference: try a filter. Surface the reference error when the
            // input does not even parse as one (a mistyped ref, not a filter).
            Err(ref_err) => {
                let filter = Filter::parse(spec).map_err(|_| ref_err)?;
                filter_tasks(project, &filter, readiness)?
            }
        },
        None => match project.config.rm.default_filter.as_deref() {
            Some(expr) => {
                let filter = Filter::parse(expr)
                    .with_context(|| format!("parse [rm] default_filter {expr:?}"))?;
                filter_tasks(project, &filter, readiness)?
            }
            None => Vec::new(),
        },
    };
    Ok(tasks.into_iter().map(|task| task.id).collect())
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
