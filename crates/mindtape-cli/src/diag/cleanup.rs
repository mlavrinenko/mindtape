//! A [`typst_cleanup::Error`] as a [`Diag`]. A task file that git can't pin a
//! tombstone to is the project's rule refusing the work, and every such file
//! is named at once. A failed read, write or splice is the work failing.

use typst_cleanup::{Error, Ineligible, Reason};

use super::{Diag, Hint, Kind, Note};

impl From<Error> for Diag {
    fn from(err: Error) -> Self {
        match err {
            Error::Ineligible(found) => ineligible(&found),
            Error::Read(path, source) => Self::cannot_read(&path, &source),
            Error::Write(path, source) => Self::cannot_write(&path, &source),
            Error::Rewrite(path, source) => Self::failed("can't rewrite links")
                .value(path.display().to_string())
                .cause(&source),
            Error::Git(message) => Self::verbatim(Kind::Failed, format!("git: {message}")),
        }
    }
}

/// The refusal for task files that can't carry a tombstone: one task by its
/// id, several as a count, each file a note saying why.
fn ineligible(found: &[Ineligible]) -> Diag {
    let head = match found {
        [only] => Diag::refused("cannot remove").value(&only.artifact.id),
        _ => Diag::refused("cannot remove several tasks"),
    };
    found
        .iter()
        .fold(head.hint(commit_first()), |diag, bad| diag.note(note(bad)))
}

/// One file and why git can't pin it.
fn note(bad: &Ineligible) -> Note {
    Note::new("file").name(format!(
        "{} {}",
        bad.artifact.rel.display(),
        words(&bad.reason)
    ))
}

/// Why git can't pin a file, as the tail of the note's sentence.
fn words(reason: &Reason) -> String {
    use swhid_mint::git::Error as Mint;
    match reason {
        Reason::OutsideRepo => "is outside the git repository".to_owned(),
        Reason::Mint(Mint::NotTracked(_)) => "is not tracked by git".to_owned(),
        Reason::Mint(Mint::Dirty(_)) => "has uncommitted changes".to_owned(),
        Reason::Mint(Mint::Git(msg)) => format!("failed in git: {msg}"),
    }
}

/// What to do about a file git can't pin.
fn commit_first() -> Hint {
    Hint::new("commit them first: a tombstone pins the last-touch commit")
}
