//! `mt flip`'s post-write pass: whether anything one landed batch wrote
//! fails to load or violates a rule ([`revalidate`]), asked of the project as
//! it stands after the writes; and how the batch reports its problems
//! ([`finish`]).
//!
//! Split out of `cmd/flip/mod.rs`: everything left there is about deciding and
//! performing one write, everything here is about reporting on all of them at
//! once. `mt link` shares [`after_batch`], since it writes a batch of task
//! files too and owes the same revalidation.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::diag::Result;
use mindtape::{load_config, load_project, validate};

use crate::cmd::flip::{FlipArgs, Via};
use crate::diag::{Command, Diag, Hint, Kind, Note};
use crate::util::Project;

/// What a batch actually wrote: the two sets answer "did this problem touch
/// something we flipped" for the revalidation.
#[derive(Default)]
pub(crate) struct Landed {
    ids: HashSet<String>,
    files: HashSet<PathBuf>,
}

impl Landed {
    pub(crate) fn record(&mut self, task: &mindtape::Task) {
        self.ids.insert(task.id.clone());
        self.files.insert(task.file.clone());
    }
}

/// The batch's post-write pass, over one reload of the project.
///
/// # Errors
/// Returns an error if the config no longer loads after the batch's writes.
pub(crate) fn after_batch_diags(
    root: &Path,
    landed: &Landed,
    problems: &mut Vec<Diag>,
) -> Result<()> {
    let config = load_config(root)?;
    let outcome = load_project(root, &config);
    let after = Project {
        root: root.to_path_buf(),
        config,
        outcome,
        hyperlinks: false,
    };
    revalidate(&after, landed, problems);
    Ok(())
}

/// Append any load error or fatal validation violation in the reloaded
/// `project` that touches a flipped file to `problems` (tolerant batch: writes
/// are kept).
///
/// A `severity = "warn"` rule breach is *not* a problem: it was already
/// reported by the pre-write check each landed file passed through, and the
/// batch is meant to carry on. Pushing it here would make a warning fail the
/// command at the summary line, which is exactly the confusion the `error:` /
/// `warning:` prefixes exist to end.
fn revalidate(project: &Project, landed: &Landed, problems: &mut Vec<Diag>) {
    for error in &project.outcome.errors {
        if landed.files.contains(&error.file) {
            let diag = match error.line {
                Some(line) => {
                    Diag::verbatim(Kind::Failed, &error.message).at(&error.file, line, error.column)
                }
                None => Diag::verbatim(Kind::Failed, &error.message).at_file(&error.file),
            };
            problems.push(diag.note(Note::new("it was written, and no longer loads")));
        }
    }
    for violation in validate(&project.outcome.tasks, &project.config) {
        if landed.ids.contains(&violation.task_id) && violation.is_fatal() {
            problems.push(Diag::from(&violation));
        }
    }
}

/// Report a batch's problems: a single target's last problem is the command's
/// error, so it gets no summary; several targets end with one.
pub(crate) fn finish(mut problems: Vec<Diag>, landed: usize, total: usize) -> Result<()> {
    crate::diag::dedupe_notes(&mut problems);
    let last = match problems.pop() {
        None => return Ok(()),
        Some(last) if total == 1 => last,
        Some(last) => {
            problems.push(last);
            Diag::batch("flipped", landed, total)
        }
    };
    for problem in &problems {
        crate::diag::emit(problem);
    }
    Err(last)
}

/// A target with a space in it that names no task is most likely a note given
/// without `-m`: say how to pass it.
pub(crate) fn note_hint(diag: Diag, spec: &str, args: &FlipArgs, via: Via) -> Diag {
    if args.note.is_some() || !spec.contains(char::is_whitespace) {
        return diag;
    }
    let status = args.status.as_deref().unwrap_or_default();
    let command = match via {
        Via::Flip => Command::mt().word("flip").arg(status),
        Via::Shortcut => Command::mt().arg(status),
    };
    let command = args
        .tasks
        .iter()
        .filter(|task| *task != spec)
        .fold(command, Command::arg)
        .word("-m")
        .arg(spec);
    diag.only_hint(Hint::command("to add a note, pass it with -m", command))
}

impl Landed {
    /// Whether the batch wrote nothing at all — then there is no reload to
    /// pay for and nothing to report on.
    pub(crate) fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[cfg(test)]
#[path = "batch_tests.rs"]
mod tests;
