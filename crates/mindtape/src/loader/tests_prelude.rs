//! The real, in-tree prelude (not `loader/tests.rs`'s `PRELUDE_STUB`) driven
//! end to end: what a task file authors against `@local/mindtape` must arrive
//! as a [`crate::model::Task`], through the actual Typst eval and harvester
//! rather than hand-built `HVal` fixtures. Shares `loader/tests.rs`'s
//! fixtures (`setup`, `write`, `write_pkg`).
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests::{setup, write_pkg};
use super::{LoadOutcome, load_project};
use crate::config::Config;
use crate::model::Task;
use crate::model::numeric::NumValue;
use crate::recur::{PeriodUnit, Rule};
use crate::test_support::write_file;
use typst::WorldExt;
use typst::syntax::VirtualRoot;
use typst_harvest::HarvestWorld;
use typst_layout::PagedDocument;

use crate::test_support::PRELUDE;

/// Load one file authored against the real prelude, under a default config.
/// The `TempDir` rides along in the return so the caller keeps it alive.
fn load_real_prelude(source: &str) -> (tempfile::TempDir, LoadOutcome) {
    let dir = setup();
    write_pkg(dir.path(), "lib", "mindtape", "0.2.0", PRELUDE);
    write_file(dir.path(), "tasks/t.typ", source);
    let outcome = load_project(dir.path(), &Config::default());
    (dir, outcome)
}

/// A world compiling `tasks/t.typ` under `dir` against the real prelude.
fn prelude_world(dir: &std::path::Path) -> HarvestWorld {
    HarvestWorld::with_root(&dir.join("tasks/t.typ"), dir)
        .unwrap()
        .with_local_package("mindtape", dir.join("lib"))
}

/// Load one task authored with a single `task.with` field and return the first
/// load-error message. Used by the assertion tests, which all expect the file
/// to fail eval and produce no task at all.
fn prelude_eval_error(field: &str) -> String {
    let source = format!(
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"T\", {field})\n"
    );
    let (_dir, outcome) = load_real_prelude(&source);
    assert!(
        outcome.tasks.is_empty(),
        "expected no task from `{field}`, got {:?}",
        outcome.tasks
    );
    assert!(!outcome.errors.is_empty(), "`{field}` loaded clean");
    outcome.errors[0].message.clone()
}

/// Load one task carrying `links` as its whole `links:` field value, assert it
/// loaded clean and shape-error-free, and return it. The spellings below are
/// meant to be interchangeable, so what separates them has to be the source
/// text and nothing else — one loader and one set of assertions is what makes
/// that claim, rather than three tests that merely happen to agree.
fn task_with_links(links: &str) -> Task {
    let source = format!(
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"T\", links: {links})\n"
    );
    let (_dir, outcome) = load_real_prelude(&source);
    assert_eq!(outcome.errors, Vec::new(), "`{links}` failed to load");
    let task = outcome.tasks.first().unwrap().clone();
    assert!(
        task.shape_errors.is_empty(),
        "`{links}` is a spelling, not a shape violation: {:?}",
        task.shape_errors
    );
    task
}

/// The edges every spelling must land on: two `depends-on` targets in authored
/// order, and one `related` carrying its note.
fn assert_canonical_edges(task: &Task) {
    let deps: Vec<&str> = task.link_targets("depends-on").collect();
    assert_eq!(deps, vec!["a.typ", "b.typ"]);

    let related: Vec<_> = task.links.iter().filter(|l| l.kind == "related").collect();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].task, "c.typ");
    assert_eq!(related[0].note.as_deref(), Some("why"));
}

/// The `links:` DSL — variadic helpers, the trailing-note idiom, and combining
/// several kinds with `+`.
#[test]
fn real_prelude_parses_typed_links_end_to_end() {
    let task = task_with_links("depends-on(\"a.typ\", \"b.typ\") + related(\"c.typ\")[why]");
    assert_canonical_edges(&task);
}

/// The array spelling of the same thing. Every helper returns an array, so
/// writing `links:` as the tuple every other multi-valued field takes nests one
/// level — and used to reach the model as zero edges, validating clean while the
/// dependency graph sat empty. A task that loads with no links here is the
/// regression.
#[test]
fn real_prelude_reads_array_form_links_as_the_same_edges_as_concatenation() {
    let task = task_with_links("(depends-on(\"a.typ\", \"b.typ\"), related(\"c.typ\")[why])");
    assert_canonical_edges(&task);
}

