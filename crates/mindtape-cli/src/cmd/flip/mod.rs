//! `mt flip <status> <task>...` — stamp a new status onto tasks.
//!
//! Mutates each task's `status:` to `<status>` dated today, the same shape
//! `add` stamps the initial status (shortcut `name(y, m, d)` for a prelude name,
//! generic `status("name", y, m, d)` otherwise — both via
//! [`mindtape::emit::status_value`]). The status must be a configured legend value.
//!
//! Every legend value also doubles as a top-level shortcut verb dispatched here:
//! `mt done a7k x3m` == `mt flip done a7k x3m` (see [`run_external`]).
//! Built-in subcommands win on a name clash; a shadowing legend value is warned
//! about. `mt flip --list` prints the available shortcut verbs.
//!
//! Batch semantics mirror `rm`: every target is resolved before any file is
//! written, then the valid edits are applied and any resolution or post-edit
//! validation problem is reported without rolling the rest back.
//!
//! A landed batch closes with one post-write revalidation over a single reload
//! ([`crate::cmd::flip::batch`]).
//!
//! `flip` is also the one command that runs the project's `[[on.flip]]` actions
//! ([`crate::cmd::flip::hooks`]). They run per target, after the transition and
//! note-requirement guards and before the write, so a refusing gate leaves that
//! task's file untouched.
//!
//! # Files
//!
//! - `batch.rs` — the post-write revalidation of a landed batch, and the
//!   batch's error report.
//! - `guards.rs` — the transition and note guards cleared before a write.
//! - `hooks.rs` — the `[[on.flip]]` action runner.
//! - `hooks_cmd.rs` — executing a `run` action's command.
//! - `verbs.rs` — the shortcut-verb errors, shadow warning and `--list`.
//! - `tests*.rs`, `*_tests.rs` — unit tests.

pub(crate) mod batch;
pub(crate) mod hooks;
pub(crate) mod hooks_cmd;
mod verbs;
use verbs::{list_verbs, unknown_command, unknown_status, warn_shadowed_verbs};

use std::io::IsTerminal;
use std::path::Path;

use clap::{Args, CommandFactory, FromArgMatches, Parser};
use mindtape::config::Legend;
use mindtape::{status_value, today_ymd};
use typst_edit::apply;

use crate::cmd::flip::batch::{Landed, after_batch_diags, finish, note_hint};
use crate::cmd::flip::hooks::{Hooks, Transition, note_with_overrides};
use crate::diag::{Diag, Hint, Note, Result};
use crate::task_edit::{task_with, upsert_named_arg};
use crate::util::{self, Project};

/// Arguments for `mt flip`.
#[derive(Args)]
pub struct FlipArgs {
    /// The new status, one of those your project defines.
    #[arg(required_unless_present = "list")]
    pub status: Option<String>,

    /// The tasks to change, each by stem, hash or path.
    pub tasks: Vec<String>,

    /// A note on the new status. Without it, mt clears the old status's note.
    #[arg(short = 'm', long = "note", value_name = "NOTE")]
    pub note: Option<String>,

    /// List your project's statuses, each usable as a command, and exit.
    #[arg(long)]
    pub list: bool,

    /// Allow a status change your project's rules forbid, and one a failing
    /// `[[on.flip]]` check would block. The status note records a skipped check.
    #[arg(long)]
    pub force: bool,

