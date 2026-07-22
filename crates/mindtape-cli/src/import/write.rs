//! Land or preview a mapped-and-emitted batch.
//!
//! Every staged record is written first, then the destination is reloaded and
//! validated *once* as a whole (so records that link to each other resolve),
//! and violations are attributed back per task by matching on its written
//! path. For a real (non-`--dry-run`) write, any file that failed is removed —
//! the batch never leaves a stray half-valid file in the real destination.
//! `--dry-run` stages into a temp mirror of the project instead, so nothing
//! under the real destination ever changes.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use mindtape::ImportReport;

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

/// Write every `staged` record under `dest`, then reload `project.root` fresh
/// and validate the whole reloaded set (every pre-existing task plus every
/// newly written one), so cross-links between imported records resolve. A
/// record attributed any problem is removed from `dest` before returning, so
/// nothing half-valid lands.
///
/// # Errors
/// Returns an error if `dest` cannot be created, a file cannot be written, or
/// the project cannot be reloaded to validate.
pub(crate) fn land(
    project: &Project,
    dest: &Path,
    staged: Vec<StagedTask>,
) -> Result<Vec<TaskOutcome>> {
    std::fs::create_dir_all(dest).with_context(|| format!("create {}", dest.display()))?;
    let written = write_all(&project.config, dest, &staged)?;
    crate::add::warn_if_outside_roots(project, dest);

    let problems = check_written(&project.root, &written)?;
    let outcomes = pair_outcomes(staged, problems);
    for (outcome, (_, path)) in outcomes.iter().zip(&written) {
        if !outcome.filed {
            let _ = std::fs::remove_file(path);
        }
    }
    Ok(outcomes)
}

/// Preview `staged` without touching `dest`: stage into a temp mirror of
/// `project` (its `.mindtape/config.toml`, in-tree `lib/` override if any, and
/// every existing task under the configured roots), write the batch there, and
/// run the same reload-and-validate gate as [`land`] — minus any real-
/// filesystem effect.
///
/// # Errors
/// Returns an error if the staging mirror or a file write fails.
pub(crate) fn preview(
    project: &Project,
    dest: &Path,
    staged: Vec<StagedTask>,
) -> Result<Vec<TaskOutcome>> {
    let tmp = tempfile::tempdir().context("create dry-run staging directory")?;
    mirror_project(project, tmp.path())?;
    // `dest` is normally under `project.root` (the common `--into .` / a
    // relative `--into` case); an absolute `--into` outside every root falls
    // back to the mirror's own top level, the same "not tracked" territory
    // `warn_if_outside_roots` already flags for a real write.
    let dest_rel = dest
        .strip_prefix(&project.root)
        .unwrap_or_else(|_| Path::new("."));
    let mirrored_dest = tmp.path().join(dest_rel);
    std::fs::create_dir_all(&mirrored_dest)
        .with_context(|| format!("create {}", mirrored_dest.display()))?;

    let written = write_all(&project.config, &mirrored_dest, &staged)?;
    let problems = check_written(tmp.path(), &written)?;
    Ok(pair_outcomes(staged, problems))
}

fn write_all(
    config: &mindtape::Config,
    dest: &Path,
    staged: &[StagedTask],
) -> Result<Vec<(String, PathBuf)>> {
    let mut written = Vec::with_capacity(staged.len());
    for task in staged {
        let path = dest.join(format!("{}.typ", task.stem));
        crate::util::write_task(config, &path, task.source.clone())?;
        written.push((task.stem.clone(), path));
    }
    Ok(written)
}

fn pair_outcomes(
    staged: Vec<StagedTask>,
    problems: Vec<(String, Vec<String>)>,
) -> Vec<TaskOutcome> {
    staged
        .into_iter()
        .zip(problems)
        .map(|(task, (_, problems))| TaskOutcome {
            label: task.label,
            filed: problems.is_empty(),
            problems,
            report: task.report,
        })
        .collect()
}

/// Reload `root` fresh (so the just-written files are visible) and validate
/// every task; returns, for each `(stem, path)` in `written`, the problems
/// attributed to it — a load error, a missing task marker, or every
/// [`mindtape::validate`] violation on its id. Empty means clean.
fn check_written(root: &Path, written: &[(String, PathBuf)]) -> Result<Vec<(String, Vec<String>)>> {
    let config = mindtape::load_config(root).context("reload config to validate the batch")?;
    let outcome = mindtape::load_project(root, &config);
    let violations = mindtape::validate(&outcome.tasks, &config);

    Ok(written
        .iter()
        .map(|(stem, path)| {
            (
                stem.clone(),
                problems_for(root, path, &outcome, &violations),
            )
        })
        .collect())
}

pub(crate) fn problems_for(
    root: &Path,
    path: &Path,
    outcome: &mindtape::LoadOutcome,
    violations: &[mindtape::Violation],
) -> Vec<String> {
    let rel = path.strip_prefix(root).unwrap_or(path);
    if let Some(error) = outcome.errors.iter().find(|err| err.file == rel) {
        return vec![error.message.clone()];
    }
    let Some(task) = outcome.tasks.iter().find(|task| task.file == rel) else {
        return vec!["did not harvest a task marker".to_owned()];
    };
    violations
        .iter()
        .filter(|violation| violation.task_id == task.id)
        .map(|violation| format!("{}: {}", violation.field, violation.message))
        .collect()
}

/// Copy `project.root`'s `.mindtape/`, in-tree `lib/` override (if present),
/// and every existing file under the configured task roots into `dst` — the
/// peers a dry-run validate pass needs so cross-links to pre-existing tasks
/// resolve.
pub(crate) fn mirror_project(project: &Project, dst: &Path) -> Result<()> {
    copy_dir(&project.root.join(".mindtape"), &dst.join(".mindtape"))?;
    let lib = project.root.join("lib");
    if lib.is_dir() {
        copy_dir(&lib, &dst.join("lib"))?;
    }
    for spec in &project.config.tasks.roots {
        let src = project.root.join(spec.path());
        if src.is_dir() {
            copy_dir(&src, &dst.join(spec.path()))?;
        }
    }
    Ok(())
}

pub(crate) fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).with_context(|| format!("create {}", dst.display()))?;
    for entry in std::fs::read_dir(src).with_context(|| format!("read {}", src.display()))? {
        let entry = entry.context("read directory entry")?;
        let target = dst.join(entry.file_name());
        let file_type = entry.file_type().context("read entry file type")?;
        if file_type.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &target)
                .with_context(|| format!("copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}
