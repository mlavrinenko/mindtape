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

use crate::cmd::check::commit::{
    StateLabels, breach_diag, failing, in_commit, unsupported_versions, warn_state_errors,
};
use crate::cmd::install::hint::skipped_prelude_commit_diag;
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
    skip_worktree_rules(&project.config);
    let commits = read_range(project, range)?;
    let found = check_commits(project, &commits, check_merges)?;
    if !found.tip_skipped
        && let Some(newest) = commits.commits.last()
    {
        report_tip(newest, &found.tip_breaches);
    }
    if found.broke > 0 || failing(&found.tip_breaches) > 0 {
        return Err(Diag::commits_broke_rules(
            found.broke,
            failing(&found.tip_breaches),
        ));
    }
    if !quiet {
        println!("{}", summary(&found, commits.commits.len()));
    }
    Ok(())
}

/// Say once that every rule reading the working tree is skipped in a range.
fn skip_worktree_rules(config: &mindtape::Config) {
    for (index, spec) in config.commits.rules.iter().enumerate() {
        if spec.reads_worktree() {
            println!(
                "skip: {}: reads the working tree, which a past commit has none of",
                spec.label(index)
            );
        }
    }
}

/// Read `range` against `project`'s root.
fn read_range(project: &util::Project, range: &str) -> Result<CommitRange> {
    CommitRange::read(&project.root, range).map_err(|err| {
        Diag::failed("can't read commit range")
            .value(range)
            .cause(&err)
    })
}

/// What walking a range found.
#[derive(Default)]
struct RangeFound {
    /// Commits that broke a rule or a trailer.
    broke: usize,
    /// The tip rules' breaches.
    tip_breaches: Vec<CommitBreach>,
    /// Merge commits skipped.
    merges: usize,
    /// Commits skipped for an unsupported prelude.
    prelude_skips: usize,
    /// Whether the newest commit's tree was skipped, so the tip rules did not
    /// run.
    tip_skipped: bool,
}

/// Check every commit of `range`, oldest first, and the tip rules over the
/// newest state that was not skipped.
fn check_commits(
    project: &util::Project,
    commits: &CommitRange,
    check_merges: bool,
) -> Result<RangeFound> {
    let config = &project.config;
    let mut loader = StateLoader::new(&project.root, &project.outcome.tasks, config);
    let mut found = RangeFound::default();
    let mut named = BTreeSet::new();
    let mut tip = ProjectState::default();
    for (index, commit) in commits.commits.iter().enumerate() {
        let newest = index + 1 == commits.commits.len();
        if commit.is_merge && !check_merges {
            found.merges += 1;
            // The newest commit's tree is still the tip, even when its own
            // message and first-parent diff are not checked.
            if newest {
                match tip_state(project, commits, commit, &mut loader)? {
                    Some(state) => {
                        tip = state;
                        found.tip_skipped = false;
                    }
                    None => found.tip_skipped = true,
                }
            }
            continue;
        }
        let one = check_one(project, commits, commit, &mut loader)?;
        if one.skipped {
            found.prelude_skips += 1;
            if newest {
                match one.tip {
                    Some(state) => {
                        tip = state;
                        found.tip_skipped = false;
                    }
                    None => found.tip_skipped = true,
                }
            }
            continue;
        }
        found.tip_skipped = false;
        found.broke += usize::from(one.broke);
        named.extend(one.refs);
        if let Some(state) = one.tip {
            tip = state;
        }
    }
    found.tip_breaches = if found.tip_skipped {
        Vec::new()
    } else {
        check_tip(named.iter().map(String::as_str), &tip, config)
    };
    Ok(found)
}

/// The `ok:` line, with each way a commit was left out.
fn summary(found: &RangeFound, total: usize) -> String {
    let checked = total - found.merges - found.prelude_skips;
    let mut reasons = Vec::new();
    if found.merges > 0 {
        reasons.push(format!(
            "{} skipped",
            mindtape::count(found.merges, "merge")
        ));
    }
    if found.prelude_skips > 0 {
        reasons.push(format!(
            "{} skipped for an unsupported prelude",
            mindtape::count(found.prelude_skips, "commit")
        ));
    }
    let suffix = if reasons.is_empty() {
        String::new()
    } else {
        format!(" ({})", reasons.join(", "))
    };
    format!(
        "ok: commit rules pass on {}{suffix}",
        mindtape::count(checked, "commit")
    )
}

/// What checking one commit of a range found.
struct Checked {
    /// The commit has a trailer problem or breaks an error-severity rule.
    broke: bool,
    /// The stems its trailer names.
    refs: Vec<String>,
    /// The project as the commit holds it, when its own tree loaded. A skipped
    /// commit still offers this when only its parent tree was unsupported, so
    /// a supported newest tree supplies the tip state.
    tip: Option<ProjectState>,
    /// A compared state holds a prelude this `mt` cannot read, so the commit
    /// was skipped instead of reported.
    skipped: bool,
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

/// The project as `commit` holds it, for the tip rules, or `None` when its
/// prelude is unsupported and the tip rules must be skipped with it.
fn tip_state(
    project: &util::Project,
    range: &CommitRange,
    commit: &RangeCommit,
    loader: &mut StateLoader<'_>,
) -> Result<Option<ProjectState>> {
    let (past, states) = read_past(project, range, commit, loader)?;
    let unsupported = unsupported_versions(&states.staged);
    if !unsupported.is_empty() {
        warn_skipped(commit, &past.message, &unsupported);
        return Ok(None);
    }
    Ok(Some(states.staged))
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
    let staged_unsupported = unsupported_versions(&states.staged);
    let head_unsupported = unsupported_versions(&states.head);
    if !(staged_unsupported.is_empty() && head_unsupported.is_empty()) {
        let staged_skipped = !staged_unsupported.is_empty();
        let mut all = staged_unsupported;
        for found in head_unsupported {
            if !all.iter().any(|(version, _)| *version == found.0) {
                all.push(found);
            }
        }
        warn_skipped(commit, &past.message, &all);
        return Ok(Checked {
            broke: false,
            refs: Vec::new(),
            tip: (!staged_skipped).then_some(states.staged),
            skipped: true,
        });
    }
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
        tip: Some(states.staged),
        skipped: false,
    })
}

/// Warn for each unsupported prelude version a compared tree holds, naming the
/// commit, the version, the first file and the set.
fn warn_skipped(commit: &RangeCommit, message: &str, versions: &[(&str, &Path)]) {
    let subject = message.lines().next().unwrap_or_default();
    for (version, file) in versions {
        crate::diag::emit(&skipped_prelude_commit_diag(
            &commit.short,
            subject,
            version,
            file,
        ));
    }
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
