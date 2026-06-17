#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{Edit, EditError, apply, find_calls, find_link_targets, line_of};

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

#[test]
fn find_calls_reads_positional_args_and_body() {
    let src = "see #git-ref(\"truth/tasks/foo.typ\", \"deadbeef\")[Foo]";
    let calls = find_calls(src, "git-ref");
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!(call.name, "git-ref");
    assert_eq!(call.positional(0).unwrap().value, "truth/tasks/foo.typ");
    assert_eq!(call.positional(1).unwrap().value, "deadbeef");
    let body = call.body.as_ref().unwrap();
    assert_eq!(body.text, "Foo");
    // head_range excludes the [Foo] block, so a caller can swap the call and keep
    // the label verbatim.
    assert_eq!(
        src.get(call.head_range()),
        Some("git-ref(\"truth/tasks/foo.typ\", \"deadbeef\")")
    );
}

#[test]
fn find_calls_reads_named_arg_value_span() {
    let src = "#scenario(\"name\", stage: \"wip\")";
    let call = &find_calls(src, "scenario")[0];
    let stage = call.named("stage").unwrap();
    assert_eq!(stage.value, "wip");
    // value_range covers the quoted literal, ready to splice a new stage in place.
    assert_eq!(src.get(stage.value_range.clone()), Some("\"wip\""));
    assert!(call.named("missing").is_none());
}

#[test]
fn find_calls_named_arg_with_bare_identifier_value() {
    let src = "#scenario(stage: wip)";
    let stage = find_calls(src, "scenario")[0]
        .named("stage")
        .cloned()
        .unwrap();
    assert_eq!(stage.value, "wip");
    assert_eq!(src.get(stage.value_range), Some("wip"));
}

#[test]
fn migrate_git_ref_to_swh_link_preserving_label() {
    let src = "#git-ref(\"foo.typ\", \"abc\")[Foo]";
    let call = find_calls(src, "git-ref").into_iter().next().unwrap();
    let path = call.positional(0).unwrap().value.clone();
    let commit = call.positional(1).unwrap().value.clone();
    let replacement = format!("link(\"swh:1:rev:{commit};path={path}\")");
    let out = apply(src, vec![Edit::new(call.head_range(), replacement)]).unwrap();
    assert_eq!(out, "#link(\"swh:1:rev:abc;path=foo.typ\")[Foo]");
}

#[test]
fn find_calls_flags_markup_hash_calls_vs_code_context() {
    // Prose `#link` is a markup call; a `link()` inside the task DSL (a
    // metadata slot's array) is a code-context call. Both parse as calls; only
    // the markup form is hash-prefixed.
    let src = "#show: task.with(depends-on: (link(\"q.typ\"),))\nbody #link(\"p.typ\")[L]";
    let calls = find_calls(src, "link");
    assert_eq!(calls.len(), 2);
    let code = calls
        .iter()
        .find(|c| c.positional(0).unwrap().value == "q.typ");
    let prose = calls
        .iter()
        .find(|c| c.positional(0).unwrap().value == "p.typ");
    assert!(!code.unwrap().markup);
    assert!(prose.unwrap().markup);
}

#[test]
fn find_calls_ignores_calls_in_raw_blocks() {
    let src = "```\n#git-ref(\"x\", \"y\")\n```";
    assert!(find_calls(src, "git-ref").is_empty());
}

#[test]
fn line_of_counts_newlines() {
    let src = "a\n#link(\"x.typ\")\nb";
    let target = find_link_targets(src).into_iter().next().unwrap();
    assert_eq!(line_of(src, target.range.start), 2);
    assert_eq!(line_of(src, 0), 1);
}
