//! Land or preview a mapped-and-emitted batch.
//!
//! The validator checks the whole batch *before* any write
//! ([`crate::pre_write`]). Every staged record goes to it as a candidate at the
//! path it would occupy, all at once, so records that link to each other
//! resolve. Only the clean ones then reach disk. A bad entry costs the batch
//! nothing. The answer to "one bad record or the whole batch" is and stays
//! *land the rest*, which is what the per-record `N filed, M skipped` report
//! has always said.
//!
//! The order is the change. The batch goes through validation before any write,
//! so a rejected record never exists on disk for a full project reload, and no
//! removal can leave it there. A rejected record doesn't create anything, so
//! there is nothing to take back.
//!
//! `--dry-run` differs from a real run in one line: it skips the write. It
//! doesn't need a temp mirror of the project, because a candidate validates
//! without a file either way.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::diag::Result;
use mindtape::ImportReport;

use crate::cmd::write_guard::{self, Mode};
use crate::diag::Diag;
use crate::pre_write::{self, Pending, Problem};
use crate::util::Project;

/// One mapped-and-emitted record, ready to land: its report label, the folder
/// it lands in, its chosen filename stem, and rendered `.typ` source.
pub(crate) struct StagedTask {
    pub label: String,
    pub dest: PathBuf,
    pub stem: String,
    pub source: String,
    pub report: ImportReport,
}

/// The landed (or previewed) outcome of one staged record.
pub(crate) struct TaskOutcome {
    pub label: String,
    /// The stem written, or under a dry run the stem that would be. Empty for a
    /// record skipped before it had one.
    pub stem: String,
    pub filed: bool,
    /// Load/validate problems attributed to this record. The list is empty when
    /// the record is clean.
    pub problems: Vec<Problem>,
    pub report: ImportReport,
}

/// Check every `staged` record at the path it would occupy, then write the ones
/// that passed. A rejected record never reaches disk, so there is nothing to
/// remove and no window in which a half-valid file exists.
///
/// `land` creates a record's folder only after the record has earned a write.
/// A batch rejected end to end leaves not even an empty directory behind. The
/// destination guard runs first, for the same reason: it refuses a destination
/// no later command would load before that folder exists.
///
/// # Errors
/// Returns an error if a destination would be invisible, or if `land` cannot
/// create a folder or write a file.
pub(crate) fn land(project: &Project, staged: Vec<StagedTask>) -> Result<Vec<TaskOutcome>> {
    guard_staged(project, &staged)?;
    let mut warned = HashSet::new();
    for task in &staged {
        if warned.insert(task.dest.clone()) {
            crate::cmd::add::warn_if_outside_roots(project, &task.dest);
        }
    }
    let problems = check_staged(project, &staged)?;

    for (task, problems) in staged.iter().zip(&problems) {
        if !problems.is_empty() {
            continue;
        }
        std::fs::create_dir_all(&task.dest).map_err(|err| {
            Diag::failed("can't create")
                .value(mindtape::display_path(&task.dest))
                .io(&err)
        })?;
        let path = task.dest.join(format!("{}.typ", task.stem));
        crate::util::write_task(&project.config, &path, task.source.clone())?;
    }
    Ok(pair_outcomes(staged, problems))
}

/// Preview `staged` without touching a destination: exactly [`land`]'s checks,
/// the destination guard included, minus the write. A candidate validates
/// without a file either way, so [`land`] and [`preview`] run the identical
/// check, and neither needs a temp mirror of the project. The guard itself
/// doesn't create anything, so a refused preview doesn't leave a directory
/// behind.
///
/// # Errors
/// Returns an error if a destination would be invisible, or if the project's
/// configured body rules do not compile (see [`crate::pre_write::check`]).
pub(crate) fn preview(project: &Project, staged: Vec<StagedTask>) -> Result<Vec<TaskOutcome>> {
    guard_staged(project, &staged)?;
    let problems = check_staged(project, &staged)?;
    Ok(pair_outcomes(staged, problems))
}

/// Refuse the batch when any record's final destination would be invisible,
/// naming that record's resolved path and the reason the loader gave.
///
/// # Errors
/// The first refusal. The guard doesn't create anything.
fn guard_staged(project: &Project, staged: &[StagedTask]) -> Result<()> {
    for task in staged {
        let file = task.dest.join(format!("{}.typ", task.stem));
        write_guard::check(project, &file, Mode::Import)?;
    }
    Ok(())
}

/// The problems attributed to each staged record, in input order. An empty list
/// means it is safe to write.
///
/// The batch goes to the validator as one set, so a record linking to another
/// record in the same import resolves. Every pre-existing task is in that set
/// too, so a link out of the batch resolves as well.
///
/// # Errors
/// Returns an error if the project's configured body rules do not compile
/// (see [`crate::pre_write::check`]).
fn check_staged(project: &Project, staged: &[StagedTask]) -> Result<Vec<Vec<Problem>>> {
    let files: Vec<PathBuf> = staged
        .iter()
        .map(|task| task.dest.join(format!("{}.typ", task.stem)))
        .collect();
    let pending: Vec<Pending<'_>> = files
        .iter()
        .zip(staged)
        .map(|(file, task)| Pending {
            file,
            source: &task.source,
        })
        .collect();
    Ok(pre_write::check(project, &project.outcome.tasks, &pending)?
        .into_iter()
        .map(|verdict| verdict.problems())
        .collect())
}

fn pair_outcomes(staged: Vec<StagedTask>, problems: Vec<Vec<Problem>>) -> Vec<TaskOutcome> {
    staged
        .into_iter()
        .zip(problems)
        .map(|(task, problems)| TaskOutcome {
            label: task.label,
            stem: task.stem,
            filed: problems.is_empty(),
            problems,
            report: task.report,
        })
        .collect()
}

#[cfg(test)]
#[path = "write_tests.rs"]
mod tests;
