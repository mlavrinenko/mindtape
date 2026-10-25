//! A breach caused by an `in_dict()` miss: the word, the dictionary and its
//! nearest entries on one hint, and the command that lists them on the next.

use mindtape::{DictMiss, Unknown, Violation};

use super::{Diag, assert_plain};

/// A breach of `rules.one-wip` whose `in_dict()` missed `Rote` in `verbs`.
fn breach(nearest: &[&str]) -> Violation {
    Violation {
        detail: None,
        dict_miss: Some(DictMiss {
            dict: "verbs".to_owned(),
            word: "Rote".to_owned(),
            nearest: nearest.iter().map(|entry| (*entry).to_owned()).collect(),
        }),
        ..super::from_lib_tests::breach()
    }
}

const REFUSED: &str = "error: tasks/x.typ:2: rules.one-wip: finish the task in progress\n";

#[test]
fn the_hint_joins_the_nearest_entries_in_order() {
    for (nearest, tail) in [
        (&[][..], ""),
        (&["note"][..], "; did you mean note?"),
        (&["note", "rate"][..], "; did you mean note or rate?"),
        (
            &["note", "rate", "rite"][..],
            "; did you mean note, rate or rite?",
        ),
    ] {
        assert_plain(
            &Diag::from(&breach(nearest)),
            &format!(
                "{REFUSED}hint: 'Rote' is not in dict verbs{tail}\n\
                 hint: run mt dicts verbs to see its entries\n"
            ),
        );
    }
}

/// The word is the task's, so it's quoted the way a shell takes it back.
#[test]
fn the_word_is_quoted_and_the_command_pastes() {
    let mut violation = breach(&[]);
    if let Some(miss) = violation.dict_miss.as_mut() {
        miss.word = "it's".to_owned();
        miss.dict = "my verbs".to_owned();
    }
    assert_plain(
        &Diag::from(&violation),
        &format!(
            "{REFUSED}hint: 'it'\\''s' is not in dict my verbs\n\
             hint: run mt dicts 'my verbs' to see its entries\n"
        ),
    );
}

/// The two-hint limit holds: an unknown name's hints come first, and the
/// miss adds only what room is left.
#[test]
fn the_miss_takes_only_the_hint_room_left() {
    let mut violation = breach(&["note"]);
    violation.unknown = Some(Unknown::among("fiel", ["file"]));
    assert_plain(
        &Diag::from(&violation),
        &format!("{REFUSED}hint: did you mean file?\nhint: use one of: file\n"),
    );
    violation.unknown = Some(Unknown::near("fiel", &["file"]));
    assert_plain(
        &Diag::from(&violation),
        &format!(
            "{REFUSED}hint: did you mean file?\n\
             hint: 'Rote' is not in dict verbs; did you mean note?\n"
        ),
    );
}