    /// Don't print what `[[on.flip]]` hooks show. A failing check still prints.
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

// Parser wrapper for the shortcut-verb path: the raw external argv
// (`<status> <task>...`, plus any `-m`) re-parsed into [`FlipArgs`]. Not a doc
// comment, because clap would print it as the verb's help.
#[derive(Parser)]
#[command(no_binary_name = true)]
struct Shortcut {
    #[command(flatten)]
    args: FlipArgs,
}

/// Run the canonical `flip` subcommand.
///
/// # Errors
/// Returns an error if the project cannot be opened, the status is not a legend
/// value, or any target fails to resolve or validate after the edit.
pub fn run(args: &FlipArgs) -> Result<()> {
    run_as(&util::open_warned()?, args, Via::Flip)
}

/// How the user spelled the command, so a hint can repeat it.
#[derive(Clone, Copy)]
pub(crate) enum Via {
    /// `mt flip <status> …`
    Flip,
    /// `mt <status> …`
    Shortcut,
}

/// Dispatch a per-status shortcut verb: `external` is the raw `[verb, args…]`
/// captured by the `external_subcommand` fallback, where `verb` is the status.
///
/// # Errors
/// Returns an error if `verb` is not a configured legend value (an unknown
/// command) or the underlying flip fails. Args that do not parse exit through
/// clap with its usage error.
pub fn run_external(external: &[String]) -> Result<()> {
    let project = util::open_warned()?;
    let verb = external
        .first()
        .ok_or_else(|| Diag::usage("empty shortcut invocation"))?
        .as_str();
    let legend = &project.config.tasks.status;
    if !legend.contains(verb) {
        return Err(unknown_command(verb, legend));
    }
    // clap prints a help request to stdout and exits 0, a usage error to
    // stderr and exits 2, as `mt flip` does.
    let matches = shortcut_command(verb)
        .try_get_matches_from(external)
        .unwrap_or_else(|err| err.exit());
    let parsed = Shortcut::from_arg_matches(&matches)?;
    run_as(&project, &parsed.args, Via::Shortcut)
}

/// The parser for `mt <verb> …`, presented as that verb: named `mt <verb>`,
/// described in its own words, with the status positional (the verb itself,
/// already given) left out of the usage.
pub(crate) fn shortcut_command(verb: &str) -> clap::Command {
    Shortcut::command()
        .name(verb.to_owned())
        .bin_name(format!("mt {verb}"))
        .about(format!(
            "Set tasks' status to `{verb}`. Same as `mt flip {verb} <task>...`"
        ))
        .mut_arg("status", |arg| arg.hide(true).index(1))
        .mut_arg("tasks", |arg| arg.index(2))
}

/// Flip `args.status` onto every resolved task in `project`.
#[cfg(test)]
pub(crate) fn run_with(project: &Project, args: &FlipArgs) -> Result<()> {
    run_as(project, args, Via::Flip)
}

fn run_as(project: &Project, args: &FlipArgs, via: Via) -> Result<()> {
    warn_shadowed_verbs(&project.config);

    if args.list {
        list_verbs(&project.config);
        return Ok(());
    }

    let status = args
        .status
        .as_deref()
        .ok_or_else(|| Diag::usage("status is required"))?;
    if args.tasks.is_empty() {
        return Err(Diag::usage("no tasks given")
            .hint(Hint::new("name at least one task after the status")));
    }
    let legend = &project.config.tasks.status;
    if !legend.is_empty() && !legend.contains(status) {
        return Err(unknown_status(status, legend));
    }

    // Resolve every target before mutating, so each reference reads the project
    // the caller listed.
    let refs = util::Refs::new(project)?;
    let mut resolved = Vec::new();
    let mut problems: Vec<Diag> = Vec::new();
    for spec in &args.tasks {
        match refs.task(spec) {
            Ok(found) => resolved.push((spec.clone(), found.task.clone(), found.path)),
            Err(err) => problems.push(note_hint(err.into(), spec, args, via)),
        }
    }

    let batch = Batch {
        project,
        args,
        status,
        hooks: &Hooks::new(
            project,
            hooks_enabled(args.quiet, std::io::stdout().is_terminal()),
        ),
        ymd: today_ymd(),
    };
    // The project as this batch's own writes have already left it. Each
    // accepted flip is folded back in, so the next target is checked against a
    // project that carries the earlier ones — a flip that unblocks a later
    // target must not have that target refused for a violation it has already
    // cured.
    let mut world = project.outcome.tasks.clone();
    let mut landed = Landed::default();
    let mut flipped = 0;
    for (spec, task, abs) in &resolved {
        match flip_target(&batch, &mut world, spec, task, abs) {
            Ok(()) => {
                landed.record(task);
                flipped += 1;
            }
            Err(problem) => problems.push(name_target(problem, spec, args.tasks.len())),
        }
    }

    if !landed.is_empty() {
        after_batch_diags(&project.root, &landed, &mut problems)?;
    }
    finish(problems, flipped, args.tasks.len())
}

/// `problem` with the task it is about named, when the batch has several.
fn name_target(problem: Diag, spec: &str, targets: usize) -> Diag {
    if targets > 1 {
        problem.note(Note::new("task").name(spec))
    } else {
        problem
    }
}

/// The pre-write guards; split out to keep this file under its line budget.
/// See [`check_transition`].
mod guards;
use guards::{check_children, check_note_required, check_transition, check_unblocked};

/// The parts of a `flip` run that every target in the batch shares.
struct Batch<'a> {
    project: &'a Project,
    args: &'a FlipArgs,
    /// The status being stamped.
    status: &'a str,
    /// The project's `[[on.flip]]` runner.
    hooks: &'a Hooks<'a>,
    /// Today, resolved once so a batch cannot straddle midnight.
    ymd: (i32, u8, u8),
}

