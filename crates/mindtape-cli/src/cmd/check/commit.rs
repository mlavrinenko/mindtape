//! `mt check --commit-msg <file>` — hold the commit being written to
//! `[commits]`: its task trailers name live tasks by stem, and it passes every
//! `[[commits.rules]]` entry.

use std::path::Path;

use anyhow::Result;
use mindtape::{
    CommitBreach, CommitMsgError, CommitRecord, CommitStates, StatePlan, check_commit,
    parse_message, staged_files,
};

use crate::diag::{Diag, Kind, Note};
use crate::util;

/// A git read that failed: `error: can't read the git index: <git's reason>`.
pub(crate) fn git_failed(what: &'static str, err: &CommitMsgError) -> Diag {
    Diag::failed(what).cause(err)
}

/// The project as of `HEAD`, the index and the working tree — loaded only
/// when a `scope = "task"` rule will read it. A task file a state holds but
/// cannot evaluate is reported, and reads as absent there.
fn task_states(project: &util::Project) -> Result<Option<CommitStates>> {
    if !project.config.commits.has_task_rules() {
        return Ok(None);
    }
    let plan = StatePlan::index(&project.root)
        .map_err(|err| git_failed("can't read the git index", &err))?;
    let states = CommitStates::load(
        &project.root,
        &project.outcome.tasks,
        &project.config,
        &plan,
    )
    .map_err(|err| git_failed("can't read the staged and HEAD task files", &err))?;
    warn_state_errors(&states, None);
    Ok(Some(states))
}

/// Report every task file a state holds but cannot evaluate, as a warning on
/// stderr. In a range, `commit` (id and subject) leads the message with the
/// commit's id.
pub(crate) fn warn_state_errors(states: &CommitStates, commit: Option<(&str, &str)>) {
    for (name, state) in [("HEAD", &states.head), ("staged", &states.staged)] {
        for error in &state.errors {
            let diag = Diag::verbatim(
                Kind::Warning,
                format!("{} as {name}: {}", error.place(), error.message),
            );
            crate::diag::emit(&in_commit(diag, commit));
        }
    }
}

/// `diag` led by the id of the commit it is about, when there is one, and
/// followed by that commit's subject.
pub(crate) fn in_commit(diag: Diag, commit: Option<(&str, &str)>) -> Diag {
    match commit {
        Some((short, subject)) => diag
            .at_file(Path::new(short))
            .note(Note::new("commit").value(subject)),
        None => diag,
    }
}

/// A breach as a message: the rule leads, then the task and the advice.
/// `place` leads the whole of it: the message file, or a commit's id.
pub(crate) fn breach_diag(breach: &CommitBreach, place: &Path) -> Diag {
    let kind = if breach.severity.is_error() {
        Kind::Refused
    } else {
        Kind::Warning
    };
    let task = breach
        .task
        .as_deref()
        .map(|stem| format!("{stem}: "))
        .unwrap_or_default();
    Diag::verbatim(kind, format!("{task}{}", breach.message))
        .at_file(place)
        .rule(breach.label.clone())
}

/// How many of `breaches` fail the check.
pub(crate) fn failing(breaches: &[CommitBreach]) -> usize {
    breaches
        .iter()
        .filter(|breach| breach.severity.is_error())
        .count()
}

/// Check the message in `file` and the index it will be committed with.
///
/// # Errors
/// Fails when the file cannot be read, the project is not in a git
/// repository, a trailer value is a path or names no task, or an
/// error-severity rule breaks.
pub(crate) fn run(project: &util::Project, file: &Path, quiet: bool) -> Result<()> {
    let text = std::fs::read_to_string(file).map_err(|err| Diag::cannot_read(file, &err))?;
    let changed =
        staged_files(&project.root).map_err(|err| git_failed("can't read the git index", &err))?;
    let message = parse_message(&text, &project.config.commits.trailer);
    let (record, ref_problems) =
        CommitRecord::build(&message, &changed, &project.outcome.tasks, &project.config);
    for problem in &ref_problems {
        let diag = Diag::verbatim(Kind::Refused, &problem.message)
            .at(file, problem.value.line, None)
            .rule(project.config.commits.trailer.clone())
            .unknown(problem.unknown.as_ref());
        crate::diag::emit(&diag);
    }
    let states = task_states(project)?;
    let breaches = check_commit(&record, states.as_ref(), &project.config);
    for breach in &breaches {
        crate::diag::emit(&breach_diag(breach, file));
    }
    let failures = ref_problems.len() + failing(&breaches);
    if failures > 0 {
        return Err(Diag::commit_message_problems(failures).into());
    }
    if !quiet {
        println!("ok: commit message valid");
    }
    Ok(())
}

#[cfg(test)]
#[path = "commit_tests.rs"]
mod tests;
