//! `mt check --commits <range>` — hold every commit of a range to
//! `[commits]`, as `mt check --commit-msg` holds the one being written: a hook
//! skipped with `--no-verify` still fails where the range is checked. Then the
//! `scope = "tip"` rules hold the tasks the range names to the state its
//! newest commit leaves them in.

use std::collections::BTreeSet;
use std::path::Path;

use crate::diag::Result;
use mindtape::{
    CommitBreach, CommitRange, CommitRecord, CommitStates, ProjectState, RangeCommit, RefProblem,
    check_commit, check_tip, parse_message,
};

use crate::cmd::check::commit::{breach_diag, failing, in_commit, warn_state_errors};
use crate::diag::{Diag, Kind, Note};
use crate::util;

/// Check every commit `range` names, oldest first.
///
/// A rule reading the working tree is skipped, and said so once: a past
/// commit has none. A `scope = "tip"` rule runs after the commits, once per
/// task the range names, over the tasks as the newest commit holds them.
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
    let mut named = BTreeSet::new();
    let mut tip = ProjectState::default();
    for commit in &commits.commits {
        let one = check_one(project, &commits, commit)?;
        broke += usize::from(one.broke);
        named.extend(one.refs);
        tip = one.tip;
    }
    let left = check_tip(named.iter().map(String::as_str), &tip, config);
    if let Some(newest) = commits.commits.last() {
        report_tip(newest, &left);
    }
    if broke > 0 || failing(&left) > 0 {
        return Err(Diag::commits_broke_rules(broke, failing(&left)));
    }
    if !quiet {
        println!(
            "ok: commit rules pass on {}",
            mindtape::count(commits.commits.len(), "commit")
        );
    }
    Ok(())
}

/// What checking one commit of a range found.
struct Checked {
    /// The commit has a trailer problem or breaks an error-severity rule.
    broke: bool,
    /// The stems its trailer names.
    refs: Vec<String>,
    /// The project as the commit holds it.
    tip: ProjectState,
}

/// Hold `commit` to `[commits]`, printing what it breaks.
fn check_one(
    project: &util::Project,
    range: &CommitRange,
    commit: &RangeCommit,
) -> Result<Checked> {
    let config = &project.config;
    let read = |err| {
        Diag::failed("can't read commit")
            .value(commit.short.clone())
            .cause(&err)
    };
    let past = range.commit(&project.root, commit).map_err(read)?;
    let states = CommitStates::load(&project.root, &project.outcome.tasks, config, &past.plan)
        .map_err(read)?;
    let message = parse_message(&past.message, &config.commits.trailer);
    // Trailers resolve against the tasks the commit itself holds.
    let (mut record, problems) =
        CommitRecord::build(&message, &past.changed, &states.staged.tasks, config);
    record.in_range = true;
    let breaches = check_commit(&record, Some(&states), config);
    let has_errors = states.head.errors.len() + states.staged.errors.len() > 0;
    if !(problems.is_empty() && breaches.is_empty() && !has_errors) {
        report_commit(
            commit,
            &message.subject,
            &states,
            (&problems, &breaches),
            &config.commits.trailer,
        );
    }
    Ok(Checked {
        broke: !problems.is_empty() || failing(&breaches) > 0,
        refs: record.refs,
        tip: states.staged,
    })
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

/// Print what the tip rules found, each led by the id of the newest commit,
/// whose tree they read.
fn report_tip(newest: &RangeCommit, breaches: &[CommitBreach]) {
    for breach in breaches {
        crate::diag::emit(&breach_diag(breach, Path::new(&newest.short)));
    }
}

#[cfg(test)]
#[path = "range_tests.rs"]
mod tests;
