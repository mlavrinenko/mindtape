//! `mt check` — validate tasks against the project configuration.

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{
    LimitBreach, Violation, check_comments, check_estimate_drift, check_headers, check_limits,
    compile_rules, validate,
};

use crate::util;

/// Arguments for `mt check`.
#[derive(Args, Default)]
pub struct CheckArgs {
    /// Suppress the `ok: N task(s) valid` summary on success; violations,
    /// warnings, and the exit code are unaffected.
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

/// Run the `check` command.
///
/// # Errors
/// Returns an error (non-zero exit) if any task is invalid or failed to load.
pub fn run(args: &CheckArgs) -> Result<()> {
    run_with(&util::open()?, args)
}

pub(crate) fn run_with(project: &util::Project, args: &CheckArgs) -> Result<()> {
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

    // `[tasks.comments]`: `forbid` joins the per-task violations below (and so
    // fails the command); `warn`/`allow` print (if anything) without failing.
    let comment_violations = check_comments(&project.outcome.tasks, &project.config, &project.root)
        .context("scan tasks.comments")?;
    if project.config.tasks.comments.policy.blocks() {
        violations.extend(comment_violations);
    } else {
        comment_violations.iter().for_each(print_violation);
    }

    for violation in &violations {
        print_violation(violation);
    }

    // Work-in-progress caps: a state of the backlog rather than a malformed
    // artifact, so a breach reports separately from the per-task violations and
    // only fails the command at `severity = "error"`.
    let breaches = check_limits(&project.outcome.tasks, &project.config)
        .context("invalid tasks.limits in config")?;
    for breach in &breaches {
        print_breach(breach);
    }
    let over_limit = breaches
        .iter()
        .filter(|breach| breach.severity.is_error())
        .count();

    report_drift(project);

    let load_errors = project.outcome.errors.len();
    if violations.is_empty() && load_errors == 0 && over_limit == 0 {
        if !args.quiet {
            println!("ok: {} task(s) valid", project.outcome.tasks.len());
        }
        return Ok(());
    }
    bail!(
        "{} violation(s), {load_errors} load error(s), {over_limit} limit breach(es)",
        violations.len()
    )
}

/// Report how far the git working tree has drifted past the file budget the
/// in-flight tasks declare.
///
/// Never touches the exit status. An unforeseen file is normal and the answer is
/// a rescope, not a failed command — so a breach only warns, and even an
/// unusable `[tasks.estimate.drift] in-flight` filter reports as a warning
/// rather than turning a read command into a failure.
fn report_drift(project: &util::Project) {
    match check_estimate_drift(&project.outcome.tasks, &project.config, &project.root) {
        Err(err) => println!("warning: tasks.estimate.drift.in-flight: {err}"),
        Ok(warnings) => {
            for warning in &warnings {
                println!("warning: {warning}");
                if warning.tasks.len() > 1 {
                    println!("  in flight: {}", warning.tasks.join(", "));
                }
            }
        }
    }
}

/// Report one breached `[[tasks.limits]]` entry: the count, the cap, the advice,
/// then the tasks that make up the matched set.
fn print_breach(breach: &LimitBreach) {
    println!("{}: {breach}", breach.severity.label());
    for id in &breach.tasks {
        println!("  - {id}");
    }
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

#[cfg(test)]
#[path = "check_tests_limits.rs"]
mod tests_limits;
