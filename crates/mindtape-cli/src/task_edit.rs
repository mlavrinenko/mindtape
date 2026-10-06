//! Shared in-place edits for the `task.with(…)` header.
//!
//! Locating the call and upserting a named argument (`reviewed:`, `status:`, …)
//! is common to every command that stamps a field onto an existing task.

use std::path::Path;

use anyhow::{Context, Result, bail};
use mindtape::{Config, Violation, load_config, load_project, validate};
use typst_edit::{Call, Edit, find_method_calls};

use crate::pre_write::Pending;
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

/// The status `mt add` files a task from `template` at: `explicit`
/// (`--status`), else the status the template's own `status:` names, as a
/// shortcut (`done(…)`) or the generic `status("done", …)`, else
/// [`initial_status`]. The template's date and note are not kept; the status
/// is stamped today.
#[must_use]
pub fn filing_status(config: &Config, template: &str, explicit: Option<&str>) -> Option<String> {
    explicit
        .map(ToOwned::to_owned)
        .or_else(|| template_status(template))
        .or_else(|| initial_status(config))
}

/// The status name a template's `status:` argument writes, if it is a status
/// call.
fn template_status(template: &str) -> Option<String> {
    let call = task_with(template).ok()?;
    let value = template.get(call.named("status")?.value_range.clone())?;
    let (callee, args) = value.split_once('(')?;
    let callee = callee.trim();
    if mindtape::STATUS_SHORTCUTS.contains(&callee) {
        return Some(callee.to_owned());
    }
    let (name, _) = (callee == "status")
        .then(|| args.trim_start().strip_prefix('"'))??
        .split_once('"')?;
    Some(name.to_owned())
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
    report_issues(&issues, &format!("in {}", rel.display()))
}

/// Print every violation and fail naming how many were fatal, or succeed when
/// none was. `scope` completes the sentence "N validation issue(s) …", naming
/// the file a post-write check found the problem in. Each line carries its
/// outcome in the prefix — `error:` for a fatal violation, `warning:` for a
/// breach the project declared at that severity — so the per-issue lines match
/// the refusal or the landing that follows them.
///
/// A `[[tasks.rules]]` breach at `severity = "warn"` prints alongside the rest
/// and is then not counted: the file already exists by the time this runs, and
/// a warning is a report about it, not a reason to call the write a failure.
fn report_issues(issues: &[Violation], scope: &str) -> Result<()> {
    for violation in issues {
        eprintln!("{}", crate::breach::line(violation));
    }
    let fatal = issues.iter().filter(|issue| issue.is_fatal()).count();
    if fatal == 0 {
        return Ok(());
    }
    bail!("{fatal} validation issue(s) {scope}")
}

/// Why a filled task was refused when it did not load, with the prelude hint
/// when the failure is the missing `@local/mindtape` package: `mt add` in a
/// project with no tasks yet has no load outcome to derive that hint from.
fn load_refusal(failure: &str) -> String {
    match crate::install_hint::failure_prelude_hint(failure) {
        Some(hint) => format!("the task as filled did not load: {failure}\n{hint}"),
        None => format!("the task as filled did not load: {failure}"),
    }
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
/// `mt add`'s door onto [`crate::pre_write`] — one file, and the only writer
/// with somewhere to fall back to when the destination is out of root.
///
/// # Errors
/// Returns an error if the source failed to evaluate, harvested no task
/// marker, or violates a rule — none of which mention a file to repair,
/// because none was created.
pub fn verify_candidate(project: &Project, file: &Path, source: &str) -> Result<bool> {
    let pending = [Pending { file, source }];
    let verdict = crate::pre_write::check(project, &project.outcome.tasks, &pending)?
        .into_iter()
        .next()
        // One verdict per pending write, and exactly one was asked about.
        .unwrap_or_else(unchecked_fallback);
    if !verdict.checked {
        return Ok(false);
    }
    // A warning says its piece whether or not the write is refused, and never
    // decides that it is.
    if let Some(failure) = &verdict.load_failure {
        bail!("{}", load_refusal(failure));
    }
    for violation in &verdict.violations {
        eprintln!("{}", crate::breach::line(violation));
    }
    if verdict.accepted() {
        return Ok(true);
    }
    Err(Refused {
        count: verdict.problems().len(),
        reads: verdict.reads(),
    }
    .into())
}

/// [`verify_candidate`]'s refusal of a source that loaded, carrying the task
/// fields its breaches read, so a caller can tell what they are about.
#[derive(Debug)]
pub struct Refused {
    /// How many breaches refused the write.
    pub count: usize,
    /// The fields the fatal breaches read ([`mindtape::Violation::reads`]),
    /// or `None` when one may be about any field.
    pub reads: Option<std::collections::BTreeSet<String>>,
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.count;
        write!(
            f,
            "{count} validation issue(s) as filled; nothing was written"
        )
    }
}

impl std::error::Error for Refused {}

/// Unreachable placeholder: [`crate::pre_write::check`] answers one verdict per
/// pending write, so the single-entry slice above always yields one.
fn unchecked_fallback() -> crate::pre_write::Verdict {
    crate::pre_write::Verdict {
        load_failure: None,
        violations: Vec::new(),
        task: None,
        checked: false,
    }
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
