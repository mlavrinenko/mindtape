//! `mt recur` — the sole recurrence writer: advance a recurring task to its
//! next occurrence.
//!
//! Dispatches on the task's effective `materialize` (`spawn`, the default, or
//! `mutate`; see `[tasks.recur] materialize`). `--dry-run` previews without
//! writing; `--advance N` pre-materializes `N` future spawns without closing
//! the current instance; `--skip` advances the anchor/`count` in place
//! without materializing anything. `mt ls`/`mt check` never call into this
//! module — see `truth/tasks/command-semantics-cqrs.typ`.
//!
//! # Files
//!
//! - `advance.rs` — `--advance N`, pre-materializing future spawns.
//! - `edit.rs` — the Typst-source rewrites for `mt recur`.
//! - `stem.rs` — the stem and occurrence number of the next spawn.

pub(crate) mod edit;

use std::collections::HashSet;
use std::path::Path;

use crate::diag::Result;
use chrono::NaiveDate;
use clap::Args;
use mindtape::recur::dates::{
    self as recur_dates, advance_once, effective_materialize, exhausted, next_date, past_until,
};
use mindtape::recur::{Recur, RecurConfig};
use mindtape::{Task, today_ymd};

use crate::cmd::recur::edit::{
    OccurrenceDates, apply_edits, decrement_count_edit, due_start_edits, set_recurred_from,
    status_due_start_edits,
};
use crate::diag::{Diag, Hint, Kind, Note};
use crate::pre_write::{self, Pending};
use crate::task_edit::task_with;
use crate::util::{self, Project};

mod advance;
use advance::do_advance;
mod stem;
use stem::{StemDest, mint_stem, occurrence_index, stem_of, successor_of};
// Only reachable from the `--advance` test module below (glob-imported via
// `crate::*`); `cmd/recur/mod.rs` itself calls `advance::do_advance` alone; `--advance`
// dispatches everything else internally.
#[cfg(test)]
use advance::{build_advance_chain, check_advance_preconditions, write_advance_chain};

/// Arguments for `mt recur`.
#[derive(Args)]
pub struct RecurArgs {
    #[command(flatten)]
    pub task: crate::cli::task_ref::TaskRef,

    /// Show what would be written without writing it.
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Write the next N occurrences as new tasks at once.
    ///
    /// Works only for a task that spawns a new file each time.

    #[arg(long, value_name = "N", conflicts_with = "skip")]
    pub advance: Option<u32>,

    /// Move the schedule past the next occurrence without writing it.
    #[arg(long, conflicts_with = "advance")]
    pub skip: bool,

    /// Write the next occurrence even if one already exists.
    #[arg(long)]
    pub force: bool,
}

/// Run `mt recur`.
///
/// # Errors
/// Returns an error if the task cannot be resolved, has no `recur` field,
/// fails the `[tasks.recur]` policy check, hits a series stop condition, or
/// the write/revalidate fails.
pub fn run(args: &RecurArgs) -> Result<()> {
    let project = util::open_warned()?;
    run_with(&project, args)
}

fn run_with(project: &Project, args: &RecurArgs) -> Result<()> {
    let (task, abs) = util::resolve_task(project, &args.task.task)?;
    let Some(recur) = task.recur.clone() else {
        return Err(Diag::refused("task has no recur field")
            .value(&args.task.task)
            .hint(Hint::new(
                "add a recur: field to the task, or pick one that has it",
            )));
    };

    let recur_config = &project.config.tasks.recur;
    let problems = mindtape::recur::check_policy(&recur, recur_config);
    if let Some(policy) = policy_refusal(&problems) {
        return Err(policy);
    }

    if args.skip {
        return do_skip(project, &task, &abs, &recur, args.dry_run);
    }
    if args.advance.is_some() {
        return do_advance(project, &task, &abs, &recur, args);
    }
    match effective_materialize(&recur, recur_config) {
        "mutate" => do_mutate(project, &task, &abs, &recur, args.dry_run),
        _ => do_spawn(project, &task, &abs, &recur, args),
    }
}

/// The refusal for a `recur` the `[tasks.recur]` policy rejects: the first
/// problem in the policy's words, every other one a note. `None` when the
/// policy has no complaint.
fn policy_refusal(problems: &[String]) -> Option<Diag> {
    let (first, rest) = problems.split_first()?;
    let refusal = Diag::verbatim(Kind::Refused, first.as_str());
    Some(rest.iter().fold(refusal, |diag, problem| {
        diag.note(Note::new("also").name(problem.as_str()))
    }))
}

