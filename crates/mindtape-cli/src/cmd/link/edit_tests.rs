//! Unit tests for the pure edge rendering and `links:` reconciliation.
//!
//! Targets arrive here already settled (`cmd::link::plan::settle_targets` decides
//! whether a kind's target carries `.typ`), so every spec below is written the
//! way the planner hands it over — except where a test is about the dedupe
//! itself, which compares filename stems and so must see both spellings.

use mindtape::link::Link;

use super::{EdgeSpec, Mode, links_edit, quote, render_edges};

const NO_LINKS: &str = mindtape::typst_task!(
    r#"(
  title: "Do thing",
  status: proposed(2026, 6, 1),
)

== Body
"#
);

const WITH_LINKS: &str = mindtape::typst_task!(
    r#"(
  title: "Do thing",
  status: proposed(2026, 6, 1),
  links: depends-on("alpha.typ"),
)

== Body
"#
);

/// A file whose only edge is a SWHID tombstone — what `mt rm` leaves behind
/// when the task it pointed at was retired.
const WITH_TOMBSTONE: &str = mindtape::typst_task!(
    r#"(
  title: "Do thing",
  links: supersedes("swh:1:cnt:0000000000000000000000000000000000000000"),
)
"#
);

fn edge(kind: &str, target: &str, note: Option<&str>) -> EdgeSpec {
    EdgeSpec {
        kind: kind.to_owned(),
        target: target.to_owned(),
        note: note.map(ToOwned::to_owned),
    }
}

fn link(kind: &str, target: &str, note: Option<&str>) -> Link {
    Link {
        task: target.to_owned(),
        kind: kind.to_owned(),
        note: note.map(ToOwned::to_owned),
    }
}

/// Apply the computed edit and hand back the rewritten source; `None` when the
/// reconciliation decided there was nothing to write.
fn applied(source: &str, existing: &[Link], desired: &[EdgeSpec], mode: Mode) -> Option<String> {
    let edit = links_edit(source, existing, desired, mode).expect("edit must build")?;
    Some(typst_edit::apply(source, vec![edit.edit]).expect("splice must apply"))
}

