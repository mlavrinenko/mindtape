//! The tombstone strategy's file check: every task `rm` retires must be a
//! tracked, unmodified file inside the repository, so its tombstone can pin the
//! last-touch commit. All offenders are reported at once.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use mindtape::Task;

use crate::diag::{Diag, Hint, Note, Result};

/// Check every eligible task file individually: each must be committed and
/// unmodified so `swhid_mint` can pin its tombstone to the last-touch commit.
/// Collects all failures before returning so the user sees every offender in
/// one shot rather than fixing and retrying one at a time.
///
/// # Errors
/// All offending tasks in a single message with per-task reason and action.
pub(super) fn check_eligible_files(
    tasks: &[Task],
    eligible: &HashSet<String>,
    root: &Path,
    git_root: &Path,
) -> Result<()> {
    let mut problems: Vec<Offender> = Vec::new();
    for task in tasks.iter().filter(|task| eligible.contains(&task.id)) {
        let abs = root.join(&task.file);
        let reason = match abs.strip_prefix(git_root) {
            Err(_) => Reason::Outside,
            Ok(rel) => match swhid_mint::git::mint_rev_path(git_root, &rel.to_string_lossy()) {
                Ok(_) => continue,
                Err(swhid_mint::git::Error::NotTracked(_)) => Reason::Untracked,
                Err(swhid_mint::git::Error::Dirty(_)) => Reason::Dirty,
                Err(swhid_mint::git::Error::Git(msg)) => Reason::Git(msg),
            },
        };
        problems.push(Offender {
            id: task.id.clone(),
            file: task.file.clone(),
            reason,
        });
    }
    match problems.as_slice() {
        [] => Ok(()),
        [only] => Err(only.refusal()),
        many => Err(many_refusal(many)),
    }
}

/// Why a task file can't carry a tombstone.
enum Reason {
    Outside,
    Untracked,
    Dirty,
    Git(String),
}

impl Reason {
    fn words(&self) -> String {
        match self {
            Self::Outside => "is outside the git repository".to_owned(),
            Self::Untracked => "is not tracked by git".to_owned(),
            Self::Dirty => "has uncommitted changes".to_owned(),
            Self::Git(msg) => format!("failed in git: {msg}"),
        }
    }
}

/// One eligible task whose file can't carry a tombstone.
struct Offender {
    id: String,
    file: PathBuf,
    reason: Reason,
}

impl Offender {
    /// The refusal for a lone offender: the task, then its file and why.
    fn refusal(&self) -> Diag {
        Diag::refused("cannot remove")
            .value(&self.id)
            .note(self.note())
            .hint(commit_first())
    }

    fn note(&self) -> Note {
        Note::new("file").name(format!("{} {}", self.file.display(), self.reason.words()))
    }
}

/// The refusal for several offenders, each a note, so the user fixes them in
/// one pass.
fn many_refusal(offenders: &[Offender]) -> Diag {
    offenders.iter().fold(
        Diag::refused("cannot remove several tasks").hint(commit_first()),
        |diag, offender| diag.note(offender.note()),
    )
}

/// What to do about a file git can't pin.
fn commit_first() -> Hint {
    Hint::new("commit them first: a tombstone pins the last-touch commit")
}
