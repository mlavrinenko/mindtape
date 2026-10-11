//! Library errors as `Diag`s: a `ConditionError`, a `typst_cleanup::Error`.

use mindtape::config::ConfigError;
use mindtape::{ConditionError, FilterError, Unknown};

use super::render::render;
use super::{Diag, Kind};

fn plain(diag: &Diag) -> String {
    render(diag, false)
}

#[test]
fn a_condition_error_is_a_failure_with_a_next_step() {
    let task_field: Diag = ConditionError::TaskField("priority".to_owned()).into();
    assert_eq!(task_field.kind(), Kind::Failed);
    assert_eq!(
        plain(&task_field),
        "error: a condition can't read a task field: 'priority'\n\
         hint: use on_branch(), tty(), env(), weekday(), now() or format()\n"
    );
    let invalid: Diag = ConditionError::Invalid("tty() takes no arguments".to_owned()).into();
    assert_eq!(
        plain(&invalid),
        "error: invalid condition: tty() takes no arguments\n"
    );
}

#[test]
fn every_condition_error_is_a_failure_in_one_shape() {
    let cases = [
        (
            ConditionError::Parse("a '(' is never closed".to_owned()),
            "error: can't parse condition: a '(' is never closed\n",
        ),
        (
            ConditionError::Unsupported("'+' isn't an operator".to_owned()),
            "error: unsupported in condition: '+' isn't an operator\n",
        ),
        (
            ConditionError::UnknownPredicate("ttyy".to_owned()),
            "error: unknown predicate in condition: 'ttyy'\n\
             hint: use on_branch, tty, env, weekday, now, format, nonempty, contested, named or blocked\n",
        ),
    ];
    for (err, expected) in cases {
        let diag = Diag::from(err);
        assert_eq!(diag.kind(), Kind::Failed);
        assert_eq!(plain(&diag), expected);
    }
}

#[test]
fn a_config_error_lists_its_known_names_on_a_hint() {
    let err = ConfigError::Invalid {
        message: "views.a.columns: unknown column: 'fiel'".to_owned(),
        place: None,
        unknown: Some(Unknown::among("fiel", ["file", "title"])),
    };
    assert_eq!(
        plain(&super::from_lib::config(&err)),
        "error: views.a.columns: unknown column: 'fiel'\n\
         hint: did you mean file?\n\
         hint: use one of: file, title\n"
    );
}

#[test]
fn a_list_alone_is_one_hint() {
    let err = ConfigError::Invalid {
        message: "extends: unknown pack: 'x'".to_owned(),
        place: None,
        unknown: Some(Unknown::among("x", ["a", "b"])),
    };
    assert_eq!(
        plain(&super::from_lib::config(&err)),
        "error: extends: unknown pack: 'x'\nhint: use one of: a, b\n"
    );
}

#[test]
fn an_unknown_dict_lists_the_dicts_on_a_hint() {
    let err = FilterError::UnknownDict {
        name: "verb".to_owned(),
        unknown: Unknown::among("verb", ["verbs", "tags"]),
    };
    assert_eq!(
        plain(&Diag::from(err)),
        "error: unknown dict: 'verb'\nhint: did you mean verbs?\nhint: use one of: verbs, tags\n"
    );
}

#[test]
fn an_unknown_path_set_says_how_to_declare_it() {
    let err = FilterError::UnknownPathSet {
        name: "verb".to_owned(),
        unknown: Unknown::among("verb", ["verbs"]),
    };
    assert_eq!(
        plain(&Diag::from(err)),
        "error: unknown path set: '@verb': declare it as [dicts.verb]\n\
         hint: did you mean verbs?\nhint: use one of: verbs\n"
    );
}

fn cleanup_text(err: typst_cleanup::Error) -> String {
    crate::diag::plain(&Diag::from(err))
}

#[test]
fn a_cleanup_failure_is_worded_by_what_went_wrong() {
    use swhid_mint::git::Error as Mint;
    use typst_cleanup::Error;
    let path = || std::path::PathBuf::from("tasks/a.typ");
    let commit = "hint: commit the file first: the tombstone pins its last-touch commit\n";

    assert_eq!(
        cleanup_text(Error::OutsideRepo(path())),
        "error: task file is outside the git repository: 'tasks/a.typ'\n"
    );
    assert_eq!(
        cleanup_text(Error::Mint(path(), Mint::NotTracked("a".into()))),
        format!("error: task file is not tracked by git: 'tasks/a.typ'\n{commit}")
    );
    assert_eq!(
        cleanup_text(Error::Mint(path(), Mint::Dirty("a".into()))),
        format!("error: task file has uncommitted changes: 'tasks/a.typ'\n{commit}")
    );
    assert_eq!(
        cleanup_text(Error::Mint(path(), Mint::Git("boom".into()))),
        "error: tasks/a.typ: git: boom\n"
    );
    let denied = || std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        cleanup_text(Error::Read(path(), denied())),
        "error: can't read: 'tasks/a.typ': permission denied\n"
    );
    assert_eq!(
        cleanup_text(Error::Write(path(), denied())),
        "error: can't write: 'tasks/a.typ': permission denied\n"
    );
    assert_eq!(
        cleanup_text(Error::Git("no repository".into())),
        "error: git: no repository\n"
    );
    let splice = typst_edit::EditError::NotCharBoundary { offset: 1 };
    assert!(
        cleanup_text(Error::Rewrite(path(), splice))
            .starts_with("error: can't rewrite links: 'tasks/a.typ': ")
    );
}
