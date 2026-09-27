//! `mt check --commit-msg <file>` — hold the commit being written to
//! `[commits]`: its task trailers name live tasks by stem, and it passes every
//! `[[commits.rules]]` entry.

use std::path::Path;

use anyhow::{Context, Result, bail};
use mindtape::{CommitRecord, check_commit, parse_message, staged_files};

use crate::util;

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
    let breaches = check_commit(&record, &project.config);
    for breach in &breaches {
        println!(
            "{}: {shown}: {}: {}",
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
