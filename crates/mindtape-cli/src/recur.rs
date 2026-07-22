//! `mt recur` — the sole recurrence writer: advance a recurring task to its
//! next occurrence.
//!
//! Dispatches on the task's effective `materialize` (`spawn`, the default, or
//! `mutate`; see `[tasks.recur] materialize`). `--dry-run` previews without
//! writing; `--advance N` pre-materializes `N` future spawns without closing
//! the current instance; `--skip` advances the anchor/`count` in place
//! without materializing anything. `mt ls`/`mt check` never call into this
//! module — see `truth/tasks/command-semantics-cqrs.typ`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use chrono::NaiveDate;
use clap::Args;
use mindtape::recur::{Recur, RecurConfig};
use mindtape::recur_dates::{self, effective_materialize, exhausted, next_date, past_until};
use mindtape::{Task, today_ymd};

use crate::add_values::slug;
use crate::recur_edit::{
    OccurrenceDates, append_recurred_from, apply_edits, decrement_count_edit, due_start_edits,
    status_due_start_edits,
};
use crate::task_edit::{task_with, verify_written};
use crate::util::{self, Project};

/// Built-in fallback stem template `mt recur` spawn uses when
/// `[tasks.naming] stem` is unset, so spawned occurrence stems stay unique
/// while the original authored file keeps its bare stem (see
/// `truth/tasks/configurable-task-stem-naming.typ`).
const RECUR_FALLBACK_STEM_TEMPLATE: &str = "{slug}-{date}";

/// Guard bound for walking a `recurred-from` chain (pathological/cyclic data
/// only — a real spawn chain is nowhere near this deep).
const MAX_CHAIN_DEPTH: u32 = 10_000;

/// Arguments for `mt recur`.
#[derive(Args)]
pub struct RecurArgs {
    /// Task to advance: handle, slug, relative path, or absolute path.
    pub task: String,

    /// Preview the spawn/mutate/advance/skip; write nothing.
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Pre-materialize N future instances (spawn only), without closing the
    /// current instance.
    #[arg(long, value_name = "N", conflicts_with = "skip")]
    pub advance: Option<u32>,

    /// Advance the anchor and `count`, creating/refreshing nothing.
    #[arg(long, conflicts_with = "advance")]
    pub skip: bool,

    /// Spawn another successor even if one already exists (bypasses the
    /// double-spawn guard).
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
    let project = util::open()?;
    util::warn_errors(&project.outcome);
    run_with(&project, args)
}

fn run_with(project: &Project, args: &RecurArgs) -> Result<()> {
    let (task, abs) = util::resolve_task(project, &args.task)?;
    let recur = task
        .recur
        .clone()
        .with_context(|| format!("{:?} has no `recur` field; nothing to advance", args.task))?;

    let recur_config = &project.config.tasks.recur;
    let problems = mindtape::recur::check_policy(&recur, recur_config);
    if !problems.is_empty() {
        bail!("{}", problems.join("; "));
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

/// Today, and the task's current `due`/`start`, as [`NaiveDate`]s.
fn dates(task: &Task) -> Result<(NaiveDate, Option<NaiveDate>, Option<NaiveDate>)> {
    let today = recur_dates::ymd_to_date(today_ymd()).context("today's date is out of range")?;
    let due = task.due.as_deref().and_then(recur_dates::parse_iso);
    let start = task.start.as_deref().and_then(recur_dates::parse_iso);
    Ok((today, due, start))
}

/// Compute the next date and enforce the series stop conditions
/// (`count`/`until`) before any write.
fn plan_next(
    recur: &Recur,
    config: &RecurConfig,
    today: NaiveDate,
    due: Option<NaiveDate>,
) -> Result<NaiveDate> {
    if exhausted(recur) {
        bail!("series exhausted: `count` has reached 0");
    }
    let next = next_date(recur, config, today, due).map_err(|err| anyhow!(err.to_string()))?;
    if past_until(recur, next) {
        bail!(
            "series ended: next occurrence {} is past `until`",
            recur_dates::format_iso(next)
        );
    }
    Ok(next)
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
    let source = std::fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    Ok((next, due, start, source))
}

/// The loaded task (if any) whose `recurred-from` edge names `target` — the
/// double-spawn guard.
fn successor_of<'a>(tasks: &'a [Task], target: &Task) -> Option<&'a Task> {
    mindtape::find_referrer(tasks, target, "recurred-from")
}

