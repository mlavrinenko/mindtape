//! A `ConditionError` as a `Diag`: its kind, words and next step.

use mindtape::ConditionError;

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
fn a_condition_error_in_an_anyhow_chain_keeps_its_words() {
    let err = anyhow::Error::from(ConditionError::TaskField("due".to_owned()))
        .context("ls.rules[0].condition");
    assert!(
        plain(&Diag::from(err)).starts_with(
            "error: ls.rules[0].condition: a condition can't read a task field: 'due'"
        )
    );
}
