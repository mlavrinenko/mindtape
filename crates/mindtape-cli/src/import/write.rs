//! Land or preview a mapped-and-emitted batch.
//!
//! The whole batch is checked *before* any of it is written
//! ([`crate::pre_write`]): every staged record goes to the validator as a
//! candidate at the path it would occupy, all of them at once so records that
//! link to each other resolve, and only the clean ones are then written. A bad
//! entry costs the batch nothing — the answer to "one bad record or the whole
//! batch" is and stays *land the rest*, which is what the per-record
//! `N filed, M skipped` report has always said.
//!
//! What changed is the order. This used to write everything, reload, and
//! `remove_file` the failures — so a rejected record existed on disk for the
//! length of a full project reload, and a removal that failed left it there
//! for good. Nothing is created for a rejected record now, so nothing has to
//! be taken back.
//!
//! `--dry-run` differs from a real run in one line: it skips the write. It
//! needs no temp mirror of the project any more, because a candidate is
//! validated without a file either way.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use mindtape::ImportReport;

use crate::pre_write::{self, Pending};
use crate::util::Project;

/// One mapped-and-emitted record, ready to land: its report label, chosen
/// filename stem, and rendered `.typ` source.
pub(crate) struct StagedTask {
    pub label: String,
    pub stem: String,
    pub source: String,
    pub report: ImportReport,
}

/// The landed (or previewed) outcome of one staged record.
pub(crate) struct TaskOutcome {
    pub label: String,
    pub filed: bool,
    /// Load/validate problems attributed to this record; empty when clean.
    pub problems: Vec<String>,
    pub report: ImportReport,
}

/// Check every `staged` record at the path it would occupy, then write the ones
/// that passed. A rejected record is never created, so there is nothing to
/// remove and no window in which a half-valid file is on disk.
///
/// `dest` is created only once a record has actually earned a write: a batch
/// rejected end to end leaves not even an empty directory behind.
///
/// # Errors
/// Returns an error if `dest` cannot be created or a file cannot be written.
pub(crate) fn land(
    project: &Project,
    dest: &Path,
    staged: Vec<StagedTask>,
) -> Result<Vec<TaskOutcome>> {
    crate::add::warn_if_outside_roots(project, dest);
    let problems = check_staged(project, dest, &staged)?;

    let mut made_dest = false;
    for (task, problems) in staged.iter().zip(&problems) {
        if !problems.is_empty() {
            continue;
        }
        if !made_dest {
            std::fs::create_dir_all(dest).with_context(|| format!("create {}", dest.display()))?;
            made_dest = true;
        }
        let path = dest.join(format!("{}.typ", task.stem));
        crate::util::write_task(&project.config, &path, task.source.clone())?;
    }
    Ok(pair_outcomes(staged, problems))
}

/// Preview `staged` without touching `dest`: exactly [`land`]'s check, minus
/// the write. A candidate is validated without a file either way, so the two
/// paths ask the identical question and no temp mirror of the project has to
/// be built to ask it.
///
/// # Errors
/// Returns an error if the project's configured body rules do not compile
/// (see [`crate::pre_write::check`]).
pub(crate) fn preview(
    project: &Project,
    dest: &Path,
    staged: Vec<StagedTask>,
) -> Result<Vec<TaskOutcome>> {
    let problems = check_staged(project, dest, &staged)?;
    Ok(pair_outcomes(staged, problems))
}

/// The problems attributed to each staged record, in input order — empty means
/// it is safe to write.
///
/// The batch goes to the validator as one set, so a record linking to another
/// record in the same import resolves; every pre-existing task is in that set
/// too, so a link out of the batch resolves as well.
///
/// # Errors
/// Returns an error if the project's configured body rules do not compile
/// (see [`crate::pre_write::check`]).
fn check_staged(project: &Project, dest: &Path, staged: &[StagedTask]) -> Result<Vec<Vec<String>>> {
    let files: Vec<PathBuf> = staged
        .iter()
        .map(|task| dest.join(format!("{}.typ", task.stem)))
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
        .map(|verdict| verdict.problems)
        .collect())
}

fn pair_outcomes(staged: Vec<StagedTask>, problems: Vec<Vec<String>>) -> Vec<TaskOutcome> {
    staged
        .into_iter()
        .zip(problems)
        .map(|(task, problems)| TaskOutcome {
            label: task.label,
            filed: problems.is_empty(),
            problems,
            report: task.report,
        })
        .collect()
}
