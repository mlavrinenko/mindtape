//! The pre-write validation seam: ask what a command is about to write
//! whether it may, before it writes it.
//!
//! `mt add` opened this seam for one file
//! (`truth/tasks/mt-add-writes-an-invalid-task-before-it-validates-one.typ`);
//! this module is the shape the other writers need, and the reason they could
//! not simply call [`mindtape::check_candidate`] one file at a time:
//!
//! - a batch is one question, not N. `mt link --apply` merging an adjacency
//!   map, `mt import` landing records that cross-link, `mt recur --advance`
//!   chaining occurrences that name their predecessor — each member of those
//!   batches is only valid *given* the others, so they go to
//!   [`mindtape::check_candidates`] together;
//! - what lands is not what the caller rendered. Every write funnels through
//!   [`crate::util::write_task`], which canonicalizes; checking the source
//!   before that pass would validate text no file ever holds. [`check`]
//!   canonicalizes first, with the same [`crate::util::canonical_source`] the
//!   write uses, so the bytes checked are the bytes written;
//! - what `mt check` would refuse is refused here too. [`check`] compiles the
//!   project's header and content body rules
//!   ([`mindtape::gate`], [`mindtape::content_gate`]) once per batch and runs
//!   them over each candidate beside [`mindtape::validate`], so a task
//!   `mt check` would reject never lands in the first place.
//!
//! What this seam deliberately does NOT do is restore. A rewrite that fails
//! its check is simply not written — the file on disk is still the one the
//! user authored, untouched, and there is nothing to put back. Restoring
//! would mean a *second* write, which can fail on its own (a permission, a
//! full disk, a concurrent edit) and would leave the file in a third state
//! neither the tool nor the user can name. Not writing cannot fail.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, bail};
use mindtape::{BodyRules, Candidate, Task, Violation, compile_content_rules, compile_rules};

use crate::util::Project;

/// One write a command is about to make.
#[derive(Clone, Copy)]
pub struct Pending<'a> {
    /// Absolute path the source would land at.
    pub file: &'a Path,
    /// The source as the command rendered it, before canonicalization.
    pub source: &'a str,
}

/// What the project says about one pending write.
#[derive(Debug)]
pub struct Verdict {
    /// Why the source did not evaluate, when it did not. A source that never
    /// loaded has no fields to have got wrong, so this is not a
    /// [`Violation`].
    pub load_failure: Option<String>,
    /// Every breach of the candidate, each carrying its own severity. Only a
    /// fatal one refuses the write; a warning is said and the file lands.
    pub violations: Vec<Violation>,
    /// The task the source would load as, when it evaluated at all — what a
    /// batch that writes one file at a time folds back into its working set,
    /// so the next write is checked against a project already carrying this
    /// one.
    pub task: Option<Task>,
    /// `false` when the destination lies outside the project root, where there
    /// is no project-relative path to evaluate a candidate at. Nothing was
    /// checked, and an empty verdict then says nothing about the source.
    pub checked: bool,
}

impl Verdict {
    /// Whether the write may land: it passed, or there was nothing to check it
    /// against.
    pub(crate) fn accepted(&self) -> bool {
        self.load_failure.is_none() && !self.violations.iter().any(Violation::is_fatal)
    }

    /// The breaches that do not refuse the write.
    pub(crate) fn warnings(&self) -> impl Iterator<Item = &Violation> {
        self.violations
            .iter()
            .filter(|violation| !violation.is_fatal())
    }

    /// The task fields the fatal breaches read together
    /// ([`Violation::reads`]), or `None` when one may be about any field.
    pub(crate) fn reads(&self) -> Option<BTreeSet<String>> {
        let mut fields = BTreeSet::new();
        for violation in self
            .violations
            .iter()
            .filter(|violation| violation.is_fatal())
        {
            fields.extend(violation.reads.clone()?);
        }
        Some(fields)
    }

    /// What refuses the write, one line each: the load failure, or every fatal
    /// breach as `file:line: field: message`.
    ///
    /// The load failure says `did not load` in front of the compiler's own
    /// words, because a task that did not evaluate has no fields to have got
    /// wrong, and a bare Typst error among `field: message` lines reads as if
    /// it were one. [`Verdict::task`] being `None` on a checked entry is the
    /// machine-readable half of the same distinction.
    pub(crate) fn problems(&self) -> Vec<String> {
        if let Some(failure) = &self.load_failure {
            return vec![format!("did not load: {failure}")];
        }
        self.violations
            .iter()
            .filter(|violation| violation.is_fatal())
            .map(crate::breach::at)
            .collect()
    }
}