/// How many `recurred-from` hops precede `start` — the occurrence index the
/// naming minter's `{n}` macro resolves for the *next* spawn.
fn occurrence_index(tasks: &[Task], start: &Task) -> u32 {
    let mut depth = 0_u32;
    let mut current = start.clone();
    for _ in 0..MAX_CHAIN_DEPTH {
        match mindtape::resolve_edge(tasks, &current, "recurred-from") {
            Some(pred) => {
                depth += 1;
                current = pred.clone();
            }
            None => break,
        }
    }
    depth + 1
}

/// Mint (and disambiguate against `dest`) the next occurrence's stem: the
/// configured `[tasks.naming] stem` template, or [`RECUR_FALLBACK_STEM_TEMPLATE`]
/// when unset — the occurrence's anchor date and index, never today's clock.
fn mint_stem(
    project: &Project,
    task: &Task,
    anchor: NaiveDate,
    occurrence: u32,
    dest: &Path,
) -> Result<String> {
    let naming = &project.config.tasks.naming;
    let template = naming
        .stem
        .as_deref()
        .unwrap_or(RECUR_FALLBACK_STEM_TEMPLATE);
    let slug_value = slug(&task.title);
    let ctx = mindtape::naming::MintContext {
        slug: &slug_value,
        date: recur_dates::date_to_ymd(anchor),
        occurrence,
    };
    let stem =
        mindtape::naming::mint_stem(template, &ctx).map_err(|err| anyhow!(err.to_string()))?;
    mindtape::naming::disambiguate(&stem, |candidate| {
        dest.join(format!("{candidate}.typ")).exists()
    })
    .map_err(|err| anyhow!(err.to_string()))
}

fn stem_of(path: &Path) -> String {
    path.file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
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
    let dest = abs.parent().context("task file has no parent directory")?;
    let stem = mint_stem(project, task, next, occurrence, dest)?;
    let file = dest.join(format!("{stem}.typ"));

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
    util::write_task(&project.config, &file, new_source)?;
    println!(
        "spawned {} (due {})",
        file.display(),
        recur_dates::format_iso(next)
    );
    verify_written(&project.root, &file)
}

