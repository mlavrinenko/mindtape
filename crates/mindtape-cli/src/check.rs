//! `mindtape check` — validate tasks against the project configuration.

use anyhow::{Result, bail};
use mindtape::validate;

use crate::util;

/// Run the `check` command.
///
/// # Errors
/// Returns an error (non-zero exit) if any task is invalid or failed to load.
pub fn run() -> Result<()> {
    run_with(&util::open()?)
}

pub(crate) fn run_with(project: &util::Project) -> Result<()> {
    util::warn_errors(&project.outcome);

    let violations = validate(&project.outcome.tasks, &project.config);
    for violation in &violations {
        println!(
            "{}: {}: {}",
            violation.task_id, violation.field, violation.message
        );
    }

    let load_errors = project.outcome.errors.len();
    if violations.is_empty() && load_errors == 0 {
        println!("ok: {} task(s) valid", project.outcome.tasks.len());
        return Ok(());
    }
    bail!(
        "{} violation(s), {load_errors} load error(s)",
        violations.len()
    )
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;
