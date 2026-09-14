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
//! (slug, path, or a `list` handle like `1v`) or a filter expression. A reference
//! resolves a single task regardless of its status; a filter retires every
//! match; the union across every entry, deduplicated by id, is retired in one
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
//! deleted directly ([`plain_delete_plan`]) — no strategy-deciding toplevel
//! discovery, no tombstone, no committed-clean guard. When a survivor does
//! reference a removed task, the configured `[rm] strategy` decides what
//! happens to those edges: the git-pinned SWHID tombstone rewrite
//! (`git-tombstone`, the default in a repository) or one of the gitless
//! strategies — `reject`, `unlink`, `retire` — implemented in [`rm_gitless`].
//! Every path's applied report still does one cheap, reused discovery
//! ([`rm_gitless::print_commit_hint`]) to decide whether the commit hint is
//! worth printing at all — pointless outside a repository, since there is
//! nothing to commit and no tool to name.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{
    Readiness, RmStrategy, Task, is_history_pointer, link_target_spans, normalize_link,
};
use typst_cleanup::{Artifact, Policy, Swhid, Target};
// Re-exported so `rm`'s presentation and tests refer to one plan type.
pub use typst_cleanup::{Plan, apply};

#[path = "rm_gitless.rs"]
mod rm_gitless;
pub(crate) use rm_gitless::{
    RetirePlan, confirm_retire, plain_delete_plan, redact_git_error, reject_message,
    report_retire_applied, report_retire_plan, resolve_strategy, retire_apply, retire_plan,
    unlink_plan,
};

#[path = "rm_select.rs"]
mod select;
use select::select_targets;

#[path = "rm_report.rs"]
mod report;
use report::{confirm, report_applied, report_plan};

use crate::{git, util};

/// Arguments for `mt rm`.
#[derive(Args)]
pub struct RmArgs {
    /// Tasks to retire: each a reference (slug, relative/absolute path, or a
    /// `list` handle like `1v`) or a filter expression. A reference resolves a
    /// single task regardless of status; a filter retires every match; the
    /// union of every entry, deduplicated, is retired in one pass. Omitted
    /// entirely: the configured `[rm] default_filter` (built-in `terminal`).
    pub targets: Vec<String>,
    /// Skip the interactive confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,
    /// Print the plan without deleting or rewriting anything.
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
    let project = util::open()?;
    util::warn_errors(&project.outcome);

    // Readiness over the full set, before any narrowing, so `ready`/`blocked`/
    // `terminal` predicates and handle ordinals resolve the same as `list`.
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let eligible = select_targets(&project, &args.targets, &readiness)?;

    if eligible.is_empty() {
        println!("nothing to remove");
        return Ok(());
    }

    let root = project
        .root
        .canonicalize()
        .with_context(|| format!("resolve {}", project.root.display()))?;

    // A task nothing references needs no tombstone, so no git at all. Only when
    // a survivor points at an eligible task is the strategy consulted.
    let referrers = referrers(&project.outcome.tasks, &eligible, &root)?;
    if referrers.is_empty() {
        return finalize(
            &Removal::Delete(plain_delete_plan(&project.outcome.tasks, &eligible, &root)),
            args,
            None,
            &root,
        );
    }