/// Guard, hook and write one target. `Ok(())` when the flip landed;
/// `Err(diag)` says why it did not — the batch reports it and moves on, so a
/// refused target leaves its own file untouched without rolling back the rest.
///
/// Order matters: the transition graph and the note requirement are cheap,
/// local checks, so they run before any configured command does.
fn flip_target(
    batch: &Batch<'_>,
    world: &mut Vec<mindtape::Task>,
    spec: &str,
    task: &mindtape::Task,
    abs: &Path,
) -> std::result::Result<(), Diag> {
    let (args, status) = (batch.args, batch.status);
    let legend = &batch.project.config.tasks.status;
    if !args.force {
        check_transition(legend, task.status.as_ref(), status)?;
        let blockers =
            mindtape::blockers_of(world, &asked(task, batch.project), &batch.project.config);
        check_unblocked(legend, status, &blockers)?;
        check_children(world, task, status, &batch.project.config)?;
    }
    check_note_required(legend, status, args.note.as_deref())?;
    let outcome = batch.hooks.run(
        &Transition {
            task,
            path: abs,
            target: status,
            force: args.force,
        },
        &mut std::io::stdout(),
    )?;
    if let Some(refusal) = outcome.refusal {
        return Err(refusal);
    }
    let note = note_with_overrides(args.note.as_deref(), &outcome.forced);
    let restamp = task.status.as_ref().is_some_and(|s| s.name == status);
    flip_one(batch, world, abs, note.as_deref())?;
    report_flip(spec, &task.title, task.status.as_ref(), status, restamp);
    notice_recur(task, status, legend, &batch.project.config.tasks.recur);
    Ok(())
}

/// `task` with every `waits-on` entry answered. The loader asks only for open
/// tasks, so a flip out of a terminal status asks here before its guard reads
/// the answers.
fn asked(task: &mindtape::Task, project: &Project) -> mindtape::Task {
    let mut task = task.clone();
    mindtape::waits::ask(&mut task, &project.config, &project.root);
    task
}

/// Rewrite the task at `abs`: stamp the batch's status dated its `ymd` (with
/// optional `note`), check the result at that same path, and write it only if
/// it passes. The re-stamp determination (did the task already carry the
/// status) is the caller's, computed before the write.
///
/// A refused flip is not written, and so is not restored either — the file
/// still holds exactly what its author put there. `after_batch`'s revalidation
/// stays as the net for what a landed flip does to OTHER tasks: this check only
/// ever names the task being rewritten.
fn flip_one(
    batch: &Batch<'_>,
    world: &mut Vec<mindtape::Task>,
    abs: &Path,
    note: Option<&str>,
) -> Result<()> {
    let project = batch.project;
    let source = std::fs::read_to_string(abs).map_err(|err| Diag::cannot_read(abs, &err))?;
    let updated = status_edit(&source, batch.status, batch.ymd, note).map_err(|err| {
        Diag::failed("can't rewrite the status")
            .value(abs.display().to_string())
            .cause(&err)
    })?;
    let landed = crate::pre_write::verify_one(project, world, abs, &updated)?;
    util::write_task(&project.config, abs, updated)?;
    if let Some(task) = landed {
        crate::pre_write::absorb(world, task);
    }
    Ok(())
}

/// Rewrite the `status:` argument of the first `task.with(…)` in `source` to
/// `status` dated `ymd` with an optional note. Pure: no IO. Shared with the
/// gitless `retire` rm strategy.
pub(crate) fn status_edit(
    source: &str,
    status: &str,
    ymd: (i32, u8, u8),
    note: Option<&str>,
) -> Result<String> {
    let call = task_with(source)?;
    let value = status_value(status, ymd, note);
    let edit = upsert_named_arg(source, &call, "status", &value)?;
    Ok(apply(source, vec![edit])?)
}

/// Print a one-line transition summary for one task.
fn report_flip(
    spec: &str,
    title: &str,
    from: Option<&mindtape::Status>,
    status: &str,
    restamp: bool,
) {
    if restamp {
        println!("re-stamped {spec} ({title}): {status} (today)");
    } else {
        let from = from.map_or("(unset)", |s| s.name.as_str());
        println!("flipped {spec} ({title}): {from} → {status}");
    }
}

/// After a successful flip to a terminal status, notice a recurring task
/// rather than staying silent: `mt done` stays a pure close — it never
/// strips `recur` or advances anything (see
/// `truth/tasks/command-semantics-cqrs.typ`) — but a `spawn` series still has
/// a next occurrence to materialize, so point at `mt recur`; a `mutate`
/// series has no further file to advance, so note that this closes it.
fn notice_recur(
    task: &mindtape::Task,
    status: &str,
    legend: &Legend,
    recur_config: &mindtape::recur::RecurConfig,
) {
    let Some(recur) = &task.recur else { return };
    if !legend.terminal().iter().any(|t| t == status) {
        return;
    }
    match mindtape::recur::dates::effective_materialize(recur, recur_config) {
        "mutate" => println!(
            "{:?} is recurring (mutate) — closing this ends the series",
            task.id
        ),
        _ => println!(
            "{:?} is recurring — run `mt recur {}` to advance",
            task.id, task.id
        ),
    }
}

/// Whether the post-write hooks should run: a non-quiet run attached to a
/// terminal. Split out so both operands are stageable without a pty.
fn hooks_enabled(quiet: bool, stdout_tty: bool) -> bool {
    !quiet && stdout_tty
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_output;

#[cfg(test)]
mod tests_revalidate;

#[cfg(test)]
mod tests_unblocked;

#[cfg(test)]
mod tests_children;

#[cfg(test)]
mod tests_verbs;

#[cfg(test)]
mod tests_note_required;
