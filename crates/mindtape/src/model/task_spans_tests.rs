use super::{CallRegion, TaskCallSpans};

const SRC: &str = crate::typst_task!(
    "(\n  title: \"T\",\n  \
                   links: depends-on(\"a.typ\"),\n)\n\n== Body\n"
);

#[test]
fn names_the_call_and_each_argument_line() {
    let spans = TaskCallSpans::parse(SRC);
    assert_eq!(spans.call_line(), Some(3));
    assert_eq!(spans.field_line("title"), Some(4));
    assert_eq!(spans.field_line("links"), Some(5));
    assert_eq!(spans.body(SRC), "== Body");
}

/// The `links` value, the call's closing `)`, the body, the import line, and a
/// line past the end; then a source with no call.
#[test]
fn a_point_falls_in_an_argument_in_the_call_or_outside_it() {
    let spans = TaskCallSpans::parse(SRC);
    let regions: Vec<CallRegion> = [(5, 10), (6, 1), (8, 1), (1, 1), (99, 1)]
        .into_iter()
        .map(|(line, column)| spans.region_at(SRC, line, column))
        .collect();
    let (links, outside) = (
        CallRegion::Argument("links".to_owned()),
        CallRegion::Outside,
    );
    let call = CallRegion::Call;
    assert_eq!(
        regions,
        [links, call, outside.clone(), outside.clone(), outside]
    );
    let none = TaskCallSpans::default().region_at(SRC, 5, 10);
    assert_eq!(none, CallRegion::Outside);
}

/// Typst counts a column in characters, so the two-byte letters in the title
/// move the byte offset of `tags` but not its column, 40.
#[test]
fn a_column_counts_characters_not_bytes() {
    let src = "#show: task.with(title: \"Größe\", tags: (\"a\",))\n";
    let spans = TaskCallSpans::parse(src);
    let tags = CallRegion::Argument("tags".to_owned());
    assert_eq!(spans.region_at(src, 1, 40), tags);
    assert_eq!(spans.region_at(src, 1, 39), CallRegion::Call);
}

#[test]
fn a_sub_path_or_absent_field_resolves_to_its_root_or_the_call() {
    let spans = TaskCallSpans::parse(SRC);
    assert_eq!(spans.field_line("links[1]"), Some(5));
    assert_eq!(spans.field_line("estimate.days"), Some(3));
}

#[test]
fn a_source_without_the_call_has_no_line_and_no_body() {
    let spans = TaskCallSpans::parse("no marker\n");
    assert_eq!(spans, TaskCallSpans::default());
    assert_eq!(spans.field_line("id"), None);
    assert_eq!(spans.body("no marker\n"), "");
}

#[test]
fn a_call_through_a_helper_is_located_by_its_name() {
    let src = "#show: helper.with(\n  title: \"T\",\n)\nbody\n";
    assert_eq!(TaskCallSpans::parse(src).call_line(), None);
    let spans = TaskCallSpans::parse_as(src, "helper");
    assert_eq!(spans.call_line(), Some(1));
    assert_eq!(spans.field_line("title"), Some(2));
    assert_eq!(spans.body(src), "body");
}

#[test]
fn relocate_prefers_the_field_then_the_same_text_then_the_call() {
    let authored = "#show: task.with(\n  title: \"T\",\n  tags: (\"a\",),\n)\n";
    let checked = "#show: task.with(\n  title: \"T\",\n  tags: (\"a\",),\n  priority: 5,\n)\n";
    let spans = TaskCallSpans::parse(authored);
    assert_eq!(spans.relocate(authored, checked, "tags", 9), Some(3));
    assert_eq!(spans.relocate(authored, checked, "other", 3), Some(3));
    assert_eq!(spans.relocate(authored, checked, "priority", 4), Some(1));
    assert_eq!(
        TaskCallSpans::default().relocate("", checked, "priority", 4),
        None
    );
}

#[test]
fn the_body_is_the_trimmed_prose_after_the_call() {
    let src = crate::typst_task!(
        "(\n  title: \"T\",\n)\n\n\
        == Summary\n\nFirst line.\n\nSecond line.\n"
    );
    let spans = TaskCallSpans::parse(src);
    assert_eq!(spans.call_line(), Some(3));
    assert_eq!(spans.body(src), "== Summary\n\nFirst line.\n\nSecond line.");
}

#[test]
fn the_call_line_counts_newlines_not_lines() {
    // The head before the call ends in a newline, so `lines().count()` would
    // report 2 for a call that in fact opens line 3.
    assert_eq!(
        TaskCallSpans::parse("a\nb\n#show: task.with()\n").call_line(),
        Some(3)
    );
    assert_eq!(
        TaskCallSpans::parse("#show: task.with()\n").call_line(),
        Some(1)
    );
    assert_eq!(
        TaskCallSpans::parse("#show: task.with(title: \"T\")\n").body(""),
        ""
    );
}
