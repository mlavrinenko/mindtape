//! `mt check` — validate tasks against the project configuration.

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{
    BodyRules, LimitBreach, Violation, check_body_rules, check_comments, check_estimate_drift,
    check_limits, compile_content_rules, compile_rules, validate,
};

use crate::util;

/// Arguments for `mt check`.
#[derive(Args, Default)]
pub struct CheckArgs {
    /// Suppress the `ok: N task(s) valid` summary on success; violations,
    /// warnings, and the exit code are unaffected.
    #[arg(short = 'q', long)]
    pub quiet: bool,
    /// Check a commit message file against `[[commits.rules]]` instead of
    /// checking tasks: its task trailers must name live tasks by stem, and
    /// the index must pass every rule. Made for a `commit-msg` hook:
    /// `mt check --commit-msg "$1"`.
    #[arg(long, value_name = "FILE")]
    pub commit_msg: Option<std::path::PathBuf>,
}

/// Run the `check` command.
///
/// # Errors
/// Returns an error (non-zero exit) if any task is invalid or failed to load.
pub fn run(args: &CheckArgs) -> Result<()> {
    run_with(&util::open()?, args)
}

pub(crate) fn run_with(project: &util::Project, args: &CheckArgs) -> Result<()> {
    if let Some(file) = &args.commit_msg {
        return crate::check_commit::run(project, file, args.quiet);
    }
    util::warn_errors(&project.outcome);

    let header_rules = compile_rules(&project.config.tasks.body.headers.rules)
        .context("invalid tasks.body.headers.rules in config")?;
    let content_rules = compile_content_rules(&project.config.tasks.body.content.rules)
        .context("invalid tasks.body.content.rules in config")?;

    let mut violations = validate(&project.outcome.tasks, &project.config);

    if !header_rules.is_empty() || !content_rules.is_empty() {
        let body_rules = BodyRules {
            headers: &header_rules,
            content: &content_rules,
        };
        for task in &project.outcome.tasks {
            let abs = project.root.join(&task.file);
            let source =
                std::fs::read_to_string(&abs).with_context(|| format!("read {}", abs.display()))?;
            violations.extend(check_body_rules(task, &source, body_rules, &project.config));
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

    // A `[[tasks.rules]]` breach at `severity = "warn"` has already printed;
    // it is a report, not a failure, so it is not counted here.
    let failures = violations
        .iter()
        .filter(|violation| violation.is_fatal())
        .count();
    let load_errors = project.outcome.errors.len();
    if failures == 0 && load_errors == 0 && over_limit == 0 {
        if !args.quiet {
            println!("ok: {} task(s) valid", project.outcome.tasks.len());
        }
        return Ok(());
    }
    bail!("{failures} violation(s), {load_errors} load error(s), {over_limit} limit breach(es)")
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

/// Report one violation: at its source line when it has one, else naming the
/// task.
///
/// The `error:` / `warning:` prefix belongs to a `[[tasks.rules]]` breach and
/// nothing else, because a rule is the only violation whose level the project
/// chose (`Violation::severity`). Everything `MindTape` checks itself is fatal,
/// and a label on a report with one level tells the reader nothing while
/// pushing the file and line it came for one word further right.
fn print_violation(violation: &Violation) {
    let prefix = violation
        .severity
        .map_or_else(String::new, |severity| format!("{}: ", severity.label()));
    if let Some(loc) = &violation.location {
        println!(
            "{prefix}{}:{}: {}: {}",
            loc.file.display(),
            loc.line,
            violation.field,
            violation.message
        );
    } else {
        println!(
            "{prefix}{}: {}: {}",
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
