//! The delete-style plan's report and confirm helpers (deletions, and the
//! git-tombstone/unlink rewrites) — split out of `cmd/rm/mod.rs` to stay under the
//! file-size gate, mirroring `cmd/rm/gitless/report.rs` for the retire plan.

use std::path::Path;

use mindtape::count;
use typst_cleanup::Plan;

/// The lines of the delete prompt: how many tasks go and how many files'
/// links change, then each deleted file.
pub(super) fn prompt_lines(plan: &Plan) -> Vec<String> {
    let rewrites = if plan.rewrites.is_empty() {
        String::new()
    } else {
        format!(
            " and rewrite links in {}",
            count(plan.rewrites.len(), "file")
        )
    };
    let mut lines = vec![format!(
        "rm will delete {}{rewrites}:",
        count(plan.deletions.len(), "task")
    )];
    lines.extend(
        plan.deletions
            .iter()
            .map(|deletion| format!("  - {}", deletion.rel.display())),
    );
    lines
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
    println!("would delete {}:", count(plan.deletions.len(), "task"));
    print_deletions(plan);
    if !plan.rewrites.is_empty() {
        println!(
            "would rewrite {} in {}:",
            count(rewrite_link_count(plan), "link"),
            count(plan.rewrites.len(), "file")
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
    println!("deleted {}:", count(plan.deletions.len(), "task"));
    print_deletions(plan);
    if !plan.rewrites.is_empty() {
        println!(
            "rewrote {} in {}",
            count(rewrite_link_count(plan), "link"),
            count(plan.rewrites.len(), "file")
        );
    }
    super::gitless::print_commit_hint(root);
}