    let (git_root, git_error) = match git::toplevel(&root) {
        Ok(dir) => (
            Some(dir.canonicalize().context("resolve git toplevel")?),
            None,
        ),
        Err(err) => (None, Some(err)),
    };
    match resolve_strategy(&project.config, git_root.is_some()) {
        RmStrategy::GitTombstone => {
            let Some(git_root) = git_root else {
                let detail = git_error.map_or_else(String::new, |err| {
                    format!(": {}", redact_git_error(&err, &root))
                });
                bail!(
                    "cannot rewrite inbound links without a git repository{detail} — set [rm] strategy to \"retire\" or \"unlink\" for gitless repositories"
                );
            };
            if !args.dry_run {
                check_eligible_files(&project.outcome.tasks, &eligible, &root, &git_root)?;
            }
            let plan = plan(&project.outcome.tasks, &eligible, &root, &git_root)?;
            finalize(&Removal::Delete(plan), args, Some("git-tombstone"), &root)
        }
        RmStrategy::Reject => bail!("{}", reject_message(&referrers)),
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

/// The tail shared by every removal path: strategy dry-run reporting,
/// confirmation, and the atomic apply + summary. Writes nothing on
/// `--dry-run`. `root` gates the applied report's commit hint — printed only
/// inside a git repository.
///
/// # Errors
/// A declined prompt aborts with `aborted by user`; the apply propagates its
/// write failures.
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
        let confirmed = match removal {
            Removal::Delete(plan) => confirm(plan)?,
            Removal::Retire(plan) => confirm_retire(plan)?,
        };
        if !confirmed {
            bail!("aborted by user");
        }
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

/// Compute the removal plan over `tasks`: partition the `eligible` ids (to
/// retire) from survivors (to scan) and delegate to [`typst_cleanup::plan`],
/// which mints a tombstone per retired task and retargets every survivor's
/// inbound references — located by [`TaskLinks::scan`], so the `links:` helper
/// edges are covered, not just `link()`. Nothing is written.
///
/// # Errors
/// Propagates [`typst_cleanup`] failures: a file outside the repository, an
/// unrecoverable tombstone mint (untracked, dirty — this is where a dirty
/// eligible file aborts a `--dry-run`), a survivor read, or a rejected splice.
fn plan(tasks: &[Task], eligible: &HashSet<String>, root: &Path, git_root: &Path) -> Result<Plan> {
    let mut retire = Vec::new();
    let mut survivors = Vec::new();
    for task in tasks {
        let artifact = Artifact {
            abs: root.join(&task.file),
            rel: task.file.clone(),
            id: task.id.clone(),
        };
        if eligible.contains(&task.id) {
            retire.push(artifact);
        } else {
            survivors.push(artifact);
        }
    }
    Ok(typst_cleanup::plan(
        &retire, &survivors, git_root, &TaskLinks,
    )?)
}

/// Which eligible ids each *surviving* task's source references — the same
/// parse-plane scan the tombstone path uses ([`TaskLinks::scan`] +
/// [`TaskLinks::identity`]), so the `links:` helper edges are covered, not
/// just `link()`. No eval, no git. Empty means no tombstone is needed: plain
/// deletion.
///
/// A link *from* an eligible task counts for nothing — it is going away in the
/// same run, so it is not waiting for attention.
///
/// # Errors
/// A survivor file that cannot be read fails the run, matching the tombstone
/// path's behaviour in [`typst_cleanup::plan`].
fn referrers(
    tasks: &[Task],
    eligible: &HashSet<String>,
    root: &Path,
) -> Result<BTreeMap<PathBuf, HashSet<String>>> {
    let policy = TaskLinks;
    let mut by_file: BTreeMap<PathBuf, HashSet<String>> = BTreeMap::new();
    for task in tasks {
        if eligible.contains(&task.id) {
            continue;
        }
        let source = std::fs::read_to_string(root.join(&task.file))
            .with_context(|| format!("read {}", task.file.display()))?;
        let mut hits = HashSet::new();
        for target in policy.scan(&source) {
            let Some(id) = policy.identity(&target.raw) else {
                continue;
            };
            if eligible.contains(&id) {
                hits.insert(id);
            }
        }
        if !hits.is_empty() {
            by_file.insert(task.file.clone(), hits);
        }
    }
    Ok(by_file)
}

/// The union of [`referrers`]'s referenced ids — enough to decide whether any
/// strategy applies at all. The run path works off the per-file map; tests
/// still exercise the union.
#[cfg(test)]
fn inbound_references(
    tasks: &[Task],
    eligible: &HashSet<String>,
    root: &Path,
) -> Result<HashSet<String>> {
    Ok(referrers(tasks, eligible, root)?
        .into_values()
        .flatten()
        .collect())
}

/// Check every eligible task file individually: each must be committed and
/// unmodified so `swhid_mint` can pin its tombstone to the last-touch commit.
/// Collects all failures before returning so the user sees every offender in
/// one shot rather than fixing and retrying one at a time.
///
/// # Errors
/// All offending tasks in a single message with per-task reason and action.
fn check_eligible_files(
    tasks: &[Task],
    eligible: &HashSet<String>,
    root: &Path,
    git_root: &Path,
) -> Result<()> {
    let mut problems: Vec<(String, PathBuf, String)> = Vec::new();
    for task in tasks {
        if !eligible.contains(&task.id) {
            continue;
        }
        let abs = root.join(&task.file);
        let git_rel = if let Ok(rel) = abs.strip_prefix(git_root) {
            rel.to_string_lossy().into_owned()
        } else {
            problems.push((
                task.id.clone(),
                task.file.clone(),
                "outside git repository".into(),
            ));
            continue;
        };
        let reason = match swhid_mint::git::mint_rev_path(git_root, &git_rel) {
            Ok(_) => continue,
            Err(swhid_mint::git::Error::NotTracked(_)) => "not tracked by git".into(),
            Err(swhid_mint::git::Error::Dirty(_)) => "has uncommitted changes".into(),
            Err(swhid_mint::git::Error::Git(msg)) => format!("git error: {msg}"),
        };
        problems.push((task.id.clone(), task.file.clone(), reason));
    }
    if problems.is_empty() {
        return Ok(());
    }
    if let Some((id, file, reason)) = (problems.len() == 1).then(|| problems.first()).flatten() {
        bail!(
            "cannot remove {id}: {} {reason}\ncommit the file first (tombstone pins the last-touch commit)",
            file.display()
        );
    }
    let lines: Vec<String> = problems
        .iter()
        .map(|(id, file, reason): &(String, PathBuf, String)| {
            format!("  {id}  {}  {reason}", file.display())
        })
        .collect();
    bail!(
        "cannot remove {} tasks — commit their files first (tombstones pin last-touch commits):\n{}",
        problems.len(),
        lines.join("\n")
    );
}

#[cfg(test)]
#[path = "rm_tests.rs"]
mod tests;
