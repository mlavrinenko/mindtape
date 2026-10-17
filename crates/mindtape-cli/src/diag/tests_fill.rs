//! `FillError` as a [`Diag`]: what a rejected `mt add` slot fill says.

use super::render::render;
use super::{Diag, Kind};

fn plain(diag: &Diag) -> String {
    render(diag, false)
}

/// A slot fill rejected: the command line's mistakes exit 2 and say what to
/// pass instead, the template's own exit 1.
#[test]
fn a_rejected_fill_says_what_to_pass_instead() {
    use mindtape::{FillError, SlotKind};

    let missing = Diag::from(FillError::Missing(vec!["title".into(), "problem".into()]));
    assert_eq!(missing.kind(), Kind::Usage);
    assert_eq!(
        plain(&missing),
        "error: missing required slots: 'title', 'problem'\n\
         hint: pass each with --set name=value, or with -f\n\
         hint: run mt add --describe to see the slots\n"
    );
    let one = Diag::from(FillError::Missing(vec!["title".into()]));
    assert!(plain(&one).starts_with("error: missing a required slot: 'title'\n"));

    let list = Diag::from(FillError::Type {
        slot: "tags".into(),
        want: SlotKind::List,
        got: "string",
    });
    assert_eq!(
        plain(&list),
        "error: slot expects a list: 'tags': got string\n\
         hint: add items one at a time: mt add --set tags+=value\n\
         hint: or pass a JSON or YAML array with -f\n"
    );
    let typed = Diag::from(FillError::Type {
        slot: "weight".into(),
        want: SlotKind::Int,
        got: "string",
    });
    assert_eq!(
        plain(&typed),
        "error: slot has the wrong type: 'weight': expects int, got string\n"
    );
}

/// The rest of a rejected fill: a value outside the choices, a bad date, a
/// template with an unknown kind, bad markup and a failed rewrite.
#[test]
fn a_rejected_fill_value_names_the_slot() {
    use mindtape::FillError;

    let choice = Diag::from(FillError::BadChoice {
        slot: "kind".into(),
        value: "fix".into(),
        choices: vec!["feat".into(), "fixup".into()],
    });
    assert_eq!(
        plain(&choice),
        "error: not one of the slot's choices: 'fix' for slot 'kind': expected one of feat, fixup\n\
         hint: did you mean fixup?\n"
    );
    let far = Diag::from(FillError::BadChoice {
        slot: "kind".into(),
        value: "zzzzzz".into(),
        choices: vec!["feat".into()],
    });
    assert!(!plain(&far).contains("hint:"));

    let date = Diag::from(FillError::BadDate {
        slot: "due".into(),
        value: "soon".into(),
    });
    assert_eq!(
        plain(&date),
        "error: not a YYYY-MM-DD date: 'soon' for slot 'due'\n"
    );

    let kind = Diag::from(FillError::UnknownKind {
        slot: "body".into(),
        kind: "typst".into(),
    });
    assert_eq!(kind.kind(), Kind::Failed);
    assert!(plain(&kind).starts_with(
        "error: slot declares an unknown kind: 'typst' in slot 'body': expected one of "
    ));

    let markup = Diag::from(FillError::BadMarkup {
        slot: "summary".into(),
        message: "unclosed delimiter".into(),
    });
    assert_eq!(
        plain(&markup),
        "error: slot is not valid Typst markup: 'summary': unclosed delimiter\n"
    );
    let edit = Diag::from(FillError::Edit("overlap".into()));
    assert_eq!(plain(&edit), "error: can't rewrite the template: overlap\n");
}

/// The rendered refusal for a mistyped kind lists every kind `mt add` accepts.
#[test]
fn an_unknown_kind_lists_every_registered_kind() {
    let diag = Diag::from(mindtape::FillError::UnknownKind {
        slot: "tags".into(),
        kind: "multi-choice".into(),
    });
    let spellings: Vec<_> = mindtape::SlotKind::ALL
        .iter()
        .map(|kind| kind.as_str())
        .collect();
    assert_eq!(
        plain(&diag),
        format!(
            "error: slot declares an unknown kind: 'multi-choice' in slot 'tags': expected one of {}\n",
            spellings.join(", ")
        )
    );
    assert!(plain(&diag).contains("choice, multichoice"));
}