/// Today, and the task's current `due`/`start`, as [`NaiveDate`]s.
fn dates(task: &Task) -> Result<(NaiveDate, Option<NaiveDate>, Option<NaiveDate>)> {
    let Some(today) = recur_dates::ymd_to_date(today_ymd()) else {
        return Err(Diag::failed("today's date is out of range"));
    };
    let due = task.due.as_deref().and_then(recur_dates::parse_iso);
    let start = task.start.as_deref().and_then(recur_dates::parse_iso);
    Ok((today, due, start))
}

/// Bail unless the series still has budget left — checked before a date is
/// computed, since an exhausted series has no next occurrence to compute.
fn guard_budget(recur: &Recur) -> Result<()> {
    if exhausted(recur) {
        return Err(Diag::refused("series exhausted: count has reached 0"));
    }
    Ok(())
}

/// Bail if `next` overshoots the series' `until` bound; hand it back otherwise.
fn guard_until(recur: &Recur, next: NaiveDate) -> Result<NaiveDate> {
    if past_until(recur, next) {
        return Err(
            Diag::refused("series ended: the next occurrence is past until")
                .value(recur_dates::format_iso(next)),
        );
    }
    Ok(next)
}

/// Compute the next date and enforce the series stop conditions
/// (`count`/`until`) before any write.
fn plan_next(
    recur: &Recur,
    config: &RecurConfig,
    today: NaiveDate,
    due: Option<NaiveDate>,
) -> Result<NaiveDate> {
    guard_budget(recur)?;
    let next = next_date(recur, config, today, due)?;
    guard_until(recur, next)
}

/// The hop `mt recur --advance N` takes after its first: one plain step of the
/// rule off the previous occurrence's date, under the same stop conditions.
///
/// The first hop honours the anchor ([`plan_next`]); no hop after it can. An
/// anchor other than `due` resolves from today rather than from the task's
/// date, so re-asking it N times plans the same date N times — `--advance 3`
/// over a daily rule asked for three occurrences and got three copies of
/// tomorrow. So `--advance N` means: N occurrences at the rule's own spacing,
/// starting wherever the anchor puts the first one. That keeps `--advance`
/// usable under `default-anchor = "completion"` (the default), which refusing
/// a non-`due` anchor outright would not.
pub(crate) fn plan_step(recur: &Recur, from: NaiveDate) -> Result<NaiveDate> {
    guard_budget(recur)?;
    let next = advance_once(&recur.rule, from)?;
    guard_until(recur, next)
}

/// The shared prologue every single-step materialize path starts from: the
/// task's current dates, the planned next occurrence date (series stop
/// conditions already enforced), and its source text.
fn plan_and_read(
    project: &Project,
    task: &Task,
    abs: &Path,
    recur: &Recur,
) -> Result<(NaiveDate, Option<NaiveDate>, Option<NaiveDate>, String)> {
    let (today, due, start) = dates(task)?;
    let next = plan_next(recur, &project.config.tasks.recur, today, due)?;
    let source = std::fs::read_to_string(abs).map_err(|err| Diag::cannot_read(abs, &err))?;
    Ok((next, due, start, source))
}

/// The pre-write seam every single-step `recur` path funnels through: check the
/// materialized occurrence at the exact path it would occupy, and write it only
/// if it passes.
///
/// A rejected spawn creates no file, and a rejected mutate/skip leaves the task
/// exactly as its author left it. Nothing to repair either way — which is why
/// no path here reports after the fact any more.
fn commit(project: &Project, file: &Path, source: String) -> Result<()> {
    pre_write::ensure(
        project,
        &[Pending {
            file,
            source: &source,
        }],
    )?;
    util::write_task(&project.config, file, source)
}

/// The folder `abs` sits in, where its occurrences are written.
pub(super) fn parent_dir(abs: &Path) -> Result<&Path> {
    abs.parent().ok_or_else(|| {
        Diag::failed("task file has no parent directory").value(abs.display().to_string())
    })
}

// ---- spawn: write the next instance as a new file --------------------