#[test]
fn builtin_kind_renders_its_named_helper() {
    let rendered = render_edges(&[edge("depends-on", "alpha.typ", None)]);
    assert_eq!(rendered, r#"depends-on("alpha.typ")"#);
}

#[test]
fn custom_kind_renders_the_generic_edge_helper() {
    // The prelude binds a helper per BUILT-IN kind and nothing per custom one,
    // so a bare `discharged-by(...)` would be an unknown identifier. The target
    // arrives settled, so a kind pointing outside the backlog keeps its URL.
    let rendered = render_edges(&[edge("discharged-by", "https://x/pull/42", None)]);
    assert_eq!(rendered, r#"edge("discharged-by", "https://x/pull/42")"#);
}

#[test]
fn consecutive_noteless_edges_of_one_kind_collapse_into_one_call() {
    let rendered = render_edges(&[
        edge("depends-on", "alpha.typ", None),
        edge("depends-on", "beta.typ", None),
    ]);
    assert_eq!(rendered, r#"depends-on("alpha.typ", "beta.typ")"#);
}

#[test]
fn a_noted_edge_never_joins_a_run() {
    // The trailing block is the note for EVERY target in its call, so folding
    // a noted edge into a run would attach its note to the others.
    let rendered = render_edges(&[
        edge("related", "alpha.typ", Some("why")),
        edge("related", "beta.typ", Some("other")),
    ]);
    assert_eq!(
        rendered,
        r#"related("alpha.typ")[why] + related("beta.typ")[other]"#
    );
}

#[test]
fn a_note_is_markup_escaped() {
    let rendered = render_edges(&[edge("related", "alpha.typ", Some("see *this* #now"))]);
    assert_eq!(rendered, r#"related("alpha.typ")[see \*this\* \#now]"#);
}

#[test]
fn merge_inserts_links_when_the_task_has_none() {
    let desired = [edge("depends-on", "alpha.typ", None)];
    let out = applied(NO_LINKS, &[], &desired, Mode::Merge).expect("a first edge must be an edit");
    assert!(out.contains(r#"links: depends-on("alpha.typ")"#), "{out}");
}

#[test]
fn merge_appends_beside_the_edges_already_there() {
    let existing = [link("depends-on", "alpha.typ", None)];
    let desired = [edge("related", "beta.typ", Some("why"))];
    let out =
        applied(WITH_LINKS, &existing, &desired, Mode::Merge).expect("a new edge must be an edit");
    assert!(
        out.contains(r#"links: depends-on("alpha.typ") + related("beta.typ")[why]"#),
        "{out}"
    );
}

#[test]
fn merge_dedupes_on_the_target_stem_so_a_rerun_is_a_no_op() {
    let existing = [link("depends-on", "alpha.typ", None)];
    // The dedupe compares filename stems, so an existing `alpha.typ` and a
    // requested bare `alpha` are the same edge, not a second one.
    let desired = [edge("depends-on", "alpha", None)];
    let edit = links_edit(WITH_LINKS, &existing, &desired, Mode::Merge).expect("must not fail");
    assert!(edit.is_none(), "an already-present edge must write nothing");
}

#[test]
fn merge_keeps_the_new_edges_only() {
    let existing = [link("depends-on", "alpha.typ", None)];
    let desired = [
        edge("depends-on", "alpha.typ", None),
        edge("depends-on", "beta.typ", None),
    ];
    let edit = links_edit(WITH_LINKS, &existing, &desired, Mode::Merge)
        .expect("must not fail")
        .expect("one edge is new");
    assert_eq!(edit.added, vec![edge("depends-on", "beta.typ", None)]);
    assert_eq!(edit.removed, 0);
}

#[test]
fn replace_makes_the_given_set_the_whole_value() {
    let existing = [link("depends-on", "alpha.typ", None)];
    let desired = [edge("parent", "umbrella.typ", None)];
    let out = applied(WITH_LINKS, &existing, &desired, Mode::Replace)
        .expect("a different set must be an edit");
    assert!(out.contains(r#"links: parent("umbrella.typ")"#), "{out}");
    assert!(!out.contains("depends-on"), "{out}");
}

#[test]
fn replace_reports_what_it_drops() {
    let existing = [
        link("depends-on", "alpha.typ", None),
        link("depends-on", "beta.typ", None),
    ];
    let desired = [edge("depends-on", "alpha.typ", None)];
    let edit = links_edit(WITH_LINKS, &existing, &desired, Mode::Replace)
        .expect("must not fail")
        .expect("dropping an edge is a change");
    assert_eq!(edit.removed, 1);
    assert!(edit.added.is_empty());
}

#[test]
fn replace_with_the_same_set_is_a_no_op() {
    let existing = [link("depends-on", "alpha.typ", None)];
    let desired = [edge("depends-on", "alpha.typ", None)];
    let edit = links_edit(WITH_LINKS, &existing, &desired, Mode::Replace).expect("must not fail");
    assert!(edit.is_none(), "an identical set must write nothing");
}

#[test]
fn replace_rewrites_when_only_a_note_changed() {
    // The key sets match, so nothing but the note comparison can catch this.
    let existing = [link("related", "alpha.typ", Some("old reason"))];
    let desired = [edge("related", "alpha.typ", Some("new reason"))];
    let out = applied(WITH_LINKS, &existing, &desired, Mode::Replace)
        .expect("a reworded note is a change");
    assert!(out.contains("[new reason]"), "{out}");
}

#[test]
fn replace_with_no_edges_writes_the_empty_array() {
    let existing = [link("depends-on", "alpha.typ", None)];
    let out =
        applied(WITH_LINKS, &existing, &[], Mode::Replace).expect("clearing the set is a change");
    assert!(out.contains("links: ()"), "{out}");
}

#[test]
fn replace_refuses_to_drop_a_history_pointer() {
    let desired = [edge("parent", "alpha.typ", None)];
    let err = links_edit(WITH_TOMBSTONE, &[], &desired, Mode::Replace)
        .expect_err("a tombstone must refuse the rewrite")
        .to_string();
    assert!(err.contains("history pointer"), "{err}");
    assert!(err.contains("swh:1:cnt:"), "{err}");
}

#[test]
fn merge_still_works_on_a_file_carrying_a_history_pointer() {
    let desired = [edge("parent", "alpha.typ", None)];
    let out = applied(WITH_TOMBSTONE, &[], &desired, Mode::Merge).expect("merge must not refuse");
    assert!(out.contains("swh:1:cnt:"), "the pointer survives: {out}");
    assert!(out.contains(r#"parent("alpha.typ")"#), "{out}");
}

#[test]
fn a_source_with_no_task_call_is_an_error() {
    let desired = [edge("parent", "a.typ", None)];
    assert!(links_edit("not a task file", &[], &desired, Mode::Merge).is_err());
}

// --- quote ---
//
// `\` and `"` are the only two characters that can end a Typst string literal
// early, and each is escaped independently. The first two assertions take the
// two sides of `ch == '\\' || ch == '"'`; the third pins the untouched ordinary
// character, so a `||` -> `&&` mutant (which escapes neither) and a
// blanket-escaper both fail.

#[test]
fn quote_escapes_a_backslash() {
    assert_eq!(quote(r"a\b"), r#""a\\b""#);
}

#[test]
fn quote_escapes_a_quote() {
    assert_eq!(quote("a\"b"), r#""a\"b""#);
}

#[test]
fn quote_leaves_an_ordinary_stem_verbatim() {
    assert_eq!(quote("ship-it"), r#""ship-it""#);
}

// --- render_edges ---
//
// Consecutive note-less edges of one kind collapse; a run is flushed the moment
// the kind changes or the previous edge carried a note. Both guards in the
// `extends` chain are exercised here: a note-less edge of a *different* kind
// must not fold into the previous run (the outer `&&`), and a note-less edge
// whose predecessor carries a note must not either (the inner `&&`).

#[test]
fn a_note_less_edge_of_another_kind_opens_its_own_call() {
    let edges = [
        edge("depends-on", "a.typ", None),
        edge("related", "b.typ", None),
    ];
    assert_eq!(
        render_edges(&edges),
        r#"depends-on("a.typ") + related("b.typ")"#
    );
}

#[test]
fn a_note_less_edge_after_a_noted_one_opens_its_own_call() {
    let edges = [
        edge("depends-on", "a.typ", Some("why")),
        edge("depends-on", "b.typ", None),
    ];
    assert_eq!(
        render_edges(&edges),
        r#"depends-on("a.typ")[why] + depends-on("b.typ")"#
    );
}
