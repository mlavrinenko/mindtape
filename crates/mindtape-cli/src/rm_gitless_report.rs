//! The retire strategy's report and confirm helpers — split out of
//! `rm_gitless.rs` to stay under the file-size gate.

use std::path::Path;

use anyhow::{Context, Result};

use super::{RetirePlan, dest_rel};

/// Print what the retire plan would do (used by `--dry-run`).
pub fn report_retire_plan(plan: &RetirePlan) {
    println!(
        "would retire {} task(s) (status → {}):",
        plan.rewrites.len(),
        plan.status
    );
    for rewrite in &plan.rewrites {
        println!("  - {}", rewrite.rel.display());
    }
    if let Some(dir) = &plan.rename {
        println!(
            "would move {} task(s) into {}/:",
            plan.deletions.len(),
            dir.display()
        );
        for deletion in &plan.deletions {
            println!(
                "  - {} → {}",
                deletion.rel.display(),
                dest_rel(&plan.rename, &deletion.rel).display()
            );
        }
    }
    report_retarget(plan, "would rewrite");
}

/// Print what the retire plan just did, and remind the user to review — the
/// commit hint only when `root` sits inside a git repository.
pub fn report_retire_applied(plan: &RetirePlan, root: &Path) {
    println!(
        "retired {} task(s) (status → {}):",
        plan.rewrites.len(),
        plan.status
    );
    for rewrite in &plan.rewrites {
        println!("  - {}", rewrite.rel.display());
    }
    if let Some(dir) = &plan.rename {
        println!(
            "moved {} task(s) into {}/",
            plan.deletions.len(),
            dir.display()
        );
    }
    report_retarget(plan, "rewrote");
    super::print_commit_hint(root);
}

/// The retarget summary shared by the dry-run and applied reports: how many
/// inbound links of surviving tasks move to new paths, and where.
fn report_retarget(plan: &RetirePlan, verb: &str) {
    if plan.link_rewrites.is_empty() {
        return;
    }
    let links: usize = plan.link_rewrites.iter().map(|rewrite| rewrite.links).sum();
    println!(
        "{verb} {links} link(s) in {} file(s):",
        plan.link_rewrites.len()
    );
    for rewrite in &plan.link_rewrites {
        println!("  - {}", rewrite.rel.display());
    }
}

/// The retire prompt: what will happen, ending in the shared confirm read.
pub fn confirm_retire(plan: &RetirePlan) -> Result<bool> {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    writeln!(
        stderr,
        "rm will retire {} task(s) to {}:",
        plan.rewrites.len(),
        plan.status
    )?;
    for rewrite in &plan.rewrites {
        writeln!(stderr, "  - {}", rewrite.rel.display())?;
    }
    if let Some(dir) = &plan.rename {
        writeln!(
            stderr,
            "and move {} task(s) into {}/",
            plan.deletions.len(),
            dir.display()
        )?;
    }
    if !plan.link_rewrites.is_empty() {
        let links: usize = plan.link_rewrites.iter().map(|rewrite| rewrite.links).sum();
        writeln!(
            stderr,
            "and retarget {links} link(s) in {} file(s)",
            plan.link_rewrites.len()
        )?;
    }
    confirm_answer()
}

/// Read the `Continue? [y/N]` answer from stdin. Shared by the delete and
/// retire prompts.
///
/// # Errors
/// A stdin read failure.
pub fn confirm_answer() -> Result<bool> {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    write!(stderr, "Continue? [y/N] ")?;
    stderr.flush()?;
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;
    let answer = line.trim().to_ascii_lowercase();
    Ok(answer == "y" || answer == "yes")
}