/// Check every entry of `pending` as one batch, as if all of them had already
/// been written, against `world` — the project's tasks as the caller believes
/// them to stand right now.
///
/// Pass `&project.outcome.tasks` for a batch computed in one go; pass a set the
/// caller has been folding accepted writes into when the batch lands file by
/// file (`mt flip`, whose per-target `[[on.flip]]` hooks run between writes).
///
/// Compiles `project.config`'s header (`[[tasks.body.headers.rules]]`) and
/// content (`[[tasks.body.content.rules]]`) body rules once for the whole
/// batch and checks every candidate against them beside
/// [`mindtape::validate`] — the same two gates `mt check` runs, so a task
/// `mt check` would reject is never written.
///
/// # Errors
/// Returns an error if those body rules do not compile — the same compile
/// step `mt check` runs, surfaced here before any candidate is evaluated.
pub fn check(project: &Project, world: &[Task], pending: &[Pending<'_>]) -> Result<Vec<Verdict>> {
    let header_rules = compile_rules(&project.config.tasks.body.headers.rules)
        .context("invalid tasks.body.headers.rules in config")?;
    let content_rules = compile_content_rules(&project.config.tasks.body.content.rules)
        .context("invalid tasks.body.content.rules in config")?;
    let body_rules = BodyRules {
        headers: &header_rules,
        content: &content_rules,
    };

    // Canonicalized here rather than at each call site: `write_task` runs the
    // same pass on the way to disk, and typstyle is idempotent, so these are
    // the bytes the file will hold.
    let sources: Vec<String> = pending
        .iter()
        .map(|entry| crate::util::canonical_source(&project.config, entry.source.to_owned()))
        .collect();

    let mut candidates = Vec::with_capacity(pending.len());
    for (entry, source) in pending.iter().zip(&sources) {
        let Ok(rel) = entry.file.strip_prefix(&project.root) else {
            return Ok(pending.iter().map(|_| unchecked()).collect());
        };
        candidates.push(Candidate { rel, source });
    }

    let answers = mindtape::check_candidates(
        &project.root,
        &project.config,
        world,
        &candidates,
        body_rules,
    );
    Ok(answers
        .into_iter()
        .zip(pending.iter().zip(&sources))
        .map(|(answer, (entry, source))| {
            let mut found = verdict(answer);
            place_on_disk(&mut found, entry.file, &project.root, source);
            found
        })
        .collect())
}

/// Move each breach of the candidate onto the file as it stands on disk.
///
/// The check ran over the rewritten source, so its lines are the candidate's:
/// an inserted field shifts every line under it. A reader opens the file that
/// is there, so each breach of the candidate's own file is placed where the
/// authored file's [`mindtape::TaskCallSpans`] say its field, or its text, or
/// its call sits. A file that is not on disk yet (`mt add`) has no lines to
/// open, so its breaches say they are in the task as filled.
fn place_on_disk(verdict: &mut Verdict, file: &Path, root: &Path, candidate: &str) {
    let Ok(rel) = file.strip_prefix(root) else {
        return;
    };
    let authored = std::fs::read_to_string(file).ok();
    let spans = authored.as_deref().map(mindtape::TaskCallSpans::parse);
    for violation in &mut verdict.violations {
        if violation.location.file() != Some(rel) {
            continue;
        }
        let line = violation.location.line();
        violation.location = match (&authored, &spans) {
            (Some(authored), Some(spans)) => {
                let placed = spans.relocate(authored, candidate, &violation.field, line);
                mindtape::Location::at(rel, placed.unwrap_or(line))
            }
            _ => mindtape::Location::Filled { line },
        };
    }
}

/// Check `pending` as one batch and refuse the whole of it if any entry is
/// rejected, so nothing is written at all.
///
/// All-or-nothing is the right shape for a batch whose members derive from one
/// another — `mt recur --advance`'s chain, where each occurrence's source is
/// the previous one rewritten, so a rejected entry condemns every entry after
/// it too.
///
/// # Errors
/// Returns an error naming how many entries were rejected, after printing each
/// problem. Nothing has been written when it does.
pub fn ensure(project: &Project, pending: &[Pending<'_>]) -> Result<()> {
    let verdicts = check(project, &project.outcome.tasks, pending)?;
    refuse_rejected(pending, &verdicts)
}

/// Check one pending write against `world` and refuse it — without writing —
/// if it is rejected. On acceptance, returns the task the source will load as,
/// for the caller to [`absorb`] into `world` before checking the next one.
///
/// `None` means the destination lies outside the project root and nothing was
/// checked; there is then no task to fold in either.
///
/// # Errors
/// Returns an error naming the problems, after printing each. Nothing has been
/// written when it does.
pub fn verify_one(
    project: &Project,
    world: &[Task],
    file: &Path,
    source: &str,
) -> Result<Option<Task>> {
    let pending = [Pending { file, source }];
    let mut verdicts = check(project, world, &pending)?;
    refuse_rejected(&pending, &verdicts)?;
    // One verdict per pending write, and exactly one was asked about.
    Ok(verdicts.pop().and_then(|verdict| verdict.task))
}

/// Fold an accepted write's task back into `world`, replacing whatever task
/// stood at the same path. The batch's next entry is then checked against a
/// project that already carries this one.
pub fn absorb(world: &mut Vec<Task>, task: Task) {
    world.retain(|existing| existing.file != task.file);
    world.push(task);
}

/// Print every entry's warnings and every rejected entry's problems, then fail
/// naming how many were rejected, or succeed when none was.
///
/// A warning prints as `warning:` and a problem as `error:`, so the prefix
/// carries each line's outcome and a reader scanning them learns what happened
/// before the trailing refusal or success.
///
/// Warnings print for accepted entries too — that is the whole of what a
/// `severity = "warn"` rule does at a write: say it, and let the file land.
/// Printing them only on refusal would make the warn level invisible exactly
/// when it is the only thing the rule has to say.
fn refuse_rejected(pending: &[Pending<'_>], verdicts: &[Verdict]) -> Result<()> {
    let mut rejected = 0_usize;
    for (entry, verdict) in pending.iter().zip(verdicts) {
        if let Some(failure) = &verdict.load_failure {
            eprintln!("error: {}: did not load: {failure}", label(entry.file));
        }
        for violation in &verdict.violations {
            eprintln!("{}", crate::breach::line(violation));
        }
        if !verdict.accepted() {
            rejected += 1;
        }
    }
    if rejected == 0 {
        return Ok(());
    }
    let failures = verdicts
        .iter()
        .filter_map(|verdict| verdict.load_failure.as_deref());
    if let Some(hint) = crate::install_hint::batch_prelude_hint(failures) {
        eprintln!("{hint}");
    }
    bail!("{rejected} task(s) rejected; nothing was written")
}

/// Turn one candidate's answer into a verdict: a load failure is the single
/// problem, a violation renders the way `mt check` and `mt import` already
/// spell one.
///
/// The load failure says `did not load` in front of the compiler's own words,
/// because the two kinds of problem are not the same kind of mistake — a task
/// that did not evaluate has no fields to have got wrong — and a bare Typst
/// error in a list of `field: message` lines reads as if it were one.
/// [`Verdict::task`] being `None` on a checked entry is the machine-readable
/// half of the same distinction.
fn verdict(result: Result<mindtape::Checked, mindtape::LoadError>) -> Verdict {
    match result {
        Err(error) => Verdict {
            load_failure: Some(error.message),
            violations: Vec::new(),
            task: None,
            checked: true,
        },
        Ok(checked) => Verdict {
            load_failure: None,
            violations: checked.violations,
            task: Some(checked.task),
            checked: true,
        },
    }
}

/// The verdict for a batch with a destination outside the project root.
///
/// The whole batch goes unchecked rather than only the stray entry: the
/// candidates are validated as a set, and a set missing one of its members is
/// a different question from the one the caller asked. `mt add`'s fallback —
/// write, then verify what landed — is what covers a single such file; a batch
/// writer warns about the destination instead
/// ([`crate::add::warn_if_outside_roots`]) and lands it unvalidated.
fn unchecked() -> Verdict {
    Verdict {
        load_failure: None,
        violations: Vec::new(),
        task: None,
        checked: false,
    }
}

/// How a problem names the file it is about: the stem, which is the task's
/// identity.
fn label(file: &Path) -> String {
    file.file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}

#[cfg(test)]
#[path = "pre_write_tests.rs"]
mod tests;
