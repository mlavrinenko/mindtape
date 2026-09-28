//! `mt check --commit-msg <file>` — hold the commit being written to
//! `[commits]`: its task trailers name live tasks by stem, and it passes every
//! `[[commits.rules]]` entry.

use std::path::Path;

use anyhow::{Context, Result, bail};
use mindtape::{CommitRecord, CommitStates, StatePlan, check_commit, parse_message, staged_files};

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
    for (name, state) in [("HEAD", &states.head), ("staged", &states.staged)] {
        for error in &state.errors {
            println!(
                "warn: {} as {name}: {}",
                error.file.display(),
                error.message
            );
        }
    }
    Ok(Some(states))
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
        let task = breach
            .task
            .as_deref()
            .map(|stem| format!("{stem}: "))
            .unwrap_or_default();
        println!(
            "{}: {shown}: {}: {task}{}",
            breach.severity.label(),
            breach.label,
            breach.message
        );
    }
    let failures = ref_problems.len()
        + breaches
            .iter()
            .filter(|breach| breach.severity.is_error())
            .count();
    if failures > 0 {
        bail!("{failures} commit message problem(s)");
    }
    if !quiet {
        println!("ok: commit message valid");
    }
    Ok(())
}
