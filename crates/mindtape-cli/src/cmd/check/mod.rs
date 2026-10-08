//! `mt check` — validate tasks, and the tasks every template files, against
//! the project configuration.
//!
//! # Files
//!
//! - `commit.rs` — `--commit-msg`, the commit being written.
//! - `range.rs` — `--commits`, a range of commits.
//! - `templates.rs` — the template pass over every `mt add --list` template.
//! - `cmd/ls/interactive/tests.rs`, `tests_rules.rs`, `*_tests.rs` — unit tests.

pub(crate) mod commit;
pub(crate) mod range;
pub(crate) mod templates;

use std::collections::BTreeSet;

use clap::Args;
use mindtape::{BodyRules, Violation, check_body_rules, check_comments, validate};

use crate::diag::{Diag, Result};
use crate::util;

/// Arguments for `mt check`.
#[derive(Args, Default)]
pub struct CheckArgs {
    /// Don't print the `ok` line when everything passes. Errors and warnings
    /// still print.
    #[arg(short = 'q', long)]
    pub quiet: bool,
    /// Check a commit message and the staged changes against your commit rules,
    /// instead of checking tasks. For a `commit-msg` hook: `mt check --commit-msg "$1"`.
    #[arg(long, value_name = "FILE", conflicts_with = "commits")]
    pub commit_msg: Option<std::path::PathBuf>,
    /// Check each commit in a git range against your commit rules, instead of
    /// checking tasks. For CI: `mt check --commits origin/main..HEAD`.
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
        return crate::cmd::check::commit::run(project, file, args.quiet);
    }
    if let Some(range) = &args.commits {
        return crate::cmd::check::range::run(project, range, args.quiet);
    }
    util::report_load_problems(&project.outcome, crate::diag::Kind::Failed);

    crate::pre_write::with_body_rules(&project.config, |body_rules| {
        check_project(project, args, body_rules)
    })
}

fn check_project(
    project: &util::Project,
    args: &CheckArgs,
    body_rules: BodyRules<'_>,
) -> Result<()> {
    let mut violations = validate(&project.outcome.tasks, &project.config);
    violations.extend(body_violations(project, body_rules)?);
    let templates = crate::cmd::check::templates::breaches(project, body_rules)?;
    violations.extend(templates.violations);
    for failure in templates
        .load_warnings
        .iter()
        .chain(&templates.load_failures)
    {
        crate::diag::emit(failure);
    }

    // `[tasks.comments]` breaches carry the severity of their policy.
    violations.extend(
        check_comments(&project.outcome.tasks, &project.config, &project.root)
            .map_err(|err| Diag::failed("can't scan tasks.comments").cause(&err))?,
    );

    for violation in &violations {
        crate::diag::emit(&Diag::from(violation));
    }

    // A `[[tasks.rules]]` breach at `severity = "warn"` has already printed;
    // it is a report, not a failure, so it is not counted here.
    let fatal: Vec<&Violation> = violations
        .iter()
        .filter(|violation| violation.is_fatal())
        .collect();
    let failures = fatal.len();
    let load_errors = project.outcome.errors.len() + templates.load_failures.len();
    if failures == 0 && load_errors == 0 {
        if !args.quiet {
            println!("ok: {} task(s) valid", project.outcome.tasks.len());
        }
        return Ok(());
    }
    let tasks = fatal
        .iter()
        .map(|violation| violation.task_id.as_str())
        .chain(
            project
                .outcome
                .errors
                .iter()
                .filter_map(|error| error.file.to_str()),
        )
        .collect::<BTreeSet<_>>()
        .len()
        + templates.load_failures.len();
    Err(Diag::problems(failures + load_errors, tasks))
}

/// The header and content body rules over every loaded task's source.
fn body_violations(project: &util::Project, body_rules: BodyRules<'_>) -> Result<Vec<Violation>> {
    let mut out = Vec::new();
    if body_rules.headers.is_empty() && body_rules.content.is_empty() {
        return Ok(out);
    }
    for task in &project.outcome.tasks {
        let abs = project.root.join(&task.file);
        let source = std::fs::read_to_string(&abs).map_err(|err| Diag::cannot_read(&abs, &err))?;
        out.extend(check_body_rules(task, &source, body_rules, &project.config));
    }
    Ok(out)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_rules;
