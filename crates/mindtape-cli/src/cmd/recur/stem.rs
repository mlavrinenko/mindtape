//! Naming the next occurrence of a recurring task: the `recurred-from` chain
//! walk that numbers it, the stem minted for it, and the guard that keeps a
//! stem from colliding with a file on disk or one in flight.

use std::collections::HashSet;
use std::path::Path;

use chrono::NaiveDate;
use mindtape::Task;
use mindtape::recur::dates as recur_dates;

use crate::cmd::add::values::slug;
use crate::diag::{Diag, Result};
use crate::util::Project;

/// Built-in fallback stem template `mt recur` spawn uses when
/// `[tasks.naming] stem` is unset, so spawned occurrence stems stay unique
/// while the original authored file keeps its bare stem (see
/// `truth/tasks/configurable-task-stem-naming.typ`).
const RECUR_FALLBACK_STEM_TEMPLATE: &str = "{slug}-{date}";

/// Guard bound for walking a `recurred-from` chain (pathological/cyclic data
/// only — a real spawn chain is nowhere near this deep).
const MAX_CHAIN_DEPTH: u32 = 10_000;

/// The loaded task (if any) whose `recurred-from` edge names `target` — the
/// double-spawn guard.
pub(super) fn successor_of<'a>(tasks: &'a [Task], target: &Task) -> Option<&'a Task> {
    mindtape::find_referrer(tasks, target, "recurred-from")
}

/// How many `recurred-from` hops precede `start` — the occurrence index the
/// naming minter's `{n}` macro resolves for the *next* spawn.
pub(super) fn occurrence_index(tasks: &[Task], start: &Task) -> u32 {
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
    pub(super) dir: &'a Path,
    /// Stems already claimed by this batch, none of them on disk yet.
    pub(super) claimed: &'a HashSet<String>,
}

/// Mint (and disambiguate against `dest`) the next occurrence's stem: the
/// configured `[tasks.naming] stem` template, or [`RECUR_FALLBACK_STEM_TEMPLATE`]
/// when unset — the occurrence's anchor date and index, never today's clock.
pub(super) fn mint_stem(
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
    let ctx = mindtape::model::naming::MintContext {
        slug: &slug_value,
        date: recur_dates::date_to_ymd(anchor),
        occurrence,
    };
    let stem = mindtape::model::naming::mint_stem(template, &ctx)
        .map_err(|err| Diag::failed("can't mint a stem").cause(&err))?;
    mindtape::model::naming::disambiguate(&stem, |candidate| {
        dest.claimed.contains(candidate) || dest.dir.join(format!("{candidate}.typ")).exists()
    })
    .map_err(|err| Diag::failed("can't mint a stem").cause(&err))
}

pub(super) fn stem_of(path: &Path) -> String {
    path.file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
}
