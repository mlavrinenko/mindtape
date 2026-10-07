//! `mt check --commits <range>` — hold every commit of a range to
//! `[commits]`, as `mt check --commit-msg` holds the one being written: a hook
//! skipped with `--no-verify` still fails where the range is checked.

use anyhow::{Context, Result, bail};
use mindtape::{CommitRange, CommitRecord, CommitStates, check_commit, parse_message};

use crate::cmd::check::commit::{breach_text, failing, warn_state_errors};
use crate::util;

/// Check every commit `range` names, oldest first.
///
/// A rule reading the working tree is skipped, and said so once: a past
/// commit has none.
///
/// # Errors
/// Fails when git cannot read the range, or when any commit carries a trailer
/// naming no task or breaks an error-severity rule.
pub(crate) fn run(project: &util::Project, range: &str, quiet: bool) -> Result<()> {
    let config = &project.config;
    for (index, spec) in config.commits.rules.iter().enumerate() {
        if spec.reads_worktree() {
            println!(
                "skip: {}: reads the working tree, which a past commit has none of",
                spec.label(index)
            );
        }
    }
    let commits = CommitRange::read(&project.root, range)
        .with_context(|| format!("--commits reads the range {range}"))?;
    let mut broke = 0;
    for commit in &commits.commits {
        let read = || format!("--commits reads commit {}", commit.short);
        let past = commits.commit(&project.root, commit).with_context(read)?;
        let states = CommitStates::load(&project.root, &project.outcome.tasks, config, &past.plan)
            .with_context(read)?;
        let message = parse_message(&past.message, &config.commits.trailer);
        // Trailers resolve against the tasks the commit itself holds.
        let (record, problems) =
            CommitRecord::build(&message, &past.changed, &states.staged.tasks, config);
        let breaches = check_commit(&record, Some(&states), config);
        let has_errors = states.head.errors.len() + states.staged.errors.len() > 0;
        if problems.is_empty() && breaches.is_empty() && !has_errors {
            continue;
        }
        println!("{} {}", commit.short, message.subject);
        warn_state_errors(&states, "  ");
        for problem in &problems {
            println!("  error: {}: {}", config.commits.trailer, problem.message);
        }
        for breach in &breaches {
            println!("  {}: {}", breach.severity.label(), breach_text(breach));
        }
        if !problems.is_empty() || failing(&breaches) > 0 {
            broke += 1;
        }
    }
    if broke > 0 {
        bail!("{broke} commit(s) broke commit rules");
    }
    if !quiet {
        println!("ok: {} commit(s) pass commit rules", commits.commits.len());
    }
    Ok(())
}

#[cfg(test)]
#[path = "range_tests.rs"]
mod tests;
