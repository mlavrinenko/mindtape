//! `mt rm` — retire tasks and rewrite inbound links into SWHID tombstones,
//! atomically. A thin consumer of the consumer-blind [`typst_cleanup`] crate,
//! which owns the shared skeleton (dirty-tree guard, tombstone minting, atomic
//! link rewriting). This module supplies the project [`Policy`] ([`TaskLinks`]):
//! which references a survivor holds ([`TaskLinks::scan`] via
//! [`mindtape::link_target_spans`] — the `links:` helper edges, not just
//! `link()`), file-stem link identity, the tombstone label, and the task-worded
//! presentation.
//!
//! Selection: `target` is variadic — zero or more entries, each a task reference
//! (stem, path, or stem hash), a folder under a task root or a filter
//! expression. A reference resolves a single task regardless of its status; a
//! folder retires every task inside it, as naming each would; a filter retires
//! every match; the union across every entry, deduplicated by id, is retired in one
//! pass — one confirmation prompt, one tombstone/rewrite pass. With no targets
//! at all, the configured `[rm] default_filter` applies (built-in `terminal`:
//! sweep every terminal-status task).
//!
//! Atomic by construction: [`typst_cleanup::plan`] computes every mint and
//! rewrite in memory; not a byte is written until [`apply`] runs on a complete
//! plan. `rm` does not commit — the user reviews the diff and commits.
//!
//! Git is gated on need, not presence: [`run`] scans the survivors for inbound
//! references before touching the repository, and a task nothing references is
//! deleted directly ([`typst_cleanup::delete_plan`]) — no strategy-deciding toplevel
//! discovery, no tombstone, no committed-clean guard. When a survivor does
//! reference a removed task, the configured `[rm] strategy` decides what
//! happens to those edges: the git-pinned SWHID tombstone rewrite
//! (`git-tombstone`, the default in a repository) or one of the gitless
//! strategies — `reject`, `unlink`, `retire` — implemented in [`gitless`].
//! Every path's applied report still does one cheap, reused discovery
//! ([`gitless::print_commit_hint`]) to decide whether the commit hint is
//! worth printing at all — pointless outside a repository, since there is
//! nothing to commit and no tool to name.
//!
//! # Files
//!
//! - `gitless/` — the strategies when no git repository pins a tombstone.
//! - `report.rs` — the delete-style plan's report and confirm helpers.
//! - `select.rs` — resolves the `target` argument into task ids.

use std::collections::HashSet;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::diag::Result;
use clap::Args;
use mindtape::{
    Readiness, RmStrategy, Task, is_history_pointer, link_target_spans, normalize_link,
};
use typst_cleanup::{Artifact, Policy, Swhid, Target};
// Re-exported so `rm`'s presentation and tests refer to one plan type.
pub use typst_cleanup::{Plan, apply};

mod gitless;
pub(crate) use gitless::{
    Retarget, RetirePlan, inbound_rewrites, print_commit_hint, redact_git_error, reject_diag,
    report_retire_applied, report_retire_plan, resolve_strategy, retarget_raw, retire_apply,
    retire_plan, retire_prompt_lines, unlink_plan,
};

mod select;
use select::select_targets;

mod report;
use report::{prompt_lines, report_applied, report_plan};

use crate::diag::{Diag, Hint, Note};
use crate::{git, util};

/// Arguments for `mt rm`.
#[derive(Args)]
pub struct RmArgs {
    /// The tasks to delete: tasks, folders, or a filter such as `done`.
    ///
    /// Each is a task (stem, hash or path), a folder under a task root (every
    /// task inside it, however deep) or a filter such as `done`. Without any,
    /// `mt rm` deletes what `[rm] default-filter` picks, which is every closed
    /// task unless you change it.
    pub targets: Vec<String>,
    /// Don't ask for confirmation.
    ///
    /// Without a terminal on stdin `mt rm` can't ask, so it refuses unless you
    /// pass this.

    #[arg(short = 'y', long)]
    pub yes: bool,
    /// Show what would be deleted and rewritten without changing anything.
    #[arg(long)]
    pub dry_run: bool,
}