/// The partial-loss shape: one helper concatenated with a tuple holding another.
/// This kept exactly one of the two edges, which is worse than losing both —
/// `mt ls blocked` still lists the task, just against half its blockers.
#[test]
fn real_prelude_reads_links_mixing_concatenation_and_the_array_form() {
    let task = task_with_links(
        "depends-on(\"a.typ\") + (depends-on(\"b.typ\"), (related(\"c.typ\")[why],))",
    );
    assert_canonical_edges(&task);
}

/// Flattening must not swallow the field-level shape error the prelude
/// deliberately leaves to the Rust layer: a non-array `links:` passes through
/// untouched, so it still reports against the expected shape rather than failing
/// eval on a method a string does not have.
#[test]
fn real_prelude_leaves_a_non_array_links_to_the_rust_shape_error() {
    let (_dir, outcome) =
        load_real_prelude(crate::typst_task!("(title: \"T\", links: \"a.typ\")\n"));

    assert_eq!(outcome.errors, Vec::new());
    let task = outcome.tasks.first().unwrap();
    assert!(task.links.is_empty());
    assert_eq!(task.shape_errors.len(), 1);
    assert_eq!(task.shape_errors[0].field, "links");
    assert_eq!(
        task.shape_errors[0].message,
        "expected an array, found a string"
    );
}

/// Every core field is a declared named parameter of `task` — that is the point
/// of the explicit signature — so each must still reach the marker under its own
/// name. A parameter the signature declares but the marker's `fields` dict
/// forgets would silently drop that field for every task in every project, with
/// nothing failing; this authors all thirteen at once and reads them back.
#[test]
fn real_prelude_carries_every_core_field_to_the_marker() {
    let (_dir, outcome) = load_real_prelude(crate::typst_task!(
        "(\n  \
           title: \"T\",\n  \
           status: wip(2026, 6, 14)[note],\n  \
           priority: 8,\n  \
           difficulty: 3,\n  \
           kind: \"bug\",\n  \
           estimate: (days: 2),\n  \
           links: depends-on(\"a.typ\"),\n  \
           tags: (\"mt\",),\n  \
           due: on(2026, 7, 1),\n  \
           start: on(2026, 6, 20),\n  \
           reviewed: on(2026, 6, 25),\n  \
           recur: every(weeks: 1),\n  \
           id: \"explicit\",\n\
         )\n"
    ));

    assert_eq!(outcome.errors, Vec::new());
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.title, "T");
    let status = task.status.as_ref().unwrap();
    assert_eq!(status.name, "wip");
    assert_eq!(status.date.as_deref(), Some("2026-06-14"));
    assert_eq!(status.note.as_deref(), Some("note"));
    assert_eq!(task.priority, Some(NumValue::Num(8.0)));
    assert_eq!(task.difficulty, Some(NumValue::Num(3.0)));
    assert_eq!(task.kind.as_deref(), Some("bug"));
    assert_eq!(task.estimate.get("days"), Some(&2.0));
    assert_eq!(
        task.link_targets("depends-on").collect::<Vec<_>>(),
        ["a.typ"]
    );
    assert_eq!(task.tags, ["mt"]);
    assert_eq!(task.due.as_deref(), Some("2026-07-01"));
    assert_eq!(task.start.as_deref(), Some("2026-06-20"));
    assert_eq!(task.reviewed.as_deref(), Some("2026-06-25"));
    assert!(matches!(
        task.recur.as_ref().unwrap().rule,
        Rule::Period {
            unit: PeriodUnit::Weeks,
            value: 1,
            ..
        }
    ));
    assert_eq!(task.id, "explicit");
}

/// The extension discipline, enforced over `..meta` alone: an `x-<ns>` slot is
/// collected, and a misspelled core field — which the named parameters do not
/// absorb, since none is named `titel` — fails the file with Typst's own
/// refusal of an argument the callee does not declare.
#[test]
fn real_prelude_collects_extensions_and_rejects_unknown_slots() {
    let (_dir, ok) = load_real_prelude(crate::typst_task!(
        "(title: \"T\", x-speconaut: (spec: \"s.typ\"))\n"
    ));
    assert_eq!(ok.errors, Vec::new());
    assert!(ok.tasks.first().unwrap().extras.contains_key("x-speconaut"));

    let (_dir, bad) = load_real_prelude(crate::typst_task!("(title: \"T\", titel: \"typo\")\n"));
    assert!(bad.tasks.is_empty());
    assert_eq!(bad.errors[0].message, "unexpected argument: titel");
}

