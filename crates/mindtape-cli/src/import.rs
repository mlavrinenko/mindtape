//! `mt import` — read canonical task-JSON (a file arg or stdin `-`) and write
//! native `.typ` tasks: map each record onto this project's config policy
//! (`mindtape::map_canonical`), emit it (`mindtape::emit_task`), land the
//! whole batch, validate it as a set, and report. "`mt add` without a
//! template, from a full record" — reuses `mt add`'s destination resolution
//! (`--into`/`--to`) and dedupe.
//!
//! One bad record does not refuse the batch: the rest still land, and the
//! per-record report says which did, and the run exits 1 once it has printed it
//! if any was skipped. What it no longer does is land the bad
//! one first — the whole batch is validated before a byte is written, so a
//! rejected record is never created. That discipline lives in [`write`]; input
//! parsing lives in [`input`].
//!
//! No `--from`/`--via` source selector: sources are jq adapters, upstream of
//! the pipe (`truth/tasks/import-tasks-from-other-trackers.typ`).

mod input;
mod place;
mod write;

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;

use anyhow::{Context, Result, anyhow, bail};
use clap::Args;
use mindtape::canonical::CanonicalStatus;
use mindtape::{CanonicalTask, Config, EntryKind, MapOptions, map_canonical};

use crate::util::{self, Project};
use place::Placement;
use write::{StagedTask, TaskOutcome};

/// Arguments for `mt import`.
#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Canonical task-JSON source: a file path, or `-` (default) for stdin.
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: String,

    /// Destination directory for the whole batch; `.` is the first configured
    /// root. Without `--into` and `--to`, a record lands in the folder its
    /// `file` names when that lies under a configured root, else in the first
    /// root.
    #[arg(long, value_name = "DIR")]
    pub into: Option<String>,

    /// Subfolder under the destination root to nest imported tasks into
    /// (folder-derived tags). Relative to `--into`; it overrides each record's
    /// `file`.
    #[arg(long = "to", value_name = "SUBDIR")]
    pub to: Option<String>,

    /// Report and preview only; write nothing to the destination.
    #[arg(long)]
    pub dry_run: bool,

    /// Default status name (dated today) stamped on a record missing one.
    #[arg(long, value_name = "NAME")]
    pub status: Option<String>,

    /// Status rename `OLD=NEW` (repeatable), applied before legend validation.
    #[arg(long = "map", value_name = "OLD=NEW")]
    pub map: Vec<String>,

    /// Drop a tag outside `[tasks.tags] allowed` instead of keeping it and
    /// reporting a violation.
    #[arg(long)]
    pub drop_unmapped: bool,
}

/// Run `mt import`.
///
/// # Errors
/// Returns an error if the project cannot be opened, the input cannot be read,
/// or `--map` is malformed.
pub fn run(args: &ImportArgs) -> Result<()> {
    run_with(&util::open()?, args)
}

fn run_with(project: &Project, args: &ImportArgs) -> Result<()> {
    let raw = input::read_source(&args.input)?;
    let outcomes = import_records(project, args, &raw)?;
    report(&outcomes, args.dry_run);
    fail_on_skips(&outcomes)
}

/// The closing verdict: an error naming the skip count when any record was
/// skipped, so a script can tell a partial import from a whole one. The
/// records that passed are already written by then.
fn fail_on_skips(outcomes: &[TaskOutcome]) -> Result<()> {
    let skipped = outcomes.iter().filter(|outcome| !outcome.filed).count();
    if skipped == 0 {
        return Ok(());
    }
    let problems = outcomes.iter().flat_map(|outcome| &outcome.problems);
    if let Some(hint) = crate::install_hint::batch_prelude_hint(problems.map(String::as_str)) {
        eprintln!("{hint}");
    }
    bail!("{skipped} record(s) skipped");
}

/// The core pipeline: parse `raw` canonical JSON, map/stem/emit each record,
/// then land (or, under `--dry-run`, preview) the batch. Split out from
/// [`run_with`] so tests can drive it without stdin/file plumbing.
///
/// # Errors
/// Returns an error if `raw` cannot be parsed, a `--map` entry is malformed,
/// or the destination cannot be created/written.
fn import_records(project: &Project, args: &ImportArgs, raw: &str) -> Result<Vec<TaskOutcome>> {
    let config = &project.config;
    let records = input::parse_records(raw)?;
    let mut placement = Placement::new(project, args)?;
    let opts = map_options(args)?;

    let mut staged = Vec::with_capacity(records.len());
    let mut early = Vec::new();

    for (idx, mut record) in records.into_iter().enumerate() {
        let label = record_label(&record, idx);
        default_status(&mut record, args.status.as_deref());
        let mapped = map_canonical(&record, config, &opts);
        let dest = placement.dest_for(&record);
        match stem_for(&record.id, &mapped.task.title, config, placement.taken()) {
            Ok(stem) => {
                let mut task = mapped.task;
                task.id.clone_from(&stem);
                let source = mindtape::emit_task(&task);
                staged.push(StagedTask {
                    label,
                    dest,
                    stem,
                    source,
                    report: mapped.report,
                });
            }
            Err(err) => early.push(TaskOutcome {
                label,
                stem: String::new(),
                filed: false,
                problems: vec![err.to_string()],
                report: mapped.report,
            }),
        }
    }

    let mut outcomes = if args.dry_run {
        write::preview(project, staged)?
    } else {
        write::land(project, staged)?
    };
    outcomes.extend(early);
    Ok(outcomes)
}

