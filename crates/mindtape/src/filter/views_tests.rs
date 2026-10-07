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
        desugar_view_refs("contains(\"a@b.com\")"),
        "contains(\"a@b.com\")"
    );
    assert_eq!(
        desugar_view_refs("@x && contains(\"a@b\")"),
        "in_view(\"x\") && contains(\"a@b\")"
    );
}

#[test]
fn bare_at_without_name_passes_through() {
    assert_eq!(desugar_view_refs("a @ b"), "a @ b");
}

#[test]
fn escaped_quote_inside_string_does_not_end_it() {
    // `contains("a\"@b")` — the escaped quote must not close the string, so
    // the following `@b` stays literal rather than being rewritten.
    let input = "contains(\"a\\\"@b\")";
    assert_eq!(desugar_view_refs(input), input);
}

#[test]
fn string_end_before_view_ref() {
    // A closing `"` must restore the scanning context so that a following @ref
    // is desugared.  The mutant "ch != '\\'" eats the closing quote as an
    // "escaped" char, keeping in_str=true and suppressing the desugar.
    assert_eq!(desugar_view_refs("\"a\"@b"), "\"a\"in_view(\"b\")");
}

#[test]
fn empty_and_plain_expressions_unchanged() {
    assert_eq!(desugar_view_refs(""), "");
    assert_eq!(
        desugar_view_refs("ready && priority > 5"),
        "ready && priority > 5"
    );
}