/// The removal policy. A survivor's rewritable references are its `link()` calls
/// and `links:` edge strings ([`Self::scan`]); each names a task by file stem
/// ([`Self::identity`]), unless already a SWHID history pointer. The tombstone is
/// the bare quoted SWHID.
struct TaskLinks;

impl Policy for TaskLinks {
    fn scan(&self, source: &str) -> Vec<Target> {
        link_target_spans(source)
            .into_iter()
            .map(|span| Target {
                raw: span.raw,
                range: span.range,
            })
            .collect()
    }

    fn identity(&self, raw: &str) -> Option<String> {
        if is_history_pointer(raw) {
            None
        } else {
            Some(normalize_link(raw))
        }
    }

    fn tombstone(&self, _retired: &Artifact, swhid: &Swhid) -> String {
        format!("\"{}\"", swhid.render())
    }
}

/// Run the `rm` command.
///
/// Applies the plan in place (deletes the selected task files, rewrites inbound
/// links) but does not commit — the user reviews the diff and commits. A task
/// no survivor references is deleted directly, with no git at all: nothing
/// needs a tombstone, so there is no toplevel discovery, no committed-clean
/// guard, no mint. When a survivor does reference a removed task, the
/// configured `[rm] strategy` (default: `git-tombstone` in a repository,
/// `reject` without one) decides what happens to the inbound edges.
///
/// # Errors
/// Returns an error (non-zero exit) on an unresolvable target, an invalid
/// filter, an unreadable surviving file, a declined prompt, a `reject`
/// strategy with surviving referrers, or — under `git-tombstone` — an
/// uncommitted task file, a task file that is not recoverable from git, a
/// wrong-shaped surviving file, or any git failure.
pub fn run(args: &RmArgs) -> Result<()> {
    let project = util::open_warned()?;

    // Readiness over the full set, before any narrowing, so `ready`/`blocked`/
    // `terminal` predicates resolve the same as `list`.
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let eligible = select_targets(&project, &args.targets, &readiness)?;

    if eligible.is_empty() {
        println!("nothing to remove");
        return Ok(());
    }

    let root = canonical(&project.root)?;

    // A task nothing references needs no tombstone, so no git at all. Only when
    // a survivor points at an eligible task is the strategy consulted.
    let (retire, survivors) = artifacts(&project.outcome.tasks, &eligible, &root);
    let referrers = typst_cleanup::referrers(&retire, &survivors, &TaskLinks)?;
    if referrers.is_empty() {
        let plan = typst_cleanup::delete_plan(&retire);
        return finalize(&Removal::Delete(plan), args, None, &root);
    }

    let (git_root, git_error) = match git::toplevel(&root) {
        Ok(dir) => (Some(canonical(&dir)?), None),
        Err(err) => (None, Some(err)),
    };
    match resolve_strategy(&project.config, git_root.is_some()) {
        RmStrategy::GitTombstone => {
            let Some(git_root) = git_root else {
                return Err(no_repository(git_error.as_ref(), &root));
            };
            let plan = typst_cleanup::plan(&retire, &survivors, &git_root, &TaskLinks)?;
            finalize(&Removal::Delete(plan), args, Some("git-tombstone"), &root)
        }
        RmStrategy::Reject => Err(reject_diag(&referrers)),
        RmStrategy::Unlink => {
            let plan = unlink_plan(&project.outcome.tasks, &eligible, &root)?;
            finalize(&Removal::Delete(plan), args, Some("unlink"), &root)
        }
        RmStrategy::Retire => {
            let plan = retire_plan(&project.outcome.tasks, &eligible, &root, &project.config)?;
            finalize(&Removal::Retire(plan), args, Some("retire"), &root)
        }
    }
}

/// What a run will do, before it happens: a delete-style plan (deletions plus
/// optional rewrites — the git-tombstone and unlink shapes) or a retire plan
/// (status flips plus optional moves).
enum Removal {
    Delete(Plan),
    Retire(RetirePlan),
}

/// Refuse to ask when nobody can answer: with stdin not a terminal the prompt
/// would read nothing, and `--yes` is the way to confirm without one.
///
/// Takes the terminal answer rather than reading it, so a test reaches both
/// arms whatever the suite's stdin is.
fn ensure_can_ask(stdin_is_terminal: bool) -> Result<()> {
    if stdin_is_terminal {
        return Ok(());
    }
    Err(
        Diag::refused("can't ask for confirmation without a terminal")
            .hint(Hint::new("pass --yes to remove without asking")),
    )
}

