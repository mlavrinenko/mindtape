//! `mt flip`'s post-write pass: whether anything one landed batch wrote
//! fails to load or violates a rule ([`revalidate`]), asked of the project as
//! it stands after the writes.
//!
//! Split out of `flip.rs`: everything left there is about deciding and
//! performing one write, everything here is about reporting on all of them at
//! once. `mt link` shares [`after_batch`], since it writes a batch of task
//! files too and owes the same revalidation.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use mindtape::{load_config, load_project, validate};

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
pub(crate) fn after_batch(root: &Path, landed: &Landed, problems: &mut Vec<String>) -> Result<()> {
    let config = load_config(root)?;
    let outcome = load_project(root, &config);
    let after = Project {
        root: root.to_path_buf(),
        config,
        outcome,
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
fn revalidate(project: &Project, landed: &Landed, problems: &mut Vec<String>) {
    for error in &project.outcome.errors {
        if landed.files.contains(&error.file) {
            problems.push(format!(
                "{}: failed to load after flip: {}",
                error.file.display(),
                error.message
            ));
        }
    }
    for violation in validate(&project.outcome.tasks, &project.config) {
        if landed.ids.contains(&violation.task_id) && violation.is_fatal() {
            problems.push(crate::breach::at(&violation));
        }
    }
}

impl Landed {
    /// Whether the batch wrote nothing at all — then there is no reload to
    /// pay for and nothing to report on.
    pub(crate) fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[cfg(test)]
#[path = "flip_batch_tests.rs"]
mod tests;
