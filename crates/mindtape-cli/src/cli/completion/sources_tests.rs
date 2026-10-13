#![allow(clippy::unwrap_used)]

use mindtape::test_support::{tempdir, tempdir_with_files};
use std::ffi::OsStr;

use super::{assignment, help, presets_in, tag_token, tag_values};

/// `mt init --preset` offers every built-in `mt preset ls` prints, each
/// helped by its description, then the user presets — a shadowing one once.
#[test]
fn presets_offer_the_builtins_and_the_user_presets() {
    let dir = tempdir_with_files(&[("mine.toml", ""), ("flow.toml", "")]);
    let offered = presets_in(Some(dir.path()));
    let names: Vec<String> = offered
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect();
    let mut expected: Vec<String> = crate::cmd::init::preset::list_builtins()
        .into_iter()
        .map(|(name, _)| name.to_owned())
        .collect();
    expected.push("mine".to_owned());
    assert_eq!(names, expected);
    assert!(offered.first().unwrap().get_help().is_some());
}

/// A user preset directory that is missing yields the built-ins alone, never
/// an error.
#[test]
fn presets_without_a_user_directory_are_the_builtins() {
    let dir = tempdir();
    let missing = dir.path().join("absent");
    let builtins = crate::cmd::init::preset::list_builtins().len();
    assert_eq!(presets_in(Some(&missing)).len(), builtins);
    assert_eq!(presets_in(None).len(), builtins);
}

/// A blank help line is dropped; a real one is kept. An empty help renders as
/// a dangling separator in zsh and fish, so this is a real branch, not a
/// detail.
#[test]
fn help_is_dropped_for_blank_text() {
    assert!(help("").is_none());
    assert!(help("   ").is_none());
    assert!(help("glyph").is_some());
}

/// `mt set`'s bare `<field>` token completes the writable fields it prefixes,
/// each turned into a whole `field=` token — never a bare field name that the
/// command would then reject.
#[test]
fn assignment_lists_the_writable_fields_it_prefixes() {
    let candidates: Vec<String> = assignment(OsStr::new("tit"))
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect();
    assert_eq!(candidates, vec!["title=".to_owned()]);
    // The empty token offers every writable field, no more and no less.
    assert_eq!(
        assignment(OsStr::new("")).len(),
        crate::cmd::set::writable_fields().count()
    );
}

/// A token that already carries an operator — and names no tag key — yields
/// nothing: there is no partial field name left to complete.
#[test]
fn assignment_declines_a_token_with_an_operator() {
    assert!(assignment(OsStr::new("priority=3")).is_empty());
    assert!(assignment(OsStr::new("@today")).is_empty());
}

/// A token naming some other key is not a tag token, and the key alone (no
/// operator) is not one either.
#[test]
fn tag_values_declines_a_non_tag_key() {
    assert!(tag_values("priority=3").is_none());
    assert!(tag_values("tags").is_none());
    assert!(tag_token(OsStr::new("priority=3")).is_empty());
}
