//! `mt check --commits <range>` — hold every commit of a range to
//! `[commits]`, as `mt check --commit-msg` holds the one being written: a hook
//! skipped with `--no-verify` still fails where the range is checked.

use std::path::Path;

use crate::diag::Result;
use mindtape::{
    CommitBreach, CommitRange, CommitRecord, CommitStates, RangeCommit, RefProblem, check_commit,
    parse_message,
};

use crate::cmd::check::commit::{breach_diag, failing, in_commit, warn_state_errors};
use crate::diag::{Diag, Kind, Note};
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
    let commits = CommitRange::read(&project.root, range).map_err(|err| {
        Diag::failed("can't read commit range")
            .value(range)
            .cause(&err)
    })?;
    let mut broke = 0;
    for commit in &commits.commits {
        let read = |err| {
            Diag::failed("can't read commit")
                .value(commit.short.clone())
                .cause(&err)
        };
        let past = commits.commit(&project.root, commit).map_err(read)?;
        let states = CommitStates::load(&project.root, &project.outcome.tasks, config, &past.plan)
            .map_err(read)?;
        let message = parse_message(&past.message, &config.commits.trailer);
        // Trailers resolve against the tasks the commit itself holds.
        let (record, problems) =
            CommitRecord::build(&message, &past.changed, &states.staged.tasks, config);
        let breaches = check_commit(&record, Some(&states), config);
        let has_errors = states.head.errors.len() + states.staged.errors.len() > 0;
        if problems.is_empty() && breaches.is_empty() && !has_errors {
            continue;
        }
        report_commit(
            commit,
            &message.subject,
            &states,
            (&problems, &breaches),
            &config.commits.trailer,
        );
        if !problems.is_empty() || failing(&breaches) > 0 {
            broke += 1;
        }
    }
    if broke > 0 {
        return Err(Diag::commits_broke_rules(broke));
    }
    if !quiet {
        println!("ok: {} commit(s) pass commit rules", commits.commits.len());
    }
    Ok(())
}

/// Print what one commit of the range breaks: its task files that did not
/// evaluate, its trailer problems and its rule breaches, each led by the
/// commit's id and followed by its subject.
fn report_commit(
    commit: &RangeCommit,
    subject: &str,
    states: &CommitStates,
    (problems, breaches): (&[RefProblem], &[CommitBreach]),
    trailer: &str,
) {
    let id = Some((commit.short.as_str(), subject));
    warn_state_errors(states, id);
    for problem in problems {
        let diag = Diag::verbatim(Kind::Refused, &problem.message).rule(trailer.to_owned());
        crate::diag::emit(&in_commit(diag, id));
    }
    for breach in breaches {
        let diag = breach_diag(breach, Path::new(&commit.short));
        crate::diag::emit(&diag.note(Note::new("commit").value(subject)));
    }
}

#[cfg(test)]
#[path = "range_tests.rs"]
mod tests;
