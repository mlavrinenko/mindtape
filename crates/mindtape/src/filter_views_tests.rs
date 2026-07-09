#![allow(clippy::unwrap_used)]

use super::desugar_view_refs;

#[test]
fn rewrites_bare_ref() {
    assert_eq!(desugar_view_refs("@broken"), "in_view(\"broken\")");
}

#[test]
fn rewrites_composed_refs() {
    assert_eq!(
        desugar_view_refs("@summer || @broken"),
        "in_view(\"summer\") || in_view(\"broken\")"
    );
    assert_eq!(
        desugar_view_refs("@summer && !@broken"),
        "in_view(\"summer\") && !in_view(\"broken\")"
    );
}

#[test]
fn hyphen_and_underscore_in_name() {
    assert_eq!(
        desugar_view_refs("@release-2_0"),
        "in_view(\"release-2_0\")"
    );
}

#[test]
fn skips_at_inside_string_literal() {
    assert_eq!(
        desugar_view_refs("search(\"a@b.com\")"),
        "search(\"a@b.com\")"
    );
    assert_eq!(
        desugar_view_refs("@x && search(\"a@b\")"),
        "in_view(\"x\") && search(\"a@b\")"
    );
}

#[test]
fn bare_at_without_name_passes_through() {
    assert_eq!(desugar_view_refs("a @ b"), "a @ b");
}

#[test]
fn escaped_quote_inside_string_does_not_end_it() {
    // `search("a\"@b")` — the escaped quote must not close the string, so the
    // following `@b` stays literal rather than being rewritten.
    let input = "search(\"a\\\"@b\")";
    assert_eq!(desugar_view_refs(input), input);
}

#[test]
fn empty_and_plain_expressions_unchanged() {
    assert_eq!(desugar_view_refs(""), "");
    assert_eq!(
        desugar_view_refs("ready && priority > 5"),
        "ready && priority > 5"
    );
}