fn guard_double_spawn(project: &Project, task: &Task, force: bool) -> Result<()> {
    if force {
        return Ok(());
    }
    if let Some(successor) = successor_of(&project.outcome.tasks, task) {
        bail!(
            "{:?} already has a successor ({}); use --force to spawn another",
            task.id,
            successor.id
        );
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
    edits.push(append_recurred_from(source, predecessor_stem)?);
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
    util::write_task(&project.config, abs, new_source)?;
    println!(
        "refreshed {} (due {})",
        abs.display(),
        recur_dates::format_iso(next)
    );
    verify_written(&project.root, abs)
}

// ---- skip: advance the anchor/count, materialize nothing --------------

fn do_skip(project: &Project, task: &Task, abs: &Path, recur: &Recur, dry_run: bool) -> Result<()> {
    let (next, due, start, source) = plan_and_read(project, task, abs, recur)?;
    let call = task_with(&source)?;
    let next_start = crate::recur_edit::shift_start(due, start, next);
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
    util::write_task(&project.config, abs, new_source)?;
    println!(
        "skipped to due {} ({})",
        recur_dates::format_iso(next),
        abs.display()
    );
    verify_written(&project.root, abs)
}

// ---- advance: pre-materialize N future spawns (spawn only) -------------

fn do_advance(
    project: &Project,
    task: &Task,
    abs: &Path,
    recur: &Recur,
    args: &RecurArgs,
) -> Result<()> {
    let count = args.advance.context("--advance requires a count")?;
    check_advance_preconditions(project, task, recur, args, count)?;
    let spawned = build_advance_chain(project, task, abs, recur, count)?;
    write_advance_chain(project, &spawned, args.dry_run)
}

/// The checks `--advance` needs before touching anything: `spawn`-only,
/// bounded by `[tasks.recur] max-advance`, and the double-spawn guard.
fn check_advance_preconditions(
    project: &Project,
    task: &Task,
    recur: &Recur,
    args: &RecurArgs,
    count: u32,
) -> Result<()> {
    let recur_config = &project.config.tasks.recur;
    let materialize = effective_materialize(recur, recur_config);
    if materialize != "spawn" {
        bail!(
            "--advance only supports spawn materialization (this task's materialize is `{materialize}`)"
        );
    }
    if count > recur_config.max_advance {
        bail!(
            "--advance {count} exceeds [tasks.recur] max-advance ({})",
            recur_config.max_advance
        );
    }
    guard_double_spawn(project, task, args.force)
}

/// Chain `count` future spawns forward from `task` in memory: each entry's
/// content already carries its own `recurred-from` back to the previous one.
/// Nothing is written yet — [`write_advance_chain`] decides dry-run vs write.
/// Stops early (without erroring) once a stop condition trips on a
/// non-empty chain, since the earlier spawns are still a valid partial batch.
fn build_advance_chain(
    project: &Project,
    task: &Task,
    abs: &Path,
    recur: &Recur,
    count: u32,
) -> Result<Vec<(PathBuf, NaiveDate, String)>> {
    let recur_config = &project.config.tasks.recur;
    let dest = abs.parent().context("task file has no parent directory")?;
    let (today, mut due_cursor, mut start_cursor) = dates(task)?;
    let mut current = recur.clone();
    let mut occurrence = occurrence_index(&project.outcome.tasks, task);
    let mut predecessor_stem = stem_of(&task.file);
    let mut predecessor_source =
        std::fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    let mut spawned: Vec<(PathBuf, NaiveDate, String)> = Vec::new();

    for _ in 0..count {
        let next = match plan_next(&current, recur_config, today, due_cursor) {
            Ok(next) => next,
            Err(err) if !spawned.is_empty() => {
                eprintln!("warning: stopping early: {err:#}");
                break;
            }
            Err(err) => return Err(err),
        };
        let stem = mint_stem(project, task, next, occurrence, dest)?;
        let file = dest.join(format!("{stem}.typ"));
        let occurrence_dates = OccurrenceDates {
            today: today_ymd(),
            old_due: due_cursor,
            old_start: start_cursor,
            next_due: next,
        };
        let new_source = spawn_source(
            &predecessor_source,
            &project.config,
            &occurrence_dates,
            &current,
            &predecessor_stem,
        )?;

        spawned.push((file, next, new_source.clone()));
        start_cursor = crate::recur_edit::shift_start(due_cursor, start_cursor, next);
        due_cursor = Some(next);
        current.count = recur_dates::carry_count(&current);
        predecessor_stem = stem;
        predecessor_source = new_source;
        occurrence += 1;
    }
    Ok(spawned)
}

/// Print a preview (dry-run) or write and verify each chained spawn.
fn write_advance_chain(
    project: &Project,
    spawned: &[(PathBuf, NaiveDate, String)],
    dry_run: bool,
) -> Result<()> {
    for (file, next, source) in spawned {
        if dry_run {
            println!(
                "would spawn {} (due {})",
                file.display(),
                recur_dates::format_iso(*next)
            );
        } else {
            util::write_task(&project.config, file, source.clone())?;
            println!(
                "spawned {} (due {})",
                file.display(),
                recur_dates::format_iso(*next)
            );
        }
    }
    if !dry_run {
        for (file, ..) in spawned {
            verify_written(&project.root, file)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "recur_tests.rs"]
mod tests;
