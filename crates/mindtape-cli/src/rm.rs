//! `mt rm` — retire tasks and rewrite inbound links into SWHID tombstones,
//! atomically. A thin consumer of the consumer-blind [`typst_cleanup`] crate,
//! which owns the shared skeleton (dirty-tree guard, tombstone minting, atomic
//! link rewriting). This module supplies the project [`Policy`] ([`TaskLinks`]):
//! which references a survivor holds ([`TaskLinks::scan`] via
//! [`mindtape::link_target_spans`] — the `links:` helper edges, not just
//! `link()`), file-stem link identity, the tombstone label, and the task-worded
//! presentation.
//!
//! Selection: the optional `target` is either a task reference (slug, path, or a
//! `list` handle like `1v`) or a filter expression. A reference resolves a single
//! task regardless of its status; a filter retires every match. With no target,
//! the configured `[rm] default_filter` applies (built-in `terminal`: sweep every
//! terminal-status task).
//!
//! Atomic by construction: [`typst_cleanup::plan`] computes every mint and
//! rewrite in memory; not a byte is written until [`apply`] runs on a complete
//! plan. `rm` does not commit — the user reviews the diff and commits.
//!
//! Git is gated on need, not presence: [`run`] scans the survivors for inbound
//! references before touching the repository, and a task nothing references is
//! deleted directly ([`plain_delete_plan`]) — no toplevel discovery, no
//! tombstone, no committed-clean guard.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{
    Filter, Readiness, Task, is_history_pointer, link_target_spans, normalize_link, select,
};
use typst_cleanup::{Artifact, Deletion, Policy, Swhid, Target};
// Re-exported so `rm`'s presentation and tests refer to one plan type.
pub use typst_cleanup::{Plan, apply};

use crate::{git, util};

