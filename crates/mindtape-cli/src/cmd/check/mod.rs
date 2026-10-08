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

    let header_rules = compile_rules(&project.config.tasks.body.headers.rules)
        .context("invalid tasks.body.headers.rules in config")?;
    let content_rules = compile_content_rules(&project.config.tasks.body.content.rules)
        .context("invalid tasks.body.content.rules in config")?;

    let body_rules = BodyRules {
        headers: &header_rules,
        content: &content_rules,
    };

    let mut violations = validate(&project.outcome.tasks, &project.config);
    violations.extend(body_violations(project, body_rules)?);
    violations.extend(crate::cmd::check::templates::breaches(project, body_rules)?);

    // `[tasks.comments]` breaches carry the severity of their policy.
    violations.extend(
        check_comments(&project.outcome.tasks, &project.config, &project.root)
            .context("scan tasks.comments")?,
    );

    for violation in &violations {
        println!("{}", crate::render::breach::line(violation));
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

/// The header and content body rules over every loaded task's source.
fn body_violations(project: &util::Project, body_rules: BodyRules<'_>) -> Result<Vec<Violation>> {
    let mut out = Vec::new();
    if body_rules.headers.is_empty() && body_rules.content.is_empty() {
        return Ok(out);
    }
    for task in &project.outcome.tasks {
        let abs = project.root.join(&task.file);
        let source =
            std::fs::read_to_string(&abs).with_context(|| format!("read {}", abs.display()))?;
        out.extend(check_body_rules(task, &source, body_rules, &project.config));
    }
    Ok(out)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_rules;
