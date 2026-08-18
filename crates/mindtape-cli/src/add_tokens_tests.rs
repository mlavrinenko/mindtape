//! Tests for the `key OP value` token grammar — see `add_tokens.rs`.
//!
//! The `=`/`+=` half is exercised through `collect_values` in
//! `add_values_tests.rs`, where the two carriers meet; this file is about the
//! `@` operator and the splitting rule all three share.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use mindtape::RawValue;

use super::apply_token;

/// Apply one token to a fresh map and unwrap the result.
fn applied(token: &str) -> BTreeMap<String, RawValue> {
    let mut out = BTreeMap::new();
    apply_token(&mut out, token).unwrap();
    out
}

/// Apply one token to a fresh map and render its error.
fn rejected(token: &str) -> String {
    let mut out = BTreeMap::new();
    apply_token(&mut out, token).unwrap_err().to_string()
}

/// A file whose contents are `body`, in `dir`, named `name`.
fn file(dir: &tempfile::TempDir, name: &str, body: &str) -> String {
    let path = dir.path().join(name);
    std::fs::write(&path, body).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn file_operator_reads_the_value_from_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = file(&dir, "notes.txt", "Work the 7th chords hands-separate.\n");
    let values = applied(&format!("summary@{path}"));
    assert_eq!(
        values.get("summary"),
        Some(&RawValue::Str(
            "Work the 7th chords hands-separate.".to_owned()
        ))
    );
}

/// Only TRAILING line terminators go. Interior blank lines are the shape of a
/// prose or markup value and stripping them would be a conversion step, which
/// this operator deliberately has none of; leading whitespace is the first
/// line's own indentation.
#[test]
fn file_operator_strips_trailing_newlines_and_nothing_else() {
    let dir = tempfile::tempdir().unwrap();
    let path = file(&dir, "body.txt", "  first\n\nlast  \n\r\n\n");
    let values = applied(&format!("summary@{path}"));
    assert_eq!(
        values.get("summary"),
        Some(&RawValue::Str("  first\n\nlast  ".to_owned()))
    );
}

/// A file of nothing but newlines is an empty value, not an error: the
/// operator reports what the file holds and lets the slot's kind judge it.
#[test]
fn file_operator_reads_an_empty_file_as_an_empty_value() {
    let dir = tempfile::tempdir().unwrap();
    let path = file(&dir, "blank.txt", "\n\n");
    let values = applied(&format!("summary@{path}"));
    assert_eq!(values.get("summary"), Some(&RawValue::Str(String::new())));
}

/// `=` first means the `@` is data. Without one rule for both operators an
/// email address or a Typst reference in a `--set` value would need quoting
/// that no existing token does.
#[test]
fn an_at_after_the_equals_is_part_of_the_value() {
    let values = applied("summary=ping @ada about a@b.example");
    assert_eq!(
        values.get("summary"),
        Some(&RawValue::Str("ping @ada about a@b.example".to_owned()))
    );
}

/// The mirror case: `@` first means the `=` is part of the PATH. A path may
/// hold an `=` (a nix store path, a generated fixture) and the operator has
/// to reach it.
#[test]
fn an_equals_after_the_at_is_part_of_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = file(&dir, "v=1.txt", "from a path holding an equals");
    assert!(path.contains('='), "fixture must exercise the rule: {path}");
    let values = applied(&format!("summary@{path}"));
    assert_eq!(
        values.get("summary"),
        Some(&RawValue::Str("from a path holding an equals".to_owned()))
    );
}

/// A file is one value, so there is no `+@`. The error has to name the
/// channel that does take a list, or it reads as "not implemented yet".
#[test]
fn appending_a_file_is_rejected_naming_the_structured_channel() {
    let err = rejected("tags+@notes.txt");
    assert!(err.contains("tags+@"), "{err}");
    assert!(err.contains("-f/--values"), "{err}");
}

/// `-f -` is the one stdin channel, and it takes a whole mapping. A second
/// claimant would have to arbitrate against it and against the piped-title
/// read, so `@-` is refused pointing at the one that exists rather than
/// silently looking for a file named `-`.
#[test]
fn the_file_operator_refuses_stdin_naming_the_values_flag() {
    let err = rejected("summary@-");
    assert!(err.contains("summary@-"), "{err}");
    assert!(err.contains("-f -"), "{err}");
}

/// An unreadable file names the slot as well as the path: the token is one
/// argument among many and a bare io error would not say which one failed.
#[test]
fn an_unreadable_file_errors_naming_the_slot_and_the_path() {
    let err = rejected("summary@no/such/notes.txt");
    assert!(err.contains("\"summary\""), "{err}");
    assert!(err.contains("no/such/notes.txt"), "{err}");
}

/// The no-operator error is the grammar's only summary of itself, so it has
/// to list the third form too — a caller who typed a bare path is exactly the
/// one who needs to be told `@` exists.
#[test]
fn a_token_without_an_operator_names_all_three_forms() {
    let err = rejected("notes.txt");
    assert!(err.contains("key=value"), "{err}");
    assert!(err.contains("key+=value"), "{err}");
    assert!(err.contains("key@path"), "{err}");
}

/// The retired append spelling keeps naming its replacement, whichever
/// operator follows the brackets — the `[]` suffix is the mistake, and the
/// `@` form must not route around the diagnostic.
#[test]
fn the_retired_bracket_spelling_still_names_its_replacement() {
    assert!(
        rejected("tags[]=docs").contains("tags+=docs"),
        "bracket set"
    );
    let at = rejected("tags[]@notes.txt");
    assert!(at.contains("tags[]@"), "{at}");
    assert!(at.contains("tags+="), "{at}");
}
