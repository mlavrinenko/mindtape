//! `mt check` — validate tasks against the project configuration.

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{
    BodyRules, Violation, check_body_rules, check_comments, compile_content_rules, compile_rules,
    validate,
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
    #[arg(long, value_name = "FILE", conflicts_with = "commits")]
    pub commit_msg: Option<std::path::PathBuf>,
    /// Check every commit of a git range against `[[commits.rules]]` instead
    /// of checking tasks, each against its first parent: its task trailers
    /// must name tasks it holds, and it must pass every rule. A rule reading
    /// the working tree is skipped and reported. Made for CI:
    /// `mt check --commits origin/main..HEAD`.
    #[arg(long, value_name = "RANGE")]
    pub commits: Option<String>,
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
    if let Some(range) = &args.commits {
        return crate::check_range::run(project, range, args.quiet);
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

    // A `[[tasks.rules]]` breach at `severity = "warn"` has already printed;
    // it is a report, not a failure, so it is not counted here.
    let failures = violations
        .iter()
        .filter(|violation| violation.is_fatal())
        .count();
    let load_errors = project.outcome.errors.len();
    if failures == 0 && load_errors == 0 {
        if !args.quiet {
            println!("ok: {} task(s) valid", project.outcome.tasks.len());
        }
        return Ok(());
    }
    bail!("{failures} violation(s), {load_errors} load error(s)")
}

/// Report one violation: `severity: file:line: field: message`.
///
/// Every line carries its severity. A `[[tasks.rules]]` breach prints the level
/// its project chose; everything `MindTape` checks itself is fatal and prints
/// `error:`. A reader scans one column for what fails and what only warns.
fn print_violation(violation: &Violation) {
    println!("{}", format_violation(violation));
}

fn format_violation(violation: &Violation) -> String {
    let prefix = violation
        .severity
        .map_or("error", |severity| severity.label());
    format!("{prefix}: {}", breach_at(violation))
}

/// A breach as `file:line: field: message`, the shape every command prints
/// after its severity prefix.
pub(crate) fn breach_at(violation: &Violation) -> String {
    let loc = &violation.location;
    format!(
        "{}:{}: {}: {}",
        loc.file.display(),
        loc.line,
        violation.field,
        violation.message
    )
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "check_tests_rules.rs"]
mod tests_rules;
