//! `mt check --commit-msg <file>` — hold the commit being written to
//! `[commits]`: its task trailers name live tasks by stem, and it passes every
//! `[[commits.rules]]` entry.

use std::path::Path;

use anyhow::{Context, Result, bail};
use mindtape::{
    CommitBreach, CommitRecord, CommitStates, StatePlan, check_commit, parse_message, staged_files,
};

use crate::util;

/// The project as of `HEAD`, the index and the working tree — loaded only
/// when a `scope = "task"` rule will read it. A task file a state holds but
/// cannot evaluate is reported, and reads as absent there.
fn task_states(project: &util::Project) -> Result<Option<CommitStates>> {
    if !project.config.commits.has_task_rules() {
        return Ok(None);
    }
    let plan = StatePlan::index(&project.root).context("--commit-msg reads the git index")?;
    let states = CommitStates::load(
        &project.root,
        &project.outcome.tasks,
        &project.config,
        &plan,
    )
    .context("--commit-msg reads the staged and HEAD task files")?;
    warn_state_errors(&states, "");
    Ok(Some(states))
}

/// Report every task file a state holds but cannot evaluate, each line led
/// by `indent`.
pub(crate) fn warn_state_errors(states: &CommitStates, indent: &str) {
    for (name, state) in [("HEAD", &states.head), ("staged", &states.staged)] {
        for error in &state.errors {
            println!(
                "{indent}warn: {} as {name}: {}",
                error.file.display(),
                error.message
            );
        }
    }
}

/// A breach's rule, task and advice: `commits.rules.<name>: <stem>: <message>`.
pub(crate) fn breach_text(breach: &CommitBreach) -> String {
    let task = breach
        .task
        .as_deref()
        .map(|stem| format!("{stem}: "))
        .unwrap_or_default();
    format!("{}: {task}{}", breach.label, breach.message)
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
    let text = std::fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
    let changed = staged_files(&project.root).context("--commit-msg reads the git index")?;
    let message = parse_message(&text, &project.config.commits.trailer);
    let (record, ref_problems) =
        CommitRecord::build(&message, &changed, &project.outcome.tasks, &project.config);
    let shown = file.display();
    for problem in &ref_problems {
        println!(
            "error: {shown}:{}: {}: {}",
            problem.value.line, project.config.commits.trailer, problem.message
        );
    }
    let states = task_states(project)?;
    let breaches = check_commit(&record, states.as_ref(), &project.config);
    for breach in &breaches {
        println!(
            "{}: {shown}: {}",
            breach.severity.label(),
            breach_text(breach)
        );
    }
    let failures = ref_problems.len() + failing(&breaches);
    if failures > 0 {
        bail!("{failures} commit message problem(s)");
    }
    if !quiet {
        println!("ok: commit message valid");
    }
    Ok(())
}

#[cfg(test)]
#[path = "check_commit_tests.rs"]
mod tests;
