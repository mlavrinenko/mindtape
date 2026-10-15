//! Shared in-place edits for the `task.with(…)` header.
//!
//! Locating the call and upserting a named argument (`reviewed:`, `status:`, …)
//! is common to every command that stamps a field onto an existing task.

use std::path::Path;

use mindtape::{Config, Location, Violation, load_config, load_project, validate};
use typst_edit::{Call, Edit, find_method_calls};

use crate::diag::{Diag, Kind, Result};
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
        .ok_or_else(|| Diag::failed("no task.with(…) call found in source"))
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
        .ok_or_else(|| Diag::failed("call range is empty"))?;
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

/// The source of the task file at `abs` with `name: value_text` set in its
/// `task.with(…)` call. Reads the file and writes nothing: the caller checks
/// the result before it lands.
///
/// # Errors
/// Returns an error when the file can't be read, holds no `task.with(…)`
/// call, or the edit can't be applied.
pub fn rewrite_field(abs: &Path, name: &str, value_text: &str) -> Result<String> {
    let source = std::fs::read_to_string(abs).map_err(|err| Diag::cannot_read(abs, &err))?;
    let call = task_with(&source).map_err(|err| {
        Diag::failed("can't find the task.with(…) call")
            .at_file(abs)
            .cause(&err)
    })?;
    let edit = upsert_named_arg(&source, &call, name, value_text)?;
    typst_edit::apply(&source, vec![edit]).map_err(|err| {
        Diag::failed("can't apply the edit")
            .at_file(abs)
            .cause(&err)
    })
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
        let diag = Diag::verbatim(
            Kind::Failed,
            format!("written task failed to load: {}", error.message),
        );
        return Err(match error.line {
            Some(line) => diag.at(&error.file, line, error.column),
            None => diag.at_file(&error.file),
        });
    }
    let Some(task) = outcome.tasks.iter().find(|task| task.file == rel) else {
        return Err(
            Diag::failed("written task harvested no task marker").value(rel.display().to_string())
        );
    };

    let issues: Vec<_> = validate(&outcome.tasks, &config)
        .into_iter()
        .filter(|violation| violation.task_id == task.id)
        .collect();
    report_issues(&issues)
}

/// Print every violation, and fail with the last fatal one, or succeed when
/// none was. Each line carries its outcome in the prefix — `error:` for a fatal
/// violation, `warning:` for a breach the project declared at that severity —
/// and the refusal is the last `error:` line, with no summary after it.
///
/// A `[[tasks.rules]]` breach at `severity = "warn"` prints alongside the rest
/// and is then not counted: the file already exists by the time this runs, and
/// a warning is a report about it, not a reason to call the write a failure.
fn report_issues(issues: &[Violation]) -> Result<()> {
    let mut lines: Vec<Diag> = issues.iter().map(Diag::from).collect();
    let Some(last) = lines.iter().rposition(|line| line.kind() != Kind::Warning) else {
        lines.iter().for_each(crate::diag::emit);
        return Ok(());
    };
    let refusal = lines.remove(last);
    lines.iter().for_each(crate::diag::emit);
    Err(refusal)
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
/// with somewhere to fall back to when the destination is out of root. A
/// problem names the task by `template`, the one it was filled from, since
/// there is no path yet: `<new task from template 'task'>:2`.
///
/// # Errors
/// Returns an error if the source failed to evaluate, harvested no task
/// marker, or violates a rule — none of which mention a file to repair,
/// because none was created.
pub fn verify_candidate(
    project: &Project,
    file: &Path,
    source: &str,
    template: &str,
) -> Result<bool> {
    let pending = [Pending { file, source }];
    let mut verdict = crate::pre_write::check(project, &project.outcome.tasks, &pending)?
        .into_iter()
        .next()
        // One verdict per pending write, and exactly one was asked about.
        .unwrap_or_else(unchecked_fallback);
    if !verdict.checked {
        return Ok(false);
    }
    name_template(&mut verdict, template);
    let labels = [Location::filled_label(Some(template))];
    // A warning says its piece whether or not the write is refused, and never
    // decides that it is.
    let Some(diag) = crate::pre_write::settle(&labels, std::slice::from_ref(&verdict)) else {
        return Ok(true);
    };
    if verdict.load_failure.is_some() {
        return Err(diag);
    }
    Err(diag.reading(verdict.reads()))
}

/// Name `template` on each place in `verdict` that is a line of the new task.
fn name_template(verdict: &mut crate::pre_write::Verdict, template: &str) {
    let failure = verdict
        .load_failure
        .iter_mut()
        .filter_map(|failure| failure.location.as_mut());
    let breaches = verdict
        .violations
        .iter_mut()
        .map(|breach| &mut breach.location);
    for location in failure.chain(breaches) {
        if let Location::Filled {
            template: named, ..
        } = location
        {
            *named = Some(template.to_owned());
        }
    }
}

/// Unreachable placeholder: [`crate::pre_write::check`] answers one verdict per
/// pending write, so the single-entry slice above always yields one.
fn unchecked_fallback() -> crate::pre_write::Verdict {
    crate::pre_write::Verdict {
        load_failure: None,
        violations: Vec::new(),
        elsewhere: Vec::new(),
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
