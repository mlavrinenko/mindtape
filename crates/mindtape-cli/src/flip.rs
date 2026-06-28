//! `mt flip <status> <task>...` — stamp a new status onto tasks.
//!
//! Mutates each task's `status:` to `<status>` dated today, the same shape
//! `add` stamps the initial status (shortcut `name(y, m, d)` for a prelude name,
//! generic `status("name", y, m, d)` otherwise — both via
//! [`mindtape::status_value`]). The status must be a configured legend value.
//!
//! Every legend value also doubles as a top-level shortcut verb dispatched here:
//! `mt done 1b 2c` == `mt flip done 1b 2c` (see [`run_external`]).
//! Built-in subcommands win on a name clash; a shadowing legend value is warned
//! about. `mt flip --list` prints the available shortcut verbs.
//!
//! Batch semantics mirror `rm`: every target is resolved before any file is
//! written (a status change reorders the default view, so handles are resolved
//! up front), then the valid edits are applied and any resolution or post-edit
//! validation problem is reported without rolling the rest back.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser};
use mindtape::config::Legend;
use mindtape::{Config, load_config, load_project, status_value, today_ymd, validate};
use typst_edit::apply;

use crate::task_edit::{task_with, upsert_named_arg};
use crate::util::{self, Project};

/// Arguments for `mt flip`.
#[derive(Args)]
pub struct FlipArgs {
    /// Status to stamp: a configured legend value. Omit only with `--list`.
    #[arg(required_unless_present = "list")]
    pub status: Option<String>,

    /// Tasks to flip: handle (`1b`), slug, relative path, or absolute path.
    pub tasks: Vec<String>,

    /// Status note (the trailing `[note]` block). Without it any existing note is
    /// cleared, since the note describes the status being left.
    #[arg(short = 'm', long = "note", value_name = "NOTE")]
    pub note: Option<String>,

    /// List the per-status shortcut verbs (the configured legend) and exit.
    #[arg(long)]
    pub list: bool,
}

/// Parser wrapper for the shortcut-verb path: the raw external argv
/// (`<status> <task>...`, plus any `-m`) re-parsed into [`FlipArgs`].
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
        bail!(
            "unknown command {verb:?}; run `mt --help` for commands or \
             `mt flip --list` for status shortcut verbs"
        );
    }
    let parsed = Shortcut::try_parse_from(external).map_err(|err| anyhow::anyhow!("{err}"))?;
    run_with(&project, &parsed.args)
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
            "unknown status {status:?}; configured legend: {}",
            verbs_csv(legend)
        );
    }

    // Resolve every target before mutating: a status change reorders the default
    // view, so handles must be pinned up front to avoid mid-batch drift.
    let mut resolved = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    for spec in &args.tasks {
        match util::resolve_task(project, spec) {
            Ok((task, abs)) => resolved.push((spec.clone(), task, abs)),
            Err(err) => problems.push(format!("{spec}: {err:#}")),
        }
    }

    let ymd = today_ymd();
    let mut touched_ids = HashSet::new();
    let mut touched_files = HashSet::new();
    for (spec, task, abs) in &resolved {
        match flip_one(task, abs, status, ymd, args.note.as_deref()) {
            Ok(restamp) => {
                touched_ids.insert(task.id.clone());
                touched_files.insert(task.file.clone());
                report_flip(spec, &task.title, task.status.as_ref(), status, restamp);
            }
            Err(err) => problems.push(format!("{spec}: {err:#}")),
        }
    }

    if !touched_ids.is_empty() {
        revalidate(&project.root, &touched_ids, &touched_files, &mut problems)?;
    }

    if problems.is_empty() {
        return Ok(());
    }
    for problem in &problems {
        eprintln!("warning: {problem}");
    }
    bail!("{} task(s) had problems", problems.len())
}

/// Rewrite one task's `status:` to `status` dated `ymd`. Returns whether this was
/// a re-stamp (the task already carried `status`).
fn flip_one(
    task: &mindtape::Task,
    abs: &Path,
    status: &str,
    ymd: (i32, u8, u8),
    note: Option<&str>,
) -> Result<bool> {
    let source = std::fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    let updated = status_edit(&source, status, ymd, note)
        .with_context(|| format!("rewrite status in {}", abs.display()))?;
    std::fs::write(abs, updated).with_context(|| format!("write {}", abs.display()))?;
    Ok(task.status.as_ref().is_some_and(|s| s.name == status))
}

/// Rewrite the `status:` argument of the first `task.with(…)` in `source` to
/// `status` dated `ymd` with an optional note. Pure: no IO.
fn status_edit(
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

/// Reload the project and append any load error or validation violation that
/// touches a flipped file to `problems` (tolerant batch: writes are kept).
fn revalidate(
    root: &Path,
    touched_ids: &HashSet<String>,
    touched_files: &HashSet<PathBuf>,
    problems: &mut Vec<String>,
) -> Result<()> {
    let config = load_config(root)?;
    let outcome = load_project(root, &config);
    for error in &outcome.errors {
        if touched_files.contains(&error.file) {
            problems.push(format!(
                "{}: failed to load after flip: {}",
                error.file.display(),
                error.message
            ));
        }
    }
    for violation in validate(&outcome.tasks, &config) {
        if touched_ids.contains(&violation.task_id) {
            problems.push(format!(
                "{}: {}: {}",
                violation.task_id, violation.field, violation.message
            ));
        }
    }
    Ok(())
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
fn verbs_csv(legend: &Legend) -> String {
    legend
        .entries
        .iter()
        .map(|entry| entry.value.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "flip_tests.rs"]
mod tests;
