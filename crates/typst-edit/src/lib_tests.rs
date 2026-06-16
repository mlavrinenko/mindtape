#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{Edit, EditError, apply, find_link_targets};

#[test]
fn applies_edits_regardless_of_input_order() {
    let src = "alpha beta gamma";
    let edits = vec![Edit::new(11..16, "GAMMA"), Edit::new(0..5, "ALPHA")];
    assert_eq!(apply(src, edits).expect("apply"), "ALPHA beta GAMMA");
}

#[test]
fn adjacent_edits_are_allowed() {
    let src = "abcd";
    let edits = vec![Edit::new(0..2, "X"), Edit::new(2..4, "Y")];
    assert_eq!(apply(src, edits).expect("apply"), "XY");
}

#[test]
fn rejects_overlapping_edits() {
    let src = "abcdef";
    let edits = vec![Edit::new(0..3, "X"), Edit::new(2..4, "Y")];
    assert!(matches!(apply(src, edits), Err(EditError::Overlap { .. })));
}

#[test]
fn rejects_out_of_bounds_edit() {
    let src = "abc";
    let edits = vec![Edit::new(2..9, "X")];
    assert!(matches!(
        apply(src, edits),
        Err(EditError::OutOfBounds { .. })
    ));
}

#[test]
fn rejects_edit_splitting_a_char() {
    let src = "é"; // two bytes
    let edits = vec![Edit::new(0..1, "x")];
    assert!(matches!(
        apply(src, edits),
        Err(EditError::NotCharBoundary { .. })
    ));
}

#[test]
fn finds_link_target_value_and_range() {
    let src = "see #link(\"old.typ\")[here]";
    let targets = find_link_targets(src);
    assert_eq!(targets.len(), 1);
    let target = targets.first().expect("one");
    assert_eq!(target.url, "old.typ");
    assert_eq!(src.get(target.range.clone()), Some("\"old.typ\""));
}

#[test]
fn rewrites_link_url_via_apply() {
    let src = "#link(\"old.typ\")[label]";
    let target = find_link_targets(src).first().cloned().expect("one");
    let out = apply(
        src,
        vec![Edit::new(target.range, "\"swh:1:rev:abc;path=x.typ\"")],
    )
    .expect("apply");
    assert_eq!(out, "#link(\"swh:1:rev:abc;path=x.typ\")[label]");
}

#[test]
fn ignores_link_inside_raw_block() {
    let src = "```\n#link(\"x.typ\")\n```";
    assert!(find_link_targets(src).is_empty());
}

#[test]
fn ignores_non_link_calls() {
    let src = "#image(\"x.png\")";
    assert!(find_link_targets(src).is_empty());
}

#[test]
fn finds_multiple_links() {
    let src = "#link(\"a.typ\") and #link(\"b.typ\")";
    let urls: Vec<_> = find_link_targets(src).into_iter().map(|t| t.url).collect();
    assert_eq!(urls, vec!["a.typ", "b.typ"]);
}
