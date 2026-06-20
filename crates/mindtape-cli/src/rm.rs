//! `mindtape rm` — retire tasks and rewrite inbound links into SWHID tombstones,
//! atomically. A thin consumer of the consumer-blind [`typst_cleanup`] crate,
//! which owns the shared skeleton (dirty-tree guard, tombstone minting, atomic
//! link rewriting). This module supplies the project policy — which tasks to
//! retire, file-stem link identity — and the task-worded presentation.
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

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{Filter, Readiness, Task, is_history_pointer, normalize_link, select};
use typst_cleanup::{Artifact, Swhid};
// Re-exported so `rm`'s presentation and tests refer to one plan type.
pub use typst_cleanup::{Plan, apply};

use crate::{git, util};

/// Arguments for `mindtape rm`.
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

/// The removal policy: a `link()` URL names a task by file stem, unless it is
/// already a SWHID history pointer. The tombstone is the bare quoted SWHID.
struct TaskLinks;

impl typst_cleanup::Policy for TaskLinks {
    fn link_identity(&self, url: &str) -> Option<String> {
        if is_history_pointer(url) {
            None
        } else {
            Some(normalize_link(url))
        }
    }

    fn tombstone(&self, _retired: &Artifact, swhid: &Swhid) -> String {
        format!("\"{}\"", swhid.render())
    }
}

/// Run the `rm` command.
///
/// Applies the plan in place (deletes the selected task files, rewrites inbound
/// links) but does not commit — the user reviews the diff and commits. Refuses a
/// dirty working tree on a real run so the removal is the only change to review;
/// `--dry-run` previews the plan and skips that guard.
///
/// # Errors
/// Returns an error (non-zero exit) on a dirty working tree, an unresolvable
/// target, an invalid filter, a task file that is not recoverable from git, a
/// wrong-shaped surviving file, a declined prompt, or any git failure.
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
    let git_root = git::toplevel(&root)?
        .canonicalize()
        .context("resolve git toplevel")?;

    if !args.dry_run && !git::is_clean(&root)? {
        bail!(
            "working tree has uncommitted changes; rm rewrites files in place — commit or stash first"
        );
    }

    let plan = plan(&project.outcome.tasks, &eligible, &root, &git_root)?;

    if args.dry_run {
        report_plan(&plan);
        println!("dry run: nothing written");
        return Ok(());
    }

    if !args.yes && !confirm(&plan)? {
        bail!("aborted by user");
    }

    apply(&plan)?;
    report_applied(&plan);
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
/// retire) from survivors (to scan) and delegate to [`typst_cleanup::plan`].
///
/// # Errors
/// Propagates [`typst_cleanup`] failures: a file outside the repository, an
/// unrecoverable tombstone mint, a survivor read, or a rejected splice.
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

fn confirm(plan: &Plan) -> Result<bool> {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    writeln!(
        stderr,
        "rm will delete {} task(s) and rewrite links in {} file(s):",
        plan.deletions.len(),
        plan.rewrites.len()
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

/// Print what the plan would do (used by `--dry-run`).
fn report_plan(plan: &Plan) {
    println!("would delete {} task(s):", plan.deletions.len());
    for deletion in &plan.deletions {
        println!("  - {}", deletion.rel.display());
    }
    if !plan.rewrites.is_empty() {
        let links: usize = plan.rewrites.iter().map(|r| r.links).sum();
        println!(
            "would rewrite {links} link(s) in {} file(s):",
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
    for deletion in &plan.deletions {
        println!("  - {}", deletion.rel.display());
    }
    if !plan.rewrites.is_empty() {
        let links: usize = plan.rewrites.iter().map(|r| r.links).sum();
        println!("rewrote {links} link(s) in {} file(s)", plan.rewrites.len());
    }
    println!("review the changes and commit when satisfied");
}

#[cfg(test)]
#[path = "rm_tests.rs"]
mod tests;