fn do_spawn(
    project: &Project,
    task: &Task,
    abs: &Path,
    recur: &Recur,
    args: &RecurArgs,
) -> Result<()> {
    guard_double_spawn(project, task, args.force)?;
    let (next, due, start, source) = plan_and_read(project, task, abs, recur)?;
    let occurrence = occurrence_index(&project.outcome.tasks, task);
    let dir = parent_dir(abs)?;
    // A single-step spawn is a batch of one: nothing else is in flight to
    // collide with, so the claimed set is empty and disk decides alone.
    let claimed = HashSet::new();
    let stem = mint_stem(
        project,
        task,
        next,
        occurrence,
        &StemDest {
            dir,
            claimed: &claimed,
        },
    )?;
    let file = dir.join(format!("{stem}.typ"));

    let occurrence_dates = OccurrenceDates {
        today: today_ymd(),
        old_due: due,
        old_start: start,
        next_due: next,
    };
    let new_source = spawn_source(
        &source,
        &project.config,
        &occurrence_dates,
        recur,
        &stem_of(&task.file),
    )?;

    if args.dry_run {
        println!(
            "would spawn {} (due {}) from {:?}",
            file.display(),
            recur_dates::format_iso(next),
            task.id
        );
        return Ok(());
    }
    commit(project, &file, new_source)?;
    println!(
        "spawned {} (due {})",
        file.display(),
        recur_dates::format_iso(next)
    );
    Ok(())
}

fn guard_double_spawn(project: &Project, task: &Task, force: bool) -> Result<()> {
    if force {
        return Ok(());
    }
    if let Some(successor) = successor_of(&project.outcome.tasks, task) {
        return Err(Diag::refused("task already has a successor")
            .value(&task.id)
            .note(Note::new("successor").name(&successor.id))
            .hint(Hint::new("pass --force to spawn another")));
    }
    Ok(())
}

/// Build the new occurrence's file content: `source` (X's own) rewritten with
/// status/due/start, a `recurred-from(predecessor_stem)` edge, and the
/// carried-forward `count` (decremented).
fn spawn_source(
    source: &str,
    config: &mindtape::Config,
    dates: &OccurrenceDates,
    recur: &Recur,
    predecessor_stem: &str,
) -> Result<String> {
    let mut edits = status_due_start_edits(source, config, dates)?;
    edits.push(set_recurred_from(source, predecessor_stem)?);
    if let Some(edit) = decrement_count_edit(source, recur.count)? {
        edits.push(edit);
    }
    apply_edits(source, edits)
}

// ---- mutate: refresh the same file in place ---------------------------

fn do_mutate(
    project: &Project,
    task: &Task,
    abs: &Path,
    recur: &Recur,
    dry_run: bool,
) -> Result<()> {
    let (next, due, start, source) = plan_and_read(project, task, abs, recur)?;
    let occurrence_dates = OccurrenceDates {
        today: today_ymd(),
        old_due: due,
        old_start: start,
        next_due: next,
    };
    let mut edits = status_due_start_edits(&source, &project.config, &occurrence_dates)?;
    if let Some(edit) = decrement_count_edit(&source, recur.count)? {
        edits.push(edit);
    }
    let new_source = apply_edits(&source, edits)?;

    if dry_run {
        println!(
            "would refresh {} in place (due {})",
            abs.display(),
            recur_dates::format_iso(next)
        );
        return Ok(());
    }
    commit(project, abs, new_source)?;
    println!(
        "refreshed {} (due {})",
        abs.display(),
        recur_dates::format_iso(next)
    );
    Ok(())
}

// ---- skip: advance the anchor/count, materialize nothing --------------

fn do_skip(project: &Project, task: &Task, abs: &Path, recur: &Recur, dry_run: bool) -> Result<()> {
    let (next, due, start, source) = plan_and_read(project, task, abs, recur)?;
    let call = task_with(&source)?;
    let next_start = crate::cmd::recur::edit::shift_start(due, start, next);
    let mut edits = due_start_edits(&source, &call, next, next_start)?;
    if let Some(edit) = decrement_count_edit(&source, recur.count)? {
        edits.push(edit);
    }
    let new_source = apply_edits(&source, edits)?;

    if dry_run {
        println!(
            "would skip to due {} ({}), status unchanged",
            recur_dates::format_iso(next),
            abs.display()
        );
        return Ok(());
    }
    commit(project, abs, new_source)?;
    println!(
        "skipped to due {} ({})",
        recur_dates::format_iso(next),
        abs.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_advance;
