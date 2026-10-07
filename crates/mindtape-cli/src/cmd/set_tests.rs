//! Unit tests for `cmd/set.rs`'s decision table: which field is writable, what
//! literal a value renders to, and what is refused before a file is read.
//! The write itself — and the property that a refusal leaves the file
//! byte-identical — is `tests/set_e2e.rs`'s.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::bare_task;

use super::{Assigned, parse_assignment, render, writable};

/// Render one assignment against a task carrying `tags`, the way `run` does.
fn rendered(token: &str, tags: &[&str]) -> anyhow::Result<Option<String>> {
    let mut task = bare_task("subject");
    task.tags = tags.iter().map(|tag| (*tag).to_owned()).collect();
    let (name, assigned) = parse_assignment(token)?;
    render(writable(&name)?, &assigned, &task)
}

#[test]
fn a_text_field_renders_a_quoted_literal() {
    assert_eq!(
        rendered("title=Wire OAuth", &[]).unwrap(),
        Some("\"Wire OAuth\"".to_owned())
    );
}

/// A quote in the value is escaped, not left to close the literal early —
/// the one mistake that would write a file the compiler cannot parse.
#[test]
fn a_text_value_is_escaped() {
    assert_eq!(
        rendered("title=say \"hi\"", &[]).unwrap(),
        Some("\"say \\\"hi\\\"\"".to_owned())
    );
}

#[test]
fn a_numeric_field_renders_a_bare_number() {
    assert_eq!(rendered("priority=8", &[]).unwrap(), Some("8".to_owned()));
    assert_eq!(
        rendered("difficulty=2.5", &[]).unwrap(),
        Some("2.5".to_owned())
    );
}

#[test]
fn a_date_field_renders_an_on_call() {
    assert_eq!(
        rendered("due=2026-07-01", &[]).unwrap(),
        Some("on(2026, 7, 1)".to_owned())
    );
}

/// A one-element array needs its trailing comma, or Typst reads it as a
/// parenthesized string.
#[test]
fn a_plain_assignment_replaces_a_list_with_one_element() {
    assert_eq!(
        rendered("tags=docs", &["backend"]).unwrap(),
        Some("(\"docs\",)".to_owned())
    );
}

#[test]
fn an_append_keeps_the_items_already_there() {
    assert_eq!(
        rendered("tags+=auth", &["backend"]).unwrap(),
        Some("(\"backend\", \"auth\")".to_owned())
    );
}

/// Appending what the task already carries is a no-op, not a repeated entry.
#[test]
fn an_append_of_a_held_item_renders_nothing() {
    assert_eq!(rendered("tags+=backend", &["backend"]).unwrap(), None);
}

#[test]
fn appending_to_a_scalar_field_is_refused_naming_the_field() {
    let err = rendered("priority+=8", &[]).unwrap_err().to_string();
    assert!(err.contains("priority"), "{err}");
    assert!(err.contains("list"), "{err}");
}

#[test]
fn a_non_numeric_value_is_refused_naming_the_field() {
    let err = rendered("priority=high", &[]).unwrap_err().to_string();
    assert!(err.contains("priority"), "{err}");
    assert!(err.contains("number"), "{err}");
}

/// An infinite value parses as a float and would render as a Typst
/// identifier, not a number.
#[test]
fn a_non_finite_number_is_refused() {
    assert!(rendered("priority=inf", &[]).is_err());
}

#[test]
fn a_non_date_value_is_refused_naming_the_format() {
    let err = rendered("due=next tuesday", &[]).unwrap_err().to_string();
    assert!(err.contains("YYYY-MM-DD"), "{err}");
}

#[test]
fn an_unknown_field_is_refused_with_a_suggestion_and_the_writable_list() {
    let err = writable("titel").unwrap_err().to_string();
    assert!(err.contains("titel"), "{err}");
    assert!(err.contains("title"), "{err}");
    assert!(
        err.contains("tags"),
        "the writable fields are listed: {err}"
    );
}

/// Every field with a writer of its own names that writer, so none of them
/// can be reported as a near-miss of a field that is writable here.
#[test]
fn a_field_written_elsewhere_names_its_own_command() {
    for (field, verb) in [
        ("status", "mt flip"),
        ("links", "mt link"),
        ("reviewed", "mt review"),
        ("recur", "mt recur"),
        ("id", "mt mv"),
    ] {
        let err = writable(field).unwrap_err().to_string();
        assert!(err.contains(verb), "{field}: {err}");
    }
    let err = writable("estimate").unwrap_err().to_string();
    assert!(err.contains("unit dict"), "{err}");
}

/// The `@` operator comes along with the shared grammar rather than being a
/// second spelling to maintain here.
#[test]
fn a_file_operator_token_reads_the_value_out_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("title.txt");
    std::fs::write(&path, "From a file\n").unwrap();
    let token = format!("title@{}", path.display());
    assert_eq!(
        rendered(&token, &[]).unwrap(),
        Some("\"From a file\"".to_owned())
    );
}

#[test]
fn a_token_with_no_operator_is_refused() {
    let err = parse_assignment("priority").unwrap_err().to_string();
    assert!(err.contains("key=value"), "{err}");
}

/// A refusal or a no-op quotes the value back exactly as the caller typed it,
/// for both operators — not a rendered literal and not a placeholder.
#[test]
fn an_assignment_quotes_back_the_value_as_typed() {
    assert_eq!(Assigned::Set("Wire OAuth".to_owned()).text(), "Wire OAuth");
    assert_eq!(Assigned::Append("auth".to_owned()).text(), "auth");
}

/// `parse_assignment` splits the name off the operator for both carriers.
#[test]
fn both_operators_parse_to_the_same_field_name() {
    let (set, _) = parse_assignment("tags=a").unwrap();
    let (append, value) = parse_assignment("tags+=b").unwrap();
    assert_eq!(set, "tags");
    assert_eq!(append, "tags");
    assert!(matches!(value, Assigned::Append(ref item) if item == "b"));
}
