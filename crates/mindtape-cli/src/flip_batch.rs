//! `mt flip`'s post-write pass: what one landed batch owes its caller.
//!
//! Two questions come due once the writes are on disk, and both are about the
//! project as it stands NOW rather than as it was resolved: did anything the
//! batch wrote fail to load or violate a rule ([`revalidate`]), and what did
//! the batch do to the handles the caller is holding
//! ([`crate::flip_handles`]). They share one reload here rather than taking
//! one each — the full evaluation cost is the reload, and two of them could
//! also disagree about a file something else rewrote in between.
//!
//! Split out of `flip.rs` rather than added to it: that file was at 89% of its
//! cap before this pass grew, and the seam is real — everything left in
//! `flip.rs` is about deciding and performing one write, everything here is
//! about reporting on all of them at once.
//!
//! `mt link` shares [`after_batch`] with `before: None`. It writes a batch of
//! task files too, so it owes the same revalidation over the same single
//! reload; what it does not owe is the handle report, since it changes no
//! task's position in the default view. Shared rather than copied: two
//! revalidation passes would be two places for "which problems belong to this
//! batch" to drift.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use mindtape::{load_config, load_project, validate};

use crate::flip_handles::Handles;
use crate::util::Project;

/// What a batch actually wrote.
///
/// The two sets answer "did this problem touch something we flipped" for the
/// revalidation; `order` is the same ids in the order their targets were
/// given, which is the order the handle report follows so its lines line up
/// with the transition lines above them. A target named twice in one batch
/// lands twice on purpose — the report is per transition reported, not per
/// distinct task.
#[derive(Default)]
pub(crate) struct Landed {
    ids: HashSet<String>,
    files: HashSet<PathBuf>,
    order: Vec<String>,
}

impl Landed {
    pub(crate) fn record(&mut self, task: &mindtape::Task) {
        self.ids.insert(task.id.clone());
        self.files.insert(task.file.clone());
        self.order.push(task.id.clone());
    }
}

/// The batch's post-write pass, over exactly one reload of the project.
///
/// Revalidation and the handle report ask the same question — what does this
/// project look like now — so they share one answer. Loading twice would pay
/// the full evaluation cost twice and, worse, let the two halves disagree
/// about a file something else rewrote in between.
///
/// # Errors
/// Returns an error if the config no longer loads after the batch's writes.
pub(crate) fn after_batch(
    root: &Path,
    landed: &Landed,
    before: Option<&Handles>,
    problems: &mut Vec<String>,
) -> Result<()> {
    let config = load_config(root)?;
    let outcome = load_project(root, &config);
    let after = Project {
        root: root.to_path_buf(),
        config,
        outcome,
    };
    revalidate(&after, landed, problems);
    if let Some(before) = before {
        crate::flip_handles::report(before, &after, &landed.order);
    }
    Ok(())
}

/// Append any load error or validation violation in the reloaded `project`
/// that touches a flipped file to `problems` (tolerant batch: writes are kept).
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
        if landed.ids.contains(&violation.task_id) {
            problems.push(format!(
                "{}: {}: {}",
                violation.task_id, violation.field, violation.message
            ));
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
