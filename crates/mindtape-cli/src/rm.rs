//! `mindtape cleanup` — retire terminal-status tasks and rewrite inbound links
//! into SWHID tombstones, atomically. A thin consumer of the consumer-blind
//! [`typst_cleanup`] crate, which owns the shared skeleton (dirty-tree guard,
//! tombstone minting, atomic link rewriting). This module supplies the project
//! policy — terminal-status eligibility, file-stem link identity — and the
//! task-worded presentation.
//!
//! Atomic by construction: [`typst_cleanup::plan`] computes every mint and
//! rewrite in memory; not a byte is written until [`apply`] runs on a complete
//! plan. `cleanup` does not commit — the user reviews the diff and commits.

use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{Task, is_history_pointer, normalize_link};
use typst_cleanup::{Artifact, Swhid};
// Re-exported so `cleanup`'s presentation and tests refer to one plan type.
pub use typst_cleanup::{Plan, apply};

use crate::{git, util};

/// Arguments for `mindtape cleanup`.
#[derive(Args)]
pub struct CleanupArgs {
    /// Skip the interactive confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,
    /// Print the plan without deleting or rewriting anything.
    #[arg(long)]
    pub dry_run: bool,
}

/// The cleanup policy: a `link()` URL names a task by file stem, unless it is
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

/// Run the `cleanup` command.
///
/// Applies the plan in place (deletes terminal task files, rewrites inbound
/// links) but does not commit — the user reviews the diff and commits. Refuses a
/// dirty working tree on a real run so the cleanup is the only change to review;
/// `--dry-run` previews the plan and skips that guard.
///
/// # Errors
/// Returns an error (non-zero exit) on a dirty working tree, a task file that is
/// not recoverable from git, a wrong-shaped surviving file, a declined prompt, or
/// any git failure.
pub fn run(args: &CleanupArgs) -> Result<()> {
    let project = util::open()?;
    util::warn_errors(&project.outcome);

    let root = project
        .root
        .canonicalize()
        .with_context(|| format!("resolve {}", project.root.display()))?;
    let git_root = git::toplevel(&root)?
        .canonicalize()
        .context("resolve git toplevel")?;

    if !args.dry_run && !git::is_clean(&root)? {
        bail!(
            "working tree has uncommitted changes; cleanup rewrites files in place — commit or stash first"
        );
    }

    let plan = plan(
        &project.outcome.tasks,
        &project.config.tasks.status.terminal(),
        &root,
        &git_root,
    )?;

    if plan.is_empty() {
        println!("nothing to clean up: no terminal-status tasks");
        return Ok(());
    }

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

/// Compute the cleanup plan over `tasks`: partition terminal tasks (to retire)
/// from survivors (to scan) and delegate to [`typst_cleanup::plan`].
///
/// # Errors
/// Propagates [`typst_cleanup`] failures: a file outside the repository, an
/// unrecoverable tombstone mint, a survivor read, or a rejected splice.
fn plan(tasks: &[Task], terminal: &[String], root: &Path, git_root: &Path) -> Result<Plan> {
    let mut eligible = Vec::new();
    let mut survivors = Vec::new();
    for task in tasks {
        let artifact = Artifact {
            abs: root.join(&task.file),
            rel: task.file.clone(),
            id: task.id.clone(),
        };
        if task.is_terminal(terminal) {
            eligible.push(artifact);
        } else {
            survivors.push(artifact);
        }
    }
    Ok(typst_cleanup::plan(
        &eligible, &survivors, git_root, &TaskLinks,
    )?)
}

fn confirm(plan: &Plan) -> Result<bool> {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    writeln!(
        stderr,
        "cleanup will delete {} task(s) and rewrite links in {} file(s):",
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
#[path = "cleanup_tests.rs"]
mod tests;
