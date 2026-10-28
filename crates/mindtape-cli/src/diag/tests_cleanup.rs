//! A `typst_cleanup::Error` as a `Diag`: each file git can't pin a tombstone
//! to is named, and a failed read, write or splice says which.

use crate::diag::{Diag, plain};

fn cleanup_text(err: typst_cleanup::Error) -> String {
    plain(&Diag::from(err))
}

#[test]
fn a_cleanup_failure_is_worded_by_what_went_wrong() {
    use swhid_mint::git::Error as Mint;
    use typst_cleanup::{Artifact, Error, Ineligible, Reason};
    let path = || std::path::PathBuf::from("tasks/a.typ");
    let bad = |id: &str, reason| Ineligible {
        artifact: Artifact {
            abs: std::path::PathBuf::from(format!("/r/tasks/{id}.typ")),
            rel: std::path::PathBuf::from(format!("tasks/{id}.typ")),
            id: id.to_owned(),
        },
        reason,
    };
    let commit = "hint: commit them first: a tombstone pins the last-touch commit\n";
    let [file_a, file_b, file_c] =
        ["a", "b", "c"].map(|id| mindtape::display_path(format!("tasks/{id}.typ")));

    assert_eq!(
        cleanup_text(Error::Ineligible(vec![bad(
            "a",
            Reason::Mint(Mint::NotTracked("a".into()))
        )])),
        format!("error: cannot remove: 'a'\n{commit}note: file: {file_a} is not tracked by git\n")
    );
    assert_eq!(
        cleanup_text(Error::Ineligible(vec![
            bad("a", Reason::OutsideRepo),
            bad("b", Reason::Mint(Mint::Dirty("b".into()))),
            bad("c", Reason::Mint(Mint::Git("boom".into()))),
        ])),
        format!(
            "error: cannot remove several tasks\n{commit}\
             note: file: {file_a} is outside the git repository\n\
             note: file: {file_b} has uncommitted changes\n\
             note: file: {file_c} failed in git: boom\n"
        )
    );
    let denied = || std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        cleanup_text(Error::Read(path(), denied())),
        format!("error: can't read: '{file_a}': permission denied\n")
    );
    assert_eq!(
        cleanup_text(Error::Write(path(), denied())),
        format!("error: can't write: '{file_a}': permission denied\n")
    );
    assert_eq!(
        cleanup_text(Error::Git("no repository".into())),
        "error: git: no repository\n"
    );
    let splice = typst_edit::EditError::NotCharBoundary { offset: 1 };
    assert!(
        cleanup_text(Error::Rewrite(path(), splice))
            .starts_with(&format!("error: can't rewrite links: '{file_a}': "))
    );
}
