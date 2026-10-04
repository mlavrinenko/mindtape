//! `mt flip <status> <task>...` — stamp a new status onto tasks.
//!
//! Mutates each task's `status:` to `<status>` dated today, the same shape
//! `add` stamps the initial status (shortcut `name(y, m, d)` for a prelude name,
//! generic `status("name", y, m, d)` otherwise — both via
//! [`mindtape::status_value`]). The status must be a configured legend value.
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
//! ([`crate::flip_batch`]).
//!
//! `flip` is also the one command that runs the project's `[[on.flip]]` actions
//! ([`crate::flip_hooks`]). They run per target, after the transition and
//! note-requirement guards and before the write, so a refusing gate leaves that
//! task's file untouched.

use std::collections::HashSet;
use std::io::IsTerminal;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, CommandFactory, FromArgMatches, Parser};
use mindtape::config::Legend;
use mindtape::{Config, status_value, today_ymd};
use typst_edit::apply;

use crate::flip_batch::{Landed, after_batch};
use crate::flip_hooks::{Hooks, Transition, note_with_overrides};
use crate::task_edit::{task_with, upsert_named_arg};
use crate::util::{self, Project};

/// Arguments for `mt flip`.
#[derive(Args)]
pub struct FlipArgs {
    /// Status to stamp: a configured legend value. Omit only with `--list`.
    #[arg(required_unless_present = "list")]
    pub status: Option<String>,

    #[arg(help = crate::task_ref::HELP)]
    pub tasks: Vec<String>,

    /// Status note (the trailing `[note]` block). Without it any existing note is
    /// cleared, since the note describes the status being left.
    #[arg(short = 'm', long = "note", value_name = "NOTE")]
    pub note: Option<String>,

    /// List the per-status shortcut verbs (the configured legend) and exit.
    #[arg(long)]
    pub list: bool,

    /// Bypass the declared status-transition graph, and waive a failing
    /// `[[on.flip]]` gate (corrections escape hatch). A waived gate is recorded
    /// in the status note.
    #[arg(long)]
    pub force: bool,

    /// Suppress `[[on.flip]]` `show` / `list` output, so a scripted flip stays
    /// clean. Gates still run; a failing one still prints its output.
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
    run_with(&util::open()?, args)
}

/// Dispatch a per-status shortcut verb: `external` is the raw `[verb, args…]`
/// captured by the `external_subcommand` fallback, where `verb` is the status.
///
/// # Errors
/// Returns an error if `verb` is not a configured legend value (an unknown
/// command), the args do not parse, or the underlying flip fails.
pub fn run_external(external: &[String]) -> Result<()> {
    let project = util::open()?;
    let verb = external
        .first()
        .context("empty shortcut invocation")?
        .as_str();
    let legend = &project.config.tasks.status;
    if !legend.contains(verb) {
        bail!(unknown_command_message(verb, legend));
    }
    let matches = match shortcut_command(verb).try_get_matches_from(external) {
        Ok(matches) => matches,
        Err(err) if !err.use_stderr() => {
            err.print().context("print the shortcut verb's help")?;
            return Ok(());
        }
        Err(err) => bail!("{err}"),
    };
    let parsed = Shortcut::from_arg_matches(&matches)?;
    run_with(&project, &parsed.args)
}

/// The parser for `mt <verb> …`, presented as that verb: named `mt <verb>`,
/// described in its own words, with the status positional (the verb itself,
/// already given) left out of the usage.
fn shortcut_command(verb: &str) -> clap::Command {
    Shortcut::command()
        .name(verb.to_owned())
        .bin_name(format!("mt {verb}"))
        .about(format!(
            "Stamp status `{verb}` onto tasks, dated today; same as `mt flip {verb} <task>...`"
        ))
        .mut_arg("status", |arg| arg.hide(true).index(1))
        .mut_arg("tasks", |arg| arg.index(2))
}

/// Configured legend values as `&str`, for a nearest-match suggestion shared
/// by the unknown-status and shortcut-verb-miss sites — both reject against
/// this same set
/// (`truth/tasks/adopt-nearest-match-suggestions-at-the-remaining-unknown-name-errors.typ`).
fn legend_values(legend: &Legend) -> Vec<&str> {
    legend.entries.iter().map(|e| e.value.as_str()).collect()
}

/// The "unknown command" message for a shortcut-verb miss (`mt <typo>
/// <task>`), naming a near legend value when one clears the similarity floor.
/// Only the message changes here — `verb` still dispatches solely on
/// `legend.contains`, so which inputs `mt <status> <task>` accepts is
/// untouched.
fn unknown_command_message(verb: &str, legend: &Legend) -> String {
    format!(
        "unknown command {verb:?}{}; run `mt --help` for commands or \
         `mt flip --list` for status shortcut verbs",
        mindtape::suggest::suggestion(verb, &legend_values(legend))
    )
}

