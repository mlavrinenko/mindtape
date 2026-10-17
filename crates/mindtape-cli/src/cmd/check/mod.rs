//! `mt check` — validate tasks, and the tasks every template files, against
//! the project configuration.
//!
//! # Files
//!
//! - `commit.rs` — `--commit-msg`, the commit being written.
//! - `range.rs` — `--commits`, a range of commits.
//! - `set.rs` — a set named by `@name`: every member, reported together.
//! - `templates.rs` — the template pass over every `mt add --list` template.
//! - `cmd/ls/interactive/tests.rs`, `tests_rules.rs`, `*_tests.rs` — unit tests.

pub(crate) mod commit;
pub(crate) mod range;
mod set;
pub(crate) mod templates;

use std::collections::BTreeSet;

use clap::Args;
use mindtape::{BodyRules, Violation, check_body_rules, check_comments, validate};

use crate::diag::{Diag, Result};
use crate::util;

/// Arguments for `mt check`.
#[derive(Args, Default)]
pub struct CheckArgs {
    /// Don't print the `ok` line when everything passes.
    ///
    /// Errors and warnings still print.
    #[arg(short = 'q', long)]
    pub quiet: bool,
    /// Check a commit message and the staged changes, not tasks.
    ///
    /// It checks them against your commit rules, instead of checking tasks.
    /// For a `commit-msg` hook: `mt check --commit-msg "$1"`.
    #[arg(long, value_name = "FILE", conflicts_with = "commits")]
    pub commit_msg: Option<std::path::PathBuf>,
    /// Check each commit in a git range, not tasks.
    ///
    /// It checks them against your commit rules, instead of checking tasks.
    /// For CI: `mt check --commits origin/main..HEAD`.
    #[arg(long, value_name = "RANGE")]
    pub commits: Option<String>,
    /// Print the problems as JSON on stdout instead of text on stderr.
    ///
    /// The exit code is the same.

    #[arg(long, conflicts_with_all = ["commit_msg", "commits"])]
    pub json: bool,
}

/// Run the `check` command.
///
/// # Errors
/// Returns an error (non-zero exit) if any task is invalid or failed to load.
pub fn run(args: &CheckArgs) -> Result<()> {
    if let Some(held) = crate::project_set::held_set() {
        return set::run(held, args);
    }
    run_with(&util::open()?, args)
}

pub(crate) fn run_with(project: &util::Project, args: &CheckArgs) -> Result<()> {
    if let Some(file) = &args.commit_msg {
        return crate::cmd::check::commit::run(project, file, args.quiet);
    }
    if let Some(range) = &args.commits {
        return crate::cmd::check::range::run(project, range, args.quiet);
    }
    let mut report = Report::new(args.json);
    report.extend(util::load_problems(
        &project.outcome,
        crate::diag::Kind::Failed,
    ));

    crate::pre_write::with_body_rules(&project.config, |body_rules| {
        check_project(project, args, body_rules, &mut report)
    })
}

/// The problems of one `mt check`, printed as they come on stderr, or held
/// for the JSON document.
struct Report {
    json: bool,
    held: Vec<Diag>,
}

impl Report {
    fn new(json: bool) -> Self {
        Self {
            json,
            held: Vec::new(),
        }
    }

    fn extend(&mut self, diags: impl IntoIterator<Item = Diag>) {
        for diag in diags {
            if self.json {
                self.held.push(diag);
            } else {
                crate::diag::emit(&diag);
            }
        }
    }

    /// The JSON document on stdout, when asked for.
    fn finish(&self) -> Result<()> {
        if self.json {
            let problems: Vec<_> = self.held.iter().map(Diag::to_json).collect();
            let document = serde_json::json!({
                "schema_version": mindtape::SCHEMA_VERSION,
                "problems": problems,
            });
            println!("{}", serde_json::to_string_pretty(&document)?);
        }
        Ok(())
    }
}

/// What one project's check found past its load problems: the problems in
/// print order, how many fail the run, and in how many tasks.
pub(crate) struct Findings {
    pub(crate) diags: Vec<Diag>,
    pub(crate) problems: usize,
    pub(crate) tasks: usize,
}

fn check_project(
    project: &util::Project,
    args: &CheckArgs,
    body_rules: BodyRules<'_>,
    report: &mut Report,
) -> Result<()> {
    let findings = gather(project, body_rules)?;
    report.extend(findings.diags);
    report.finish()?;
    if findings.problems == 0 {
        if !args.quiet && !args.json {
            println!(
                "ok: {}",
                mindtape::count(project.outcome.tasks.len(), "task")
            );
        }
        return Ok(());
    }
    Err(Diag::problems(findings.problems, findings.tasks))
}

/// Check one project's tasks, its templates and its comments.
///
/// # Errors
/// Returns an error when a file can't be read or a rule doesn't compile.
pub(crate) fn gather(project: &util::Project, body_rules: BodyRules<'_>) -> Result<Findings> {
    let mut violations = validate(&project.outcome.tasks, &project.config);
    violations.extend(body_violations(project, body_rules)?);
    let templates = crate::cmd::check::templates::breaches(project, body_rules)?;
    violations.extend(templates.violations);
    let mut diags: Vec<Diag> = templates
        .load_warnings
        .iter()
        .chain(&templates.load_failures)
        .cloned()
        .collect();

    // `[tasks.comments]` breaches carry the severity of their policy.
    violations.extend(
        check_comments(&project.outcome.tasks, &project.config, &project.root)
            .map_err(|err| Diag::failed("can't scan tasks.comments").cause(&err))?,
    );
    diags.extend(violations.iter().map(Diag::from));

    // A `[[tasks.rules]]` breach at `severity = "warn"` has already printed;
    // it is a report, not a failure, so it is not counted here.
    let fatal: Vec<&Violation> = violations
        .iter()
        .filter(|violation| violation.is_fatal())
        .collect();
    let load_errors = project.outcome.errors.len() + templates.load_failures.len();
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
    Ok(Findings {
        diags,
        problems: fatal.len() + load_errors,
        tasks,
    })
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
