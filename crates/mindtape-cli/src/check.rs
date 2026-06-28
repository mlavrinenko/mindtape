//! `mt check` — validate tasks against the project configuration.

use anyhow::{Context, Result, bail};
use mindtape::{Violation, check_headers, compile_rules, validate};

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

    let header_rules = compile_rules(&project.config.tasks.body.headers.rules)
        .context("invalid tasks.body.headers.rules in config")?;

    let mut violations = validate(&project.outcome.tasks, &project.config);

    if !header_rules.is_empty() {
        for task in &project.outcome.tasks {
            let abs = project.root.join(&task.file);
            let source =
                std::fs::read_to_string(&abs).with_context(|| format!("read {}", abs.display()))?;
            violations.extend(check_headers(task, &source, &header_rules, &project.config));
        }
    }

    for violation in &violations {
        print_violation(violation);
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

fn print_violation(violation: &Violation) {
    if let Some(loc) = &violation.location {
        println!(
            "{}:{}: {}: {}",
            loc.file.display(),
            loc.line,
            violation.field,
            violation.message
        );
    } else {
        println!(
            "{}: {}: {}",
            violation.task_id, violation.field, violation.message
        );
    }
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;