/// Where an editor draws the misspelled-field error: on the misspelled name in
/// the task file. An error raised inside the prelude would carry the prelude's
/// span, and tinymist would publish it for the package file instead.
#[test]
fn real_prelude_places_the_unknown_slot_error_on_the_name_in_the_task_file() {
    let source =
        crate::typst_task!("(\n  titel: \"typo\",\n  x-speconaut: (spec: \"s.typ\"),\n)\n");
    let dir = setup();
    write_pkg(dir.path(), "lib", "mindtape", "0.2.0", PRELUDE);
    write_file(dir.path(), "tasks/t.typ", source);
    let world = prelude_world(dir.path());
    let world: &dyn typst::World = &world;

    let errors = typst::compile::<PagedDocument>(world).output.unwrap_err();
    let error = &errors[0];
    assert_eq!(error.message, "unexpected argument: titel");
    let file = error.span.id().unwrap();
    assert_eq!(file.vpath().get_with_slash(), "/tasks/t.typ");
    assert!(matches!(file.root(), VirtualRoot::Project));
    let range = world.range(error.span).unwrap();
    assert!(source[range.start..].starts_with("titel"), "{range:?}");
}

/// Every call the sink took before it refused through Typst still loads: no
/// extension slot, several, one whose name Typst parses as `x-` alone, and one
/// a dictionary spread names with a space, which no parameter can spell.
#[test]
fn real_prelude_still_takes_every_extension_slot_it_took() {
    let cases: [(&str, &[&str]); 4] = [
        ("", &[]),
        ("x-a: 1, x-b-2: 2", &["x-a", "x-b-2"]),
        ("x-: 1", &["x-"]),
        ("..(\"x-a b\": 1), x-c: 2", &["x-a b", "x-c"]),
    ];
    for (fields, keys) in cases {
        let (_dir, outcome) = load_real_prelude(&format!(
            "#import \"@local/mindtape:0.2.0\": *\n\
             #show: task.with(title: \"T\", {fields})\n"
        ));
        assert_eq!(outcome.errors, Vec::new(), "`{fields}` failed to load");
        let extras: Vec<&str> = outcome.tasks[0].extras.keys().map(String::as_str).collect();
        assert_eq!(extras, keys, "`{fields}`");
    }
}

/// The refusals around the extension sink keep their text: a misspelled field
/// beside a slot no parameter can spell, and a positional argument.
#[test]
fn real_prelude_still_refuses_what_the_sink_refused() {
    let cases = [
        ("..(\"x-a b\": 1), titel: 2", "unexpected argument: titel"),
        (
            "..(\"x-a b\": 1, \"ti tel\": 2)",
            "unexpected argument: ti tel",
        ),
    ];
    for (field, message) in cases {
        assert_eq!(prelude_eval_error(field), message.to_owned());
    }
    let (_dir, outcome) = load_real_prelude(crate::typst_task!("(\"stray\", title: \"T\")\n"));
    assert_eq!(
        outcome.errors[0].message,
        "assertion failed: task takes no positional args besides its body"
    );
}

/// The regression this file exists for: `link()` returns content, content is
/// the note, so `depends-on(link("a.typ"))` used to yield an edge array with no
/// edge in it. Nothing downstream could see the loss — the marker carried an
/// empty `links`, `mt check` passed, and the task reported ready. The helper
/// must name itself and say why the target vanished.
#[test]
fn real_prelude_rejects_a_link_target_in_an_edge_helper() {
    let message = prelude_eval_error("links: depends-on(link(\"a.typ\"))");
    assert!(
        message.contains("`depends-on()` needs at least one target")
            && message.contains("`link(...)` is content"),
        "unexpected error: {message}"
    );
}

/// Every other argument shape the helpers would otherwise drop on the floor.
/// Each is an authoring mistake whose value never reaches the marker, so the
/// Rust layer has nothing to validate — the prelude is the only layer that can
/// still see it. The error names the offending helper in each case.
#[test]
fn real_prelude_rejects_argument_shapes_that_would_be_dropped() {
    let cases = [
        (
            "links: depends-on()",
            "`depends-on()` needs at least one target",
        ),
        (
            "links: related(\"a.typ\")[one][two]",
            "`related()` takes at most one note block",
        ),
        (
            "links: depends-on(42)",
            "`depends-on()` target must be a plain stem string",
        ),
        (
            "links: parent(none)",
            "`parent()` target must be a plain stem string",
        ),
        (
            "links: depends-on(\"a.typ\", note: [n])",
            "`depends-on()` takes no named arguments",
        ),
        (
            "links: edge(7, \"a.typ\")",
            "edge() kind must be a non-empty string",
        ),
        (
            "status: wip(2026, 6, 14)[one][two]",
            "`wip()` takes at most one note block",
        ),
        (
            "status: wip(2026, 6, 14, \"note\")",
            "`wip()` note must be a content block",
        ),
        (
            "status: wip(2026, 6, 14, note: [n])",
            "`wip()` takes no named arguments",
        ),
        (
            "priority: framework(\"ice\", 8)",
            "`ice()` inputs must be named",
        ),
    ];
    for (field, needle) in cases {
        let message = prelude_eval_error(field);
        assert!(
            message.contains(needle),
            "`{field}`: expected {needle:?}, got {message}"
        );
    }
}

