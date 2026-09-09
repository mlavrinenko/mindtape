//! `mt recur --advance N` — pre-materialize N future spawns forward from a
//! task, without closing the current instance. Split out of `recur.rs` to
//! stay under the file-size gate: the single-step materialize paths
//! (`spawn`/`mutate`/`skip`) and the planning primitives this chain builds on
//! (`dates`, `plan_next`, `mint_stem`, `spawn_source`, `occurrence_index`,
//! `guard_double_spawn`) stay there, since `do_spawn` leans on the same
//! toolkit — `--advance` is the one materialize path that walks it in a loop
//! instead of once.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use mindtape::Task;
use mindtape::recur::Recur;
use mindtape::recur_dates::{self, effective_materialize};
use mindtape::today_ymd;

use super::{
    RecurArgs, StemDest, dates, guard_double_spawn, mint_stem, occurrence_index, plan_next,
    plan_step, spawn_source, stem_of,
};
use crate::pre_write::{self, Pending};
use crate::recur_edit::OccurrenceDates;
use crate::util::{self, Project};

pub(super) fn do_advance(
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
pub(super) fn check_advance_preconditions(
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
///
/// The first hop is anchor-aware (`plan_next`) and every hop after it steps
/// off the previous occurrence's date instead (`plan_step`) — see that
/// function for why an anchor cannot be re-asked mid-chain. Each minted stem
/// is claimed as it is planned, so a later hop landing on the same name
/// (a `{slug}`-only template, or a rule that repeats a date) gets the
/// `-2`/`-3` suffix rather than the file the hop before it already spoke for.
pub(super) fn build_advance_chain(
    project: &Project,
    task: &Task,
    abs: &Path,
    recur: &Recur,
    count: u32,
) -> Result<Vec<(PathBuf, NaiveDate, String)>> {
    let recur_config = &project.config.tasks.recur;
    let dir = abs.parent().context("task file has no parent directory")?;
    let (today, mut due_cursor, mut start_cursor) = dates(task)?;
    let mut current = recur.clone();
    let mut occurrence = occurrence_index(&project.outcome.tasks, task);
    let mut predecessor_stem = stem_of(&task.file);
    let mut predecessor_source =
        std::fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    let mut claimed: HashSet<String> = HashSet::new();
    let mut chain_cursor: Option<NaiveDate> = None;
    let mut spawned: Vec<(PathBuf, NaiveDate, String)> = Vec::new();

    for _ in 0..count {
        let planned = match chain_cursor {
            None => plan_next(&current, recur_config, today, due_cursor),
            Some(from) => plan_step(&current, from),
        };
        let next = match planned {
            Ok(next) => next,
            Err(err) if !spawned.is_empty() => {
                eprintln!("warning: stopping early: {err:#}");
                break;
            }
            Err(err) => return Err(err),
        };
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
        claimed.insert(stem.clone());
        let file = dir.join(format!("{stem}.typ"));
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
        chain_cursor = Some(next);
        current.count = recur_dates::carry_count(&current);
        predecessor_stem = stem;
        predecessor_source = new_source;
        occurrence += 1;
    }
    Ok(spawned)
}

/// Print a preview (dry-run), or check the whole chain and write it.
///
/// The check is all-or-nothing, and the chain is why: every occurrence's source
/// is the previous one rewritten, and each names its predecessor through a
/// `recurred-from` edge. So the members are only valid as a set — a lone
/// occurrence whose predecessor was refused points at a file that does not
/// exist — and one rejected entry condemns every entry after it. Refusing the
/// batch writes nothing at all, which is the only outcome with no half-chain to
/// unpick afterwards.
pub(super) fn write_advance_chain(
    project: &Project,
    spawned: &[(PathBuf, NaiveDate, String)],
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        for (file, next, _) in spawned {
            println!(
                "would spawn {} (due {})",
                file.display(),
                recur_dates::format_iso(*next)
            );
        }
        return Ok(());
    }

    let pending: Vec<Pending<'_>> = spawned
        .iter()
        .map(|(file, _, source)| Pending { file, source })
        .collect();
    pre_write::ensure(project, &pending)?;

    for (file, next, source) in spawned {
        util::write_task(&project.config, file, source.clone())?;
        println!(
            "spawned {} (due {})",
            file.display(),
            recur_dates::format_iso(*next)
        );
    }
    Ok(())
}