/// Ask the person at the terminal whether to go on with `removal`.
///
/// # Errors
/// Refuses without a terminal to ask on; `aborted by user` when the answer is
/// no.
fn ask_to_proceed(removal: &Removal) -> Result<()> {
    ask_with(
        removal,
        std::io::stdin().is_terminal(),
        crate::diag::confirm,
    )
}

/// [`ask_to_proceed`] with the terminal answer and the question given, so a
/// test reaches the prompt whatever the suite's stdin is.
fn ask_with(
    removal: &Removal,
    stdin_is_terminal: bool,
    confirm: impl FnOnce(&[String]) -> Result<bool>,
) -> Result<()> {
    ensure_can_ask(stdin_is_terminal)?;
    let lines = match removal {
        Removal::Delete(plan) => prompt_lines(plan),
        Removal::Retire(plan) => retire_prompt_lines(plan),
    };
    ensure_confirmed(confirm(&lines)?)
}

/// `Ok` for a yes; a no is the user stopping the run.
fn ensure_confirmed(answered_yes: bool) -> Result<()> {
    if answered_yes {
        return Ok(());
    }
    Err(Diag::failed("aborted by user"))
}

/// The tail shared by every removal path: strategy dry-run reporting,
/// confirmation, and the atomic apply + summary. Writes nothing on
/// `--dry-run`. `root` gates the applied report's commit hint — printed only
/// inside a git repository.
///
/// # Errors
/// Without a terminal and without `--yes` it refuses; a declined prompt aborts
/// with `aborted by user`; the apply propagates its write failures.
fn finalize(removal: &Removal, args: &RmArgs, strategy: Option<&str>, root: &Path) -> Result<()> {
    if args.dry_run {
        if let Some(name) = strategy {
            println!("strategy: {name}");
        }
        match removal {
            Removal::Delete(plan) => report_plan(plan),
            Removal::Retire(plan) => report_retire_plan(plan),
        }
        println!("dry run: nothing written");
        return Ok(());
    }

    if !args.yes {
        ask_to_proceed(removal)?;
    }

    match removal {
        Removal::Delete(plan) => apply(plan)?,
        Removal::Retire(plan) => retire_apply(plan)?,
    }
    match removal {
        Removal::Delete(plan) => report_applied(plan, root),
        Removal::Retire(plan) => report_retire_applied(plan, root),
    }
    Ok(())
}

/// The `eligible` tasks (to retire) and the rest (the survivors to scan), as
/// typst-cleanup's [`Artifact`]s under `root`. Every removal path starts here:
/// [`typst_cleanup::referrers`], [`typst_cleanup::delete_plan`] and
/// [`typst_cleanup::plan`] all take this split.
fn artifacts(
    tasks: &[Task],
    eligible: &HashSet<String>,
    root: &Path,
) -> (Vec<Artifact>, Vec<Artifact>) {
    tasks
        .iter()
        .map(|task| Artifact {
            abs: root.join(&task.file),
            rel: task.file.clone(),
            id: task.id.clone(),
        })
        .partition(|artifact| eligible.contains(&artifact.id))
}

/// `path` made absolute and symlink-free.
fn canonical(path: &Path) -> Result<PathBuf, Diag> {
    mindtape::canonical(path).map_err(|err| {
        Diag::failed("can't resolve path")
            .value(path.display().to_string())
            .io(&err)
    })
}

/// The refusal for a link rewrite that needs a git repository when there is
/// none: what git said, with the tasks root redacted, then the way out.
fn no_repository(git_error: Option<&typst_cleanup::Error>, root: &Path) -> Diag {
    let diag = Diag::refused("cannot rewrite inbound links without a git repository").hint(
        Hint::new("set [rm] strategy to \"retire\" or \"unlink\" for gitless repositories"),
    );
    match git_error {
        Some(err) => diag.note(Note::new("git said").name(redact_git_error(err, root))),
        None => diag,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_select;