/// The other half of the same seam: a wrong-typed value INSIDE a well-shaped
/// field must survive eval and arrive at the Rust layer as a shape error
/// naming its position.
///
/// Each of these used to fail eval instead, by accident rather than by design
/// — the card render reached for a method the value did not have (`raw(tag)`,
/// `status.at("name")`, `on.join(", ")`). That produced a message naming
/// neither the field nor the task, and would have vanished the moment the
/// render stopped making that call. The render owes nobody validation, so it
/// is total now and this is where the check lives.
#[test]
fn real_prelude_carries_a_wrong_typed_element_to_the_rust_shape_error() {
    let cases = [
        (
            "tags: (\"x\", 7)",
            "tags[1]",
            "expected a string, found a number",
        ),
        (
            "status: \"wip\"",
            "status",
            "expected a status like `wip(2026, 6, 14)`, found a string",
        ),
        (
            "recur: every(weeks: 2, on: (\"mon\", 7))",
            "recur.on[1]",
            "expected a weekday name (mon, tue, wed, thu, fri, sat, sun), found a number",
        ),
        (
            "estimate: (days: \"two\")",
            "estimate.days",
            "expected a number or count(...), found a string",
        ),
    ];
    for (field, path, message) in cases {
        let source = format!(
            "#import \"@local/mindtape:0.2.0\": *\n\
             #show: task.with(title: \"T\", {field})\n"
        );
        let (_dir, outcome) = load_real_prelude(&source);

        assert_eq!(outcome.errors, Vec::new(), "`{field}` failed to load");
        let task = outcome.tasks.first().unwrap();
        assert_eq!(
            task.shape_errors.len(),
            1,
            "`{field}`: {:?}",
            task.shape_errors
        );
        assert_eq!(task.shape_errors[0].field, path);
        assert_eq!(task.shape_errors[0].message, message);
    }
}

/// A template previews as a task: `slot` evaluates to its `default` and takes
/// an `example` it ignores.
#[test]
fn a_slot_evaluates_to_its_default_and_ignores_its_example() {
    let (_dir, outcome) = load_real_prelude(crate::typst_task!(
        "(title: slot(\"title\", default: \"T\", example: \"E\"))\n"
    ));
    assert_eq!(outcome.errors, Vec::new());
    assert_eq!(outcome.tasks.first().unwrap().title, "T");
}

/// `count(...)`, `grep(...)` and `glob(...)` reach the marker as the dicts the
/// Rust layer reads, a raw-block regex as its text, and the printed card lays
/// the composite out without an error.
#[test]
fn real_prelude_carries_a_count_estimate_and_renders_it() {
    let source = crate::typst_task!(
        "(title: \"T\", estimate: (src-files: count(2, grep(`fn \\w+`, \"src/**\"), glob(\"a.rs\"))))\n"
    );
    let (dir, outcome) = load_real_prelude(source);
    assert_eq!(outcome.errors, Vec::new());
    let task = outcome.tasks.first().unwrap();
    assert!(task.shape_errors.is_empty(), "{:?}", task.shape_errors);
    let count = task.estimate_counts.get("src-files").unwrap();
    assert_eq!(count.manual, 2.0);
    assert_eq!(
        count.queries,
        vec![
            crate::estimate_queries::Query::Grep {
                regex: "fn \\w+".to_owned(),
                glob: Some("src/**".to_owned()),
            },
            crate::estimate_queries::Query::Glob {
                pattern: "a.rs".to_owned(),
            },
        ]
    );
    let world = prelude_world(dir.path());
    let world: &dyn typst::World = &world;
    assert!(typst::compile::<PagedDocument>(world).output.is_ok());
}

/// `grep(...)` takes one optional glob, so a second one or a named argument
/// fails the file rather than narrowing nothing.
#[test]
fn real_prelude_refuses_a_grep_with_more_than_one_glob() {
    for call in [
        "grep(`a`, \"x/**\", \"y/**\")",
        "grep(`a`, within: \"x/**\")",
    ] {
        let field = format!("estimate: (files: count(1, {call}))");
        let message = prelude_eval_error(&field);
        assert!(
            message.contains("grep() takes a regex and then at most one glob"),
            "{call}: {message}"
        );
    }
}