/// Arguments for `mt rm`.
#[derive(Args)]
pub struct RmArgs {
    /// Task to retire: a reference (slug, relative/absolute path, or a `list`
    /// handle like `1v`) or a filter expression. References resolve a single
    /// task regardless of status; filters retire every match. Omitted: the
    /// configured `[rm] default_filter` (built-in `terminal`).
    pub target: Option<String>,
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
/// guard, no mint. When a survivor does reference a removed task, a real run
/// (not `--dry-run`) refuses early if any selected task file is not
/// committed-clean; all offenders are listed at once so the user can fix them
/// in one step.
///
/// # Errors
/// Returns an error (non-zero exit) on an unresolvable target, an invalid
/// filter, an unreadable surviving file, a declined prompt, or — when a
/// survivor references a removed task — an uncommitted task file, a task file
/// that is not recoverable from git, a wrong-shaped surviving file, or any git
/// failure.
pub fn run(args: &RmArgs) -> Result<()> {
    let project = util::open()?;
    util::warn_errors(&project.outcome);

    // Readiness over the full set, before any narrowing, so `ready`/`blocked`/
    // `terminal` predicates and handle ordinals resolve the same as `list`.
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let eligible = select_targets(&project, args.target.as_deref(), &readiness)?;

    if eligible.is_empty() {
        println!("nothing to remove");
        return Ok(());
    }

    let root = project
        .root
        .canonicalize()
        .with_context(|| format!("resolve {}", project.root.display()))?;

    // A task nothing references needs no tombstone, so no git at all. Only when
    // a survivor points at an eligible task is the repository consulted.
    if inbound_references(&project.outcome.tasks, &eligible, &root)?.is_empty() {
        return finalize(
            &plain_delete_plan(&project.outcome.tasks, &eligible, &root),
            args,
        );
    }

    let git_root = git::toplevel(&root)?
        .canonicalize()
        .context("resolve git toplevel")?;

    if !args.dry_run {
        check_eligible_files(&project.outcome.tasks, &eligible, &root, &git_root)?;
    }

    let plan = plan(&project.outcome.tasks, &eligible, &root, &git_root)?;
    finalize(&plan, args)
}

/// The tail shared by both removal paths: dry-run reporting, confirmation, and
/// the atomic apply + summary. Writes nothing on `--dry-run`.
///
/// # Errors
/// A declined prompt aborts with `aborted by user`; [`apply`] propagates its
/// write failures.
fn finalize(plan: &Plan, args: &RmArgs) -> Result<()> {
    if args.dry_run {
        report_plan(plan);
        println!("dry run: nothing written");
        return Ok(());
    }

    if !args.yes && !confirm(plan)? {
        bail!("aborted by user");
    }

    apply(plan)?;
    report_applied(plan);
    Ok(())
}

/// Resolve the optional `target` into the set of task ids to retire.
///
/// A `Some` target resolves reference-first (path, handle, slug — slug wins on a
/// bare token), falling back to a filter parse + [`select`] when it is not a
/// reference. A `None` target applies the configured `[rm] default_filter`.
///
/// # Errors
/// Returns an error if the reference does not resolve and the input is not a
/// valid filter, or if the filter references an unknown field.
fn select_targets(
    project: &util::Project,
    target: Option<&str>,
    readiness: &Readiness,
) -> Result<HashSet<String>> {
    let tasks = match target {
        Some(spec) => match util::resolve_task(project, spec) {
            Ok((task, _)) => vec![task],
            // Not a reference: try a filter. Surface the reference error when the
            // input does not even parse as one (a mistyped ref, not a filter).
            Err(ref_err) => {
                let filter = Filter::parse(spec).map_err(|_| ref_err)?;
                filter_tasks(project, &filter, readiness)?
            }
        },
        None => match project.config.rm.default_filter.as_deref() {
            Some(expr) => {
                let filter = Filter::parse(expr)
                    .with_context(|| format!("parse [rm] default_filter {expr:?}"))?;
                filter_tasks(project, &filter, readiness)?
            }
            None => Vec::new(),
        },
    };
    Ok(tasks.into_iter().map(|task| task.id).collect())
}

/// Select every task matching `filter` over the full loaded set.
fn filter_tasks(
    project: &util::Project,
    filter: &Filter,
    readiness: &Readiness,
) -> Result<Vec<Task>> {
    Ok(select(
        project.outcome.tasks.clone(),
        Some(filter),
        &project.config,
        readiness,
    )?)
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

/// Which eligible ids a *surviving* task's source references — the same
/// parse-plane scan the tombstone path uses ([`TaskLinks::scan`] +
/// [`TaskLinks::identity`]), so the `links:` helper edges are covered, not just
/// `link()`. No eval, no git. Empty means no tombstone is needed: plain
/// deletion.
///
/// A link *from* an eligible task counts for nothing — it is going away in the
/// same run, so it is not waiting for attention.
///
/// # Errors
/// A survivor file that cannot be read fails the run, matching the tombstone
/// path's behaviour in [`typst_cleanup::plan`].
fn inbound_references(
    tasks: &[Task],
    eligible: &HashSet<String>,
    root: &Path,
) -> Result<HashSet<String>> {
    let policy = TaskLinks;
    let mut referenced = HashSet::new();
    for task in tasks {
        if eligible.contains(&task.id) {
            continue;
        }
        let source = std::fs::read_to_string(root.join(&task.file))
            .with_context(|| format!("read {}", task.file.display()))?;
        for target in policy.scan(&source) {
            let Some(id) = policy.identity(&target.raw) else {
                continue;
            };
            if eligible.contains(&id) {
                referenced.insert(id);
            }
        }
    }
    Ok(referenced)
}

/// The deletion-only plan for tasks nothing references: no tombstones, no
/// rewrites, no git. Deletions sorted like [`typst_cleanup::plan`]'s, for a
/// stable report.
fn plain_delete_plan(tasks: &[Task], eligible: &HashSet<String>, root: &Path) -> Plan {
    let mut deletions: Vec<Deletion> = tasks
        .iter()
        .filter(|task| eligible.contains(&task.id))
        .map(|task| Deletion {
            abs: root.join(&task.file),
            rel: task.file.clone(),
        })
        .collect();
    deletions.sort_by(|a, b| a.rel.cmp(&b.rel));
    Plan {
        deletions,
        rewrites: Vec::new(),
    }
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

fn confirm(plan: &Plan) -> Result<bool> {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    let rewrites = if plan.rewrites.is_empty() {
        String::new()
    } else {
        format!(" and rewrite links in {} file(s)", plan.rewrites.len())
    };
    writeln!(
        stderr,
        "rm will delete {} task(s){rewrites}:",
        plan.deletions.len()
    )?;
    for deletion in &plan.deletions {
        writeln!(stderr, "  - {}", deletion.rel.display())?;
    }
    write!(stderr, "Continue? [y/N] ")?;
    stderr.flush()?;
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;
    let answer = line.trim().to_ascii_lowercase();
    Ok(answer == "y" || answer == "yes")
}

/// List each deletion in `plan` as an indented bullet on stdout.
fn print_deletions(plan: &Plan) {
    for deletion in &plan.deletions {
        println!("  - {}", deletion.rel.display());
    }
}

/// Total inbound links rewritten across `plan`'s rewrites.
fn rewrite_link_count(plan: &Plan) -> usize {
    plan.rewrites.iter().map(|r| r.links).sum()
}

/// Print what the plan would do (used by `--dry-run`).
fn report_plan(plan: &Plan) {
    println!("would delete {} task(s):", plan.deletions.len());
    print_deletions(plan);
    if !plan.rewrites.is_empty() {
        println!(
            "would rewrite {} link(s) in {} file(s):",
            rewrite_link_count(plan),
            plan.rewrites.len()
        );
        for rewrite in &plan.rewrites {
            println!("  - {}", rewrite.rel.display());
        }
    }
}

/// Print what the plan just did, and remind the user to review and commit.
fn report_applied(plan: &Plan) {
    println!("deleted {} task(s):", plan.deletions.len());
    print_deletions(plan);
    if !plan.rewrites.is_empty() {
        println!(
            "rewrote {} link(s) in {} file(s)",
            rewrite_link_count(plan),
            plan.rewrites.len()
        );
    }
    println!("review the changes and commit when satisfied");
}

#[cfg(test)]
#[path = "rm_tests.rs"]
mod tests;
