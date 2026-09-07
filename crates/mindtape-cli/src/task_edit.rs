//! Shared in-place edits for the `task.with(…)` header.
//!
//! Locating the call and upserting a named argument (`reviewed:`, `status:`, …)
//! is common to every command that stamps a field onto an existing task.

use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use mindtape::{
    Candidate, Config, Violation, check_candidate, load_config, load_project, validate,
};
use typst_edit::{Call, Edit, find_method_calls};

use crate::util::Project;

/// The first `task.with(…)` call in `source`.
///
/// # Errors
/// Returns an error when the source holds no `task.with(…)` call.
pub fn task_with(source: &str) -> Result<Call> {
    find_method_calls(source, "task", "with")
        .into_iter()
        .next()
        .context("no task.with(…) call found in source")
}

/// Build an edit setting `name: value_text` in `call`: replace an existing
/// argument's value, or insert a new argument separated from the ones already
/// there.
///
/// The insertion anchors immediately after the last argument's value and
/// carries its own leading comma, rather than landing before the closing paren
/// and relying on a separator already sitting there. A call written on one line
/// (`task.with(title: slot("title"))`) or without a trailing comma has nothing
/// to separate from at the paren, and the old anchor spliced the new argument
/// straight onto the previous value — writing a file that could not parse.
/// Carrying the separator also keeps the edit self-composing: any number of
/// insertions share the anchor, apply in the order they were pushed, and each
/// arrives already separated from the one before.
///
/// An empty argument list is the opposite case — nothing precedes the new
/// argument, so it takes no leading comma and keeps the trailing one instead,
/// which is what separates a second insertion from it.
///
/// # Errors
/// Returns an error when the call range is empty (malformed source).
pub fn upsert_named_arg(source: &str, call: &Call, name: &str, value_text: &str) -> Result<Edit> {
    if let Some(arg) = call.named(name) {
        return Ok(Edit::new(arg.value_range.clone(), value_text.to_owned()));
    }
    let close_paren = call
        .range
        .end
        .checked_sub(1)
        .context("call range is empty")?;
    let Some(last) = call.args.last() else {
        let indent = detect_indent(source, call);
        return Ok(Edit::new(
            close_paren..close_paren,
            format!("{indent}{name}: {value_text},\n"),
        ));
    };
    let anchor = last.value_range.end;
    let gap = match source.get(anchor..close_paren) {
        // The closing paren is on a line of its own: one argument per line.
        Some(tail) if tail.contains('\n') => format!("\n{}", detect_indent(source, call)),
        // Everything closes on one line: stay on it.
        _ => " ".to_owned(),
    };
    Ok(Edit::new(
        anchor..anchor,
        format!(",{gap}{name}: {value_text}"),
    ))
}

/// The status name a freshly written task (`mt add`) or occurrence
/// (`mt recur` spawn/mutate) should stamp: the legend's own initial (first
/// non-terminal) value, falling back to the legend's first entry when every
/// entry happens to be terminal. `None` only when the legend is genuinely
/// empty (a project that cleared it with `status = []`) — the caller must
/// then leave the status slot unfilled and let validation report the real
/// problem, rather than inventing a value.
#[must_use]
pub fn initial_status(config: &Config) -> Option<String> {
    let legend = &config.tasks.status;
    legend
        .initial()
        .or_else(|| legend.entries.first())
        .map(|entry| entry.value.clone())
}

/// Reload the project rooted at `root` and report any load error or
/// validation violation that touches `file` — the shared post-write check
/// `mt add` and `mt recur` both run on a freshly written or rewritten task.
///
/// # Errors
/// Returns an error if the file failed to load, harvested no task marker, or
/// the reloaded task fails validation.
pub fn verify_written(root: &Path, file: &Path) -> Result<()> {
    let config = load_config(root)?;
    let outcome = load_project(root, &config);
    let rel = file.strip_prefix(root).unwrap_or(file);

    if let Some(error) = outcome.errors.iter().find(|err| err.file == rel) {
        bail!("written task failed to load: {}", error.message);
    }
    let Some(task) = outcome.tasks.iter().find(|task| task.file == rel) else {
        bail!(
            "written task at {} did not harvest a task marker",
            rel.display()
        );
    };

    let issues: Vec<_> = validate(&outcome.tasks, &config)
        .into_iter()
        .filter(|violation| violation.task_id == task.id)
        .collect();
    report_issues(&task.id, &issues, &format!("in {}", rel.display()))
}

/// Validate a rendered task *before* it is written, so a rejected write leaves
/// nothing on disk. `source` is checked as if it already sat at `file`, against
/// the project's already-loaded tasks — the same load and the same
/// [`validate`] pass [`verify_written`] runs, in the other order.
///
/// Returns `false` when `file` lies outside the project root (`mt add --into
/// /elsewhere`), where there is no project-relative path to evaluate the
/// candidate at; the caller then falls back to [`verify_written`]. A `true`
/// return means the check ran and passed.
///
/// # Errors
/// Returns an error if the source failed to evaluate, harvested no task
/// marker, or violates a rule — none of which mention a file to repair,
/// because none was created.
pub fn verify_candidate(project: &Project, file: &Path, source: &str) -> Result<bool> {
    let Ok(rel) = file.strip_prefix(&project.root) else {
        return Ok(false);
    };
    let issues = check_candidate(
        &project.root,
        &project.config,
        &project.outcome.tasks,
        Candidate { rel, source },
    )
    .map_err(|err| anyhow!("the task as filled did not load: {}", err.message))?;
    let id = file_stem(file);
    report_issues(&id, &issues, "as filled; nothing was written").map(|()| true)
}

/// Print each violation and fail naming their count, or succeed on none.
/// `scope` completes the sentence "N validation issue(s) …", so a post-write
/// check names the file and a pre-write one names the input instead.
fn report_issues(id: &str, issues: &[Violation], scope: &str) -> Result<()> {
    if issues.is_empty() {
        return Ok(());
    }
    for violation in issues {
        eprintln!("warning: {id}: {}: {}", violation.field, violation.message);
    }
    bail!("{} validation issue(s) {scope}", issues.len())
}

fn file_stem(file: &Path) -> String {
    file.file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}

#[cfg(test)]
#[path = "task_edit_tests.rs"]
mod tests;

fn detect_indent(source: &str, call: &Call) -> String {
    call.args.first().map_or_else(
        || "  ".to_owned(),
        |arg| {
            let line_begin = source[..arg.value_range.start]
                .rfind('\n')
                .map_or(0, |pos| pos + 1);
            source[line_begin..]
                .chars()
                .take_while(|ch| *ch == ' ' || *ch == '\t')
                .collect()
        },
    )
}