/// Flip `args.status` onto every resolved task in `project`.
pub(crate) fn run_with(project: &Project, args: &FlipArgs) -> Result<()> {
    util::warn_errors(&project.outcome);
    warn_shadowed_verbs(&project.config);

    if args.list {
        list_verbs(&project.config);
        return Ok(());
    }

    let status = args.status.as_deref().context("status is required")?;
    if args.tasks.is_empty() {
        bail!("no tasks given; usage: mt flip <status> <task>...");
    }
    let legend = &project.config.tasks.status;
    if !legend.is_empty() && !legend.contains(status) {
        bail!(
            "unknown status {status:?}; configured legend: {}{}",
            verbs_csv(legend),
            mindtape::suggest::suggestion(status, &legend_values(legend))
        );
    }

    // Resolve every target before mutating, so each reference reads the project
    // the caller listed.
    let mut resolved = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    for spec in &args.tasks {
        match util::resolve_task(project, spec) {
            Ok((task, abs)) => resolved.push((spec.clone(), task, abs)),
            Err(err) => problems.push(format!("{spec}: {err:#}")),
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
    for (spec, task, abs) in &resolved {
        match flip_target(&batch, &mut world, spec, task, abs) {
            Ok(()) => landed.record(task),
            Err(problem) => problems.push(format!("{spec}: {problem}")),
        }
    }

    if !landed.is_empty() {
        after_batch(&project.root, &landed, &mut problems)?;
    }

    if problems.is_empty() {
        return Ok(());
    }
    for problem in &problems {
        eprintln!("error: {problem}");
    }
    bail!("{} task(s) had problems", problems.len())
}

/// The pre-write guards; split out to keep this file under its line budget.
/// See [`check_transition`].
#[path = "flip_guards.rs"]
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
/// `Err(message)` says why it did not — the batch reports it and moves on, so a
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
) -> std::result::Result<(), String> {
    let (args, status) = (batch.args, batch.status);
    let legend = &batch.project.config.tasks.status;
    if !args.force {
        check_transition(legend, task.status.as_ref(), status)?;
        let blockers = mindtape::blockers_of(world, task, &batch.project.config);
        check_unblocked(legend, status, &blockers)?;
        check_children(world, task, status, &batch.project.config)?;
    }
    check_note_required(legend, status, args.note.as_deref())?;
    let outcome = batch
        .hooks
        .run(
            &Transition {
                task,
                path: abs,
                target: status,
                force: args.force,
            },
            &mut std::io::stdout(),
        )
        .map_err(|err| format!("{err:#}"))?;
    if let Some(refusal) = outcome.refusal {
        return Err(refusal);
    }
    let note = note_with_overrides(args.note.as_deref(), &outcome.forced);
    let restamp = task.status.as_ref().is_some_and(|s| s.name == status);
    flip_one(batch, world, abs, note.as_deref()).map_err(|err| format!("{err:#}"))?;
    report_flip(spec, &task.title, task.status.as_ref(), status, restamp);
    notice_recur(task, status, legend, &batch.project.config.tasks.recur);
    Ok(())
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
    let source = std::fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    let updated = status_edit(&source, batch.status, batch.ymd, note)
        .with_context(|| format!("rewrite status in {}", abs.display()))?;
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
    apply(source, vec![edit]).map_err(|err| anyhow!(err.to_string()))
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
    match mindtape::recur_dates::effective_materialize(recur, recur_config) {
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

/// Warn when a legend value collides with a built-in subcommand name: the
/// built-in wins, so that value never reaches `flip` as a shortcut verb.
fn warn_shadowed_verbs(config: &Config) {
    let builtins: HashSet<String> = crate::cli::builtin_subcommand_names().into_iter().collect();
    for entry in &config.tasks.status.entries {
        if builtins.contains(&entry.value) {
            eprintln!(
                "warning: status {value:?} shadows the `mt {value}` subcommand; its \
                 shortcut verb won't dispatch — use `mt flip {value} …`",
                value = entry.value
            );
        }
    }
}

/// Print the per-status shortcut verbs (the configured legend) to stdout.
fn list_verbs(config: &Config) {
    let legend = &config.tasks.status;
    if legend.is_empty() {
        println!("no status legend configured; per-status shortcut verbs are unavailable");
        println!("(add [[tasks.status]] entries to enable `mt <status> <task>...`)");
        return;
    }
    println!(
        "per-status shortcut verbs (`mt <status> <task>...` == `mt flip <status> <task>...`):"
    );
    for entry in &legend.entries {
        match &entry.glyph {
            Some(glyph) => println!("  {glyph} {}", entry.value),
            None => println!("  {}", entry.value),
        }
    }
}

/// Comma-separated legend values, for an unknown-status error message.
/// Whether the post-write hooks should run: a non-quiet run attached to a
/// terminal. Split out so both operands are stageable without a pty.
fn hooks_enabled(quiet: bool, stdout_tty: bool) -> bool {
    !quiet && stdout_tty
}

fn verbs_csv(legend: &Legend) -> String {
    legend_values(legend).join(", ")
}

#[cfg(test)]
#[path = "flip_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "flip_tests_output.rs"]
mod tests_output;

#[cfg(test)]
#[path = "flip_tests_revalidate.rs"]
mod tests_revalidate;

#[cfg(test)]
#[path = "flip_tests_unblocked.rs"]
mod tests_unblocked;

#[cfg(test)]
#[path = "flip_tests_children.rs"]
mod tests_children;