/// Build [`MapOptions`] from `--map`/`--drop-unmapped`.
fn map_options(args: &ImportArgs) -> Result<MapOptions> {
    let mut status_map = BTreeMap::new();
    for entry in &args.map {
        let (old, new) = entry
            .split_once('=')
            .with_context(|| format!("--map expects OLD=NEW, got {entry:?}"))?;
        if old.is_empty() || new.is_empty() {
            bail!("--map expects OLD=NEW, got {entry:?}");
        }
        status_map.insert(old.to_owned(), new.to_owned());
    }
    Ok(MapOptions {
        status_map,
        drop_unmapped_tags: args.drop_unmapped,
    })
}

/// A human label for reporting: the authored `id`, else the title, else a
/// positional fallback (`idx` is 0-based).
fn record_label(record: &CanonicalTask, idx: usize) -> String {
    if !record.id.is_empty() {
        record.id.clone()
    } else if !record.title.is_empty() {
        record.title.clone()
    } else {
        format!("record #{}", idx + 1)
    }
}

/// Stamp `status` (dated today) onto a record missing one, when `--status`
/// supplies a default. Applied before [`map_canonical`], so the value flows
/// through the same `--map`/legend checks as an authored status; without
/// `--status`, `map_canonical` applies its own default (the legend's initial
/// value).
fn default_status(record: &mut CanonicalTask, status: Option<&str>) {
    if record.status.is_some() {
        return;
    }
    let Some(name) = status else { return };
    record.status = Some(CanonicalStatus {
        name: name.to_owned(),
        date: Some(mindtape::today_iso()),
        note: None,
    });
}

/// The destination stem for a record: its `id` when that is a usable file stem
/// (an export carries it, and links between records name it), else the
/// deterministic candidate from `title` ([`mindtape::stem_for_title`], what a
/// jq adapter's id-less record gets). Either is disambiguated against
/// `existing` (every loaded task's stem, the destination files and every stem
/// already claimed within this batch). `existing` gains the chosen stem so later records in the
/// same batch never collide with it — the collision policy is a `-2`, `-3`, ...
/// suffix, [`mindtape::naming::disambiguate`]'s existing behaviour.
///
/// # Errors
/// Returns an error when there is no usable `id` and `title` slugifies to
/// nothing, the configured `[tasks.naming] stem` template is malformed, or
/// disambiguation exhausts its numeric-suffix ceiling.
fn stem_for(
    id: &str,
    title: &str,
    config: &Config,
    existing: &mut HashSet<String>,
) -> Result<String> {
    let candidate = if usable_stem(id) {
        id.to_owned()
    } else {
        if mindtape::slug_from_title(title).is_empty() {
            bail!("empty title; nothing to name the task file from");
        }
        mindtape::stem_for_title(title, config, mindtape::today_ymd())
            .map_err(|err| anyhow!(err.to_string()))?
    };
    let stem = mindtape::naming::disambiguate(&candidate, |c| existing.contains(c))
        .map_err(|err| anyhow!(err.to_string()))?;
    existing.insert(stem.clone());
    Ok(stem)
}

/// Whether `id` can be a bare file stem: non-empty, no path separator, no dot.
fn usable_stem(id: &str) -> bool {
    !id.is_empty() && !id.contains(['/', '\\', '.'])
}

/// Print "N filed, M skipped" plus, per record, its label, outcome, mapping
/// report entries (mapped/defaulted/dropped/violated), carried `x-<ns>`
/// extras, and any load/validate problem — to stderr, mirroring `mt add`'s
/// warnings-to-stderr convention. Under `--dry-run` the wording is `would
/// file` and the report ends with `dry run: nothing written`, as `mt mv` does.
fn report(outcomes: &[TaskOutcome], dry_run: bool) {
    eprint!("{}", render_report(outcomes, dry_run));
}

fn render_report(outcomes: &[TaskOutcome], dry_run: bool) -> String {
    let filed = outcomes.iter().filter(|outcome| outcome.filed).count();
    let verb = if dry_run { "would file" } else { "filed" };
    let mut out = format!("{filed} {verb}, {} skipped\n", outcomes.len() - filed);
    for outcome in outcomes {
        render_task(&mut out, outcome, verb);
    }
    if dry_run {
        out.push_str("dry run: nothing written\n");
    }
    out
}

fn render_task(out: &mut String, outcome: &TaskOutcome, filed_verb: &str) {
    let verb = if outcome.filed { filed_verb } else { "skipped" };
    let renamed = if outcome.filed && outcome.stem != outcome.label {
        format!(" as {}", outcome.stem)
    } else {
        String::new()
    };
    let _ = writeln!(out, "  {}: {verb}{renamed}", outcome.label);
    for entry in &outcome.report.entries {
        let _ = writeln!(
            out,
            "    [{}] {}: {}",
            entry_kind_label(entry.kind),
            entry.field,
            entry.message
        );
    }
    if !outcome.report.carried_extras.is_empty() {
        let _ = writeln!(
            out,
            "    extras carried: {}",
            outcome.report.carried_extras.join(", ")
        );
    }
    for problem in &outcome.problems {
        let _ = writeln!(out, "    - {problem}");
    }
}

fn entry_kind_label(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Mapped => "mapped",
        EntryKind::Defaulted => "defaulted",
        EntryKind::Dropped => "dropped",
        EntryKind::Violated => "violated",
    }
}

#[cfg(test)]
#[path = "import_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "import_tests_helpers.rs"]
mod tests_helpers;
