//! `mt recur` — the sole recurrence writer: advance a recurring task to its
//! next occurrence.
//!
//! Dispatches on the task's effective `materialize` (`spawn`, the default, or
//! `mutate`; see `[tasks.recur] materialize`). `--dry-run` previews without
//! writing; `--advance N` pre-materializes `N` future spawns without closing
//! the current instance; `--skip` advances the anchor/`count` in place
//! without materializing anything. `mt ls`/`mt check` never call into this
//! module — see `truth/tasks/command-semantics-cqrs.typ`.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use chrono::NaiveDate;
use clap::Args;
use mindtape::recur::{Recur, RecurConfig};
use mindtape::recur_dates::{
    self, advance_once, effective_materialize, exhausted, next_date, past_until,
};
use mindtape::{Task, today_ymd};

use crate::add_values::slug;
use crate::pre_write::{self, Pending};
use crate::recur_edit::{
    OccurrenceDates, apply_edits, decrement_count_edit, due_start_edits, set_recurred_from,
    status_due_start_edits,
};
use crate::task_edit::task_with;
use crate::util::{self, Project};

#[path = "recur_advance.rs"]
mod advance;
use advance::do_advance;
// Only reachable from the `--advance` test module below (glob-imported via
// `super::*`); `recur.rs` itself calls `advance::do_advance` alone; `--advance`
// dispatches everything else internally.
#[cfg(test)]
use advance::{build_advance_chain, check_advance_preconditions, write_advance_chain};

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
    #[command(flatten)]
    pub task: crate::task_ref::TaskRef,

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
    let (task, abs) = util::resolve_task(project, &args.task.task)?;
    let recur = task.recur.clone().with_context(|| {
        format!(
            "{:?} has no `recur` field; nothing to advance",
            args.task.task
        )
    })?;

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

/// Bail unless the series still has budget left — checked before a date is
/// computed, since an exhausted series has no next occurrence to compute.
fn guard_budget(recur: &Recur) -> Result<()> {
    if exhausted(recur) {
        bail!("series exhausted: `count` has reached 0");
    }
    Ok(())
}

/// Bail if `next` overshoots the series' `until` bound; hand it back otherwise.
fn guard_until(recur: &Recur, next: NaiveDate) -> Result<NaiveDate> {
    if past_until(recur, next) {
        bail!(
            "series ended: next occurrence {} is past `until`",
            recur_dates::format_iso(next)
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
    let next = next_date(recur, config, today, due).map_err(|err| anyhow!(err.to_string()))?;
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
pub(super) fn plan_step(recur: &Recur, from: NaiveDate) -> Result<NaiveDate> {
    guard_budget(recur)?;
    let next = advance_once(&recur.rule, from).map_err(|err| anyhow!(err.to_string()))?;
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

/// Everything a minted stem must not collide with: the destination directory
/// (the `.typ` files already on disk) and the stems a chain in flight has
/// claimed but not yet written.
///
/// The two travel together because a `--advance` chain lives entirely in
/// memory until `write_advance_chain` lands it, so disk alone answers "is this
/// name taken?" with `no` for every entry of the batch — the same shape
/// `mt import` solves with its `existing_stems` seed. Bundled into one
/// argument rather than passed as a sixth: [`mint_stem`] is at clippy's
/// 5-argument limit.
pub(super) struct StemDest<'a> {
    /// The directory the occurrence file will be written into.
    pub dir: &'a Path,
    /// Stems already claimed by this batch, none of them on disk yet.
    pub claimed: &'a HashSet<String>,
}

/// Mint (and disambiguate against `dest`) the next occurrence's stem: the
/// configured `[tasks.naming] stem` template, or [`RECUR_FALLBACK_STEM_TEMPLATE`]
/// when unset — the occurrence's anchor date and index, never today's clock.
fn mint_stem(
    project: &Project,
    task: &Task,
    anchor: NaiveDate,
    occurrence: u32,
    dest: &StemDest<'_>,
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
        dest.claimed.contains(candidate) || dest.dir.join(format!("{candidate}.typ")).exists()
    })
    .map_err(|err| anyhow!(err.to_string()))
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
    let dir = abs.parent().context("task file has no parent directory")?;
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
    commit(project, abs, new_source)?;
    println!(
        "skipped to due {} ({})",
        recur_dates::format_iso(next),
        abs.display()
    );
    Ok(())
}

#[cfg(test)]
#[path = "recur_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "recur_tests_advance.rs"]
mod tests_advance;
