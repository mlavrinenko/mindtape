//! Reporting a batch's verdicts: print the warnings, build the refusal and
//! the notes for breaches raised on other tasks.

use std::collections::BTreeSet;
use std::path::Path;

use crate::cmd::install::hint::{names_prelude_package, prelude_hints};
use crate::diag::{Diag, Kind, Note, Result};
use crate::pre_write::{Pending, Verdict};

/// Print every entry's warnings and every rejected entry's problems, then fail
/// with the last problem, or succeed when none was rejected. The refusal has
/// no summary line of its own: the problems are the report.
///
/// A warning prints as `warning:` and a problem as `error:`, so the prefix
/// carries each line's outcome.
///
/// Warnings print for accepted entries too — that is the whole of what a
/// `severity = "warn"` rule does at a write: say it, and let the file land.
/// Printing them only on refusal would make the warn level invisible exactly
/// when it is the only thing the rule has to say.
pub(super) fn refuse_rejected(pending: &[Pending<'_>], verdicts: &[Verdict]) -> Result<()> {
    let labels: Vec<String> = pending.iter().map(|entry| label(entry.file)).collect();
    match settle(&labels, verdicts) {
        Some(refusal) => Err(refusal),
        None => Ok(()),
    }
}

/// Print what `verdicts` say, one per entry named by `labels`, and return the
/// refusal when any entry was rejected: the last problem, not yet printed, for
/// the caller to fail with.
///
/// A load failure that names the missing prelude carries the install hints,
/// once per batch. A breach the write raises on another task is a `note:` on
/// the refusal, once per batch however many entries raise it; when it is the
/// only reason, the refusal says so.
pub(crate) fn settle(labels: &[String], verdicts: &[Verdict]) -> Option<Diag> {
    let mut lines = entry_lines(labels, verdicts);
    let notes = elsewhere_lines(verdicts, &mut lines);
    let rejected = verdicts.iter().any(|verdict| !verdict.accepted());
    let last = lines.iter().rposition(|line| line.kind() != Kind::Warning);
    let refusal = match (rejected, last) {
        (false, _) => None,
        (true, Some(at)) => Some(lines.remove(at)),
        (true, None) => Some(Diag::refused("the write breaks a rule on another task")),
    };
    lines.iter().for_each(crate::diag::emit);
    refusal.map(|refusal| notes.into_iter().fold(refusal, Diag::note))
}

/// Each entry's load failure and breaches as messages, in entry order. The
/// first load failure that names the missing prelude carries the install hints.
fn entry_lines(labels: &[String], verdicts: &[Verdict]) -> Vec<Diag> {
    let mut lines: Vec<Diag> = Vec::new();
    let mut hinted = false;
    for (name, verdict) in labels.iter().zip(verdicts) {
        if let Some(failure) = &verdict.load_failure {
            let mut diag = failure.diag(name);
            if !hinted && names_prelude_package(&failure.message) {
                hinted = true;
                diag = prelude_hints(diag);
            }
            lines.push(diag);
        }
        lines.extend(verdict.violations.iter().map(Diag::from));
    }
    lines
}

/// The breaches the batch raises on other tasks, each once however many
/// entries raise it: a fatal one as a note for the refusal, a warning as a
/// message of its own in `lines`.
fn elsewhere_lines(verdicts: &[Verdict], lines: &mut Vec<Diag>) -> Vec<Note> {
    let mut notes = Vec::new();
    let mut seen = BTreeSet::new();
    for breach in verdicts.iter().flat_map(|verdict| &verdict.elsewhere) {
        let diag = Diag::from(breach);
        if !seen.insert(diag.to_string()) {
            continue;
        }
        if breach.is_fatal() {
            notes.push(Note::line(diag.to_string()));
        } else {
            lines.push(diag);
        }
    }
    notes
}

/// How a problem names the file it is about: the stem, which is the task's
/// identity.
pub(super) fn label(file: &Path) -> String {
    file.file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}
