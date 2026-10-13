#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{TargetSpan, link_target_spans};

/// The raw targets a scan finds, in scan order (`link()` first, then helpers).
fn raws(source: &str) -> Vec<String> {
    link_target_spans(source)
        .into_iter()
        .map(|span| span.raw)
        .collect()
}

/// Wrap `links` into a minimal task marker so the scan runs on realistic source.
fn task(links: &str) -> String {
    format!("#import \"@local/mindtape:0.2.0\": *\n#show: task.with(\n  links: {links},\n)\n")
}

#[test]
fn finds_depends_on_targets() {
    let got = raws(&task("depends-on(\"a.typ\", \"b.typ\")"));
    assert_eq!(got, vec!["a.typ".to_owned(), "b.typ".to_owned()]);
}

#[test]
fn finds_targets_across_combined_kinds() {
    let got = raws(&task("depends-on(\"a.typ\") + related(\"b.typ\")[why]"));
    assert_eq!(got, vec!["a.typ".to_owned(), "b.typ".to_owned()]);
}

#[test]
fn edge_kind_argument_is_not_a_target() {
    // `edge("blocks", "a.typ")`: only the second positional is a target — the
    // leading kind string must never be reported, or retiring a task named
    // after a kind would corrupt the edge.
    let got = raws(&task("edge(\"blocks\", \"a.typ\")"));
    assert_eq!(got, vec!["a.typ".to_owned()]);
}

#[test]
fn note_content_block_is_not_a_target() {
    // The `[why]` note is a content block, not a string arg — never scanned.
    let spans = link_target_spans(&task("related(\"a.typ\")[why not b]"));
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].raw, "a.typ");
}

#[test]
fn still_finds_legacy_link_calls() {
    // A bare `link("…")` (body cross-reference or a legacy edge form) is still
    // reported, so retiring a task tombstones those too.
    let got = raws("see #link(\"a.typ\")[here]");
    assert_eq!(got, vec!["a.typ".to_owned()]);
}

#[test]
fn span_covers_the_quoted_literal() {
    let source = task("depends-on(\"a.typ\")");
    let spans = link_target_spans(&source);
    assert_eq!(spans.len(), 1);
    let TargetSpan { range, .. } = &spans[0];
    // The span is the quoted literal, quotes included — ready to splice.
    assert_eq!(&source[range.clone()], "\"a.typ\"");
}

#[test]
fn no_links_yields_no_spans() {
    let source = crate::typst_task!("#show: task.with(title: \"x\")\n");
    assert!(link_target_spans(source).is_empty());
}
