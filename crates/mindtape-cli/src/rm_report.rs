//! The delete-style plan's report and confirm helpers (deletions, and the
//! git-tombstone/unlink rewrites) — split out of `rm.rs` to stay under the
//! file-size gate, mirroring `rm_gitless_report.rs` for the retire plan.

use std::path::Path;

use anyhow::Result;
use typst_cleanup::Plan;

pub(super) fn confirm(plan: &Plan) -> Result<bool> {
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
    super::rm_gitless::confirm_answer()
}

/// List each deletion in `plan` as an indented bullet on stdout.
fn print_deletions(plan: &Plan) {
    for deletion in &plan.deletions {
        println!("  - {}", deletion.rel.display());
    }
}

/// Total inbound links rewritten across `plan`'s rewrites.
pub(super) fn rewrite_link_count(plan: &Plan) -> usize {
    plan.rewrites.iter().map(|r| r.links).sum()
}

/// Print what the plan would do (used by `--dry-run`).
pub(super) fn report_plan(plan: &Plan) {
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

/// Print what the plan just did, and remind the user to review and commit —
/// the hint only when `root` sits inside a git repository; there is nothing
/// to commit and no tool to name otherwise.
pub(super) fn report_applied(plan: &Plan, root: &Path) {
    println!("deleted {} task(s):", plan.deletions.len());
    print_deletions(plan);
    if !plan.rewrites.is_empty() {
        println!(
            "rewrote {} link(s) in {} file(s)",
            rewrite_link_count(plan),
            plan.rewrites.len()
        );
    }
    super::rm_gitless::print_commit_hint(root);
}
