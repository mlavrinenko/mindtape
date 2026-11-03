//! `mt check --commits <range>` — hold every commit of a range to
//! `[commits]`, as `mt check --commit-msg` holds the one being written: a hook
//! skipped with `--no-verify` still fails where the range is checked. Then the
//! `scope = "tip"` rules hold the tasks the range names to the state its
//! newest commit leaves them in.

use std::collections::BTreeSet;
use std::path::Path;

use crate::diag::Result;
use mindtape::{
    CommitBreach, CommitRange, CommitRecord, CommitStates, PastCommit, ProjectState, RangeCommit,
    RefProblem, StateLoader, check_commit, check_tip, parse_message,
};

use crate::cmd::check::commit::{StateLabels, breach_diag, failing, in_commit, warn_state_errors};
use crate::diag::{Diag, Kind, Note};
use crate::util;

/// Check every commit `range` names, oldest first. A merge is skipped unless
/// `check_merges` asks for it: it joins commits checked on their own, and its
/// first-parent diff repeats them. The newest commit still supplies the tip
/// state, merge or not.
///
/// A rule reading the working tree is skipped, and said so once: a past
/// commit has none. A `scope = "tip"` rule runs after the commits, once per
/// task the range names, over the tasks as the newest commit holds them.
///
/// # Errors
/// Fails when git cannot read the range, or when any commit carries a trailer
/// naming no task and matching no external pattern, or breaks an error-severity
/// rule.
pub(crate) fn run(
    project: &util::Project,
    range: &str,
    quiet: bool,
    check_merges: bool,
) -> Result<()> {
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
    let mut loader = StateLoader::new(&project.root, &project.outcome.tasks, config);
    let mut broke = 0;
    let mut named = BTreeSet::new();
    let mut tip = ProjectState::default();
    let mut skipped = 0;
    for (index, commit) in commits.commits.iter().enumerate() {
        if commit.is_merge && !check_merges {
            skipped += 1;
            // The newest commit's tree is still the tip, even when its own
            // message and first-parent diff are not checked.
            if index + 1 == commits.commits.len() {
                tip = tip_state(project, &commits, commit, &mut loader)?;
            }
            continue;
        }
        let one = check_one(project, &commits, commit, &mut loader)?;
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
        let checked = commits.commits.len() - skipped;
        let suffix = if skipped == 0 {
            String::new()
        } else {
            format!(" ({} skipped)", mindtape::count(skipped, "merge"))
        };
        println!(
            "ok: commit rules pass on {}{suffix}",
            mindtape::count(checked, "commit")
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

/// Read `commit` and the task states it holds.
fn read_past(
    project: &util::Project,
    range: &CommitRange,
    commit: &RangeCommit,
    loader: &mut StateLoader<'_>,
) -> Result<(PastCommit, CommitStates)> {
    let read = |err| {
        Diag::failed("can't read commit")
            .value(commit.short.clone())
            .cause(&err)
    };
    let past = range.commit(&project.root, commit).map_err(read)?;
    let states = loader.load(&past.plan).map_err(read)?;
    Ok((past, states))
}

/// The project as `commit` holds it, for the tip rules.
fn tip_state(
    project: &util::Project,
    range: &CommitRange,
    commit: &RangeCommit,
    loader: &mut StateLoader<'_>,
) -> Result<ProjectState> {
    Ok(read_past(project, range, commit, loader)?.1.staged)
}

/// Hold `commit` to `[commits]`, printing what it breaks.
fn check_one(
    project: &util::Project,
    range: &CommitRange,
    commit: &RangeCommit,
    loader: &mut StateLoader<'_>,
) -> Result<Checked> {
    let config = &project.config;
    let (past, states) = read_past(project, range, commit, loader)?;
    let message = parse_message(&past.message, &config.commits.trailer);
    // Trailers resolve against the tasks the commit itself holds.
    let (mut record, problems) =
        CommitRecord::build(&message, &past.changed, &states.staged.tasks, config);
    record.in_range = true;
    record.is_merge = commit.is_merge;
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
    // Tip rules read tasks the trailers name, not external refs that happen
    // to match a task stem later.
    let refs = record
        .refs
        .iter()
        .filter(|ref_| record.tasks.contains(ref_))
        .cloned()
        .collect();
    Ok(Checked {
        broke: !problems.is_empty() || failing(&breaches) > 0,
        refs,
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
    let labels = StateLabels {
        head: commit.parent.as_deref().unwrap_or("the empty tree"),
        staged: &commit.short,
    };
    warn_state_errors(states, labels, id);
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
