//! Tests for [`super::format_typst`].

use super::format_typst;

const HEADER: &str = crate::typst_task!("\n#show: task.with(\n  title: \"x\",\n)\n");

/// A body paragraph carrying hundreds of leading spaces — the exact drift
/// `mt add` left behind — is flattened to column zero.
#[test]
fn strips_deep_body_indentation() {
    let drift = format!(
        "{HEADER}\n== Summary\n\n{}indented paragraph\n",
        " ".repeat(400)
    );
    let out = format_typst(&drift);
    assert!(
        out.contains("\nindented paragraph\n"),
        "leading indentation not stripped: {out:?}"
    );
    assert!(!out.contains("     indented"), "spaces survived: {out:?}");
}

/// typstyle normalizes over-indented list nesting to its two-space canonical.
#[test]
fn normalizes_list_nesting() {
    let drift = format!("{HEADER}\n== Scope\n\n- a\n    - nested\n");
    let out = format_typst(&drift);
    assert!(
        out.contains("- a\n  - nested\n"),
        "list not normalized: {out:?}"
    );
}

/// Formatting is idempotent: a second pass over already-formatted output is a
/// no-op, so formatting a canonical file only ever cleans drift.
#[test]
fn idempotent() {
    let drift = format!("{HEADER}\n== Summary\n\n{}para\n", " ".repeat(120));
    let once = format_typst(&drift);
    let twice = format_typst(&once);
    assert_eq!(once, twice, "second pass changed the output");
}

/// A clean header round-trips unchanged — the common `mt flip` case where the
/// file was already canonical must not churn.
#[test]
fn clean_source_unchanged() {
    let clean = format!("{HEADER}\n== Summary\n\nAlready clean.\n");
    assert_eq!(format_typst(&clean), clean);
}

/// Formatting never fails the caller: source typstyle cannot handle comes back
/// as a `String`, not a panic (the cosmetic-not-correctness contract).
#[test]
fn malformed_source_does_not_panic() {
    let _ = format_typst("#show: task.with(\n  title: \"x\",");
    let _ = format_typst("");
}
