//! The retire strategy's report and confirm helpers — split out of
//! `cmd/rm/gitless/mod.rs` to stay under the file-size gate.

use std::path::Path;

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

/// The lines of the retire prompt: what will happen, then the files, the
/// move and the link retargets when there are any.
pub fn retire_prompt_lines(plan: &RetirePlan) -> Vec<String> {
    let mut lines = vec![format!(
        "rm will retire {} task(s) to {}:",
        plan.rewrites.len(),
        plan.status
    )];
    lines.extend(
        plan.rewrites
            .iter()
            .map(|rewrite| format!("  - {}", rewrite.rel.display())),
    );
    if let Some(dir) = &plan.rename {
        lines.push(format!(
            "and move {} task(s) into {}/",
            plan.deletions.len(),
            dir.display()
        ));
    }
    if !plan.link_rewrites.is_empty() {
        let moved: usize = plan.link_rewrites.iter().map(|rewrite| rewrite.links).sum();
        lines.push(format!(
            "and retarget {moved} link(s) in {} file(s)",
            plan.link_rewrites.len()
        ));
    }
    lines
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;

    use typst_cleanup::{Deletion, Rewrite};

    use super::*;

    fn rewrite(rel: &str, links: usize) -> Rewrite {
        Rewrite {
            abs: PathBuf::from("/r").join(rel),
            rel: PathBuf::from(rel),
            contents: String::new(),
            links,
        }
    }

    fn plan(rename: Option<&str>, link_rewrites: Vec<Rewrite>) -> RetirePlan {
        RetirePlan {
            status: "done".to_owned(),
            rename: rename.map(PathBuf::from),
            rewrites: vec![rewrite("gone.typ", 0)],
            link_rewrites,
            deletions: vec![Deletion {
                abs: PathBuf::from("/r/gone.typ"),
                rel: PathBuf::from("gone.typ"),
            }],
        }
    }

    #[test]
    fn the_retire_prompt_lists_the_plan_the_move_and_the_retargets() {
        let plan = plan(Some(".archive"), vec![rewrite("live.typ", 3)]);
        assert_eq!(
            retire_prompt_lines(&plan),
            [
                "rm will retire 1 task(s) to done:",
                "  - gone.typ",
                "and move 1 task(s) into .archive/",
                "and retarget 3 link(s) in 1 file(s)",
            ]
        );
    }

    #[test]
    fn the_retire_prompt_in_place_has_no_move_or_retarget_line() {
        assert_eq!(
            retire_prompt_lines(&plan(None, Vec::new())),
            ["rm will retire 1 task(s) to done:", "  - gone.typ"]
        );
    }
}
