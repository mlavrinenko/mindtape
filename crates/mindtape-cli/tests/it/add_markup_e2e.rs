//! End-to-end tests for how `mt add` handles slot values as Typst markup:
//! `text`-slot escaping, `raw`-slot splicing, and the pre-write refusal of a
//! raw filler that is not valid markup.
//!
//! Split out of `add_e2e.rs` to stay under the line budget. Fixtures come
//! from `support`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

/// Prose carrying every Typst markup leader must file a task that loads and
/// whose values read back exactly as typed. The `text` slots reach markup and
/// are escaped on the way in; the `string` title and `int` priority render as
/// literals and must not be, or the backslashes would become part of the value.
#[test]
fn markup_hazards_in_slot_values_round_trip_literally() {
    let dir = support::project_with_slot_template();
    let hazards = support::MARKUP_HAZARDS;

    let written = support::add_and_read(
        dir.path(),
        "hazards",
        &[
            "--set",
            &format!("title={hazards}"),
            "--set",
            &format!("why={hazards}"),
            "--set",
            &format!("summary={hazards}"),
            "--set",
            "weight=7",
        ],
    );
    // The two literal kinds keep the caller's bytes: an unquoted integer, and a
    // string literal with no escaping beyond what a Typst string already needs.
    assert!(written.contains("priority: 7,"), "{written}");
    assert!(
        written.contains(&format!("title: \"{hazards}\"")),
        "a string slot must not be markup-escaped: {written}"
    );
    // The markup kinds are escaped at every leader.
    assert!(
        written.contains(r"create-\*star\* \_u\_ \$x\$ \#h \[b\] \<a> >g \@at"),
        "text slots must be escaped: {written}"
    );

    // Reading back is the real assertion: unescaped, `*star*` loses its stars
    // and `<a>`/`@at` evaluate to a label and a reference and disappear.
    let task = support::task_json(dir.path(), "hazards");
    assert_eq!(support::at(&task, "/title").as_str(), Some(hazards));
    assert_eq!(support::at(&task, "/priority").as_f64(), Some(7.0));
    assert_eq!(support::at(&task, "/links/0/note").as_str(), Some(hazards));
}

/// A `raw`-kind slot splices its value verbatim: a code span filled through
/// `--set` survives as a code span, not an escaped literal. Round trip: a
/// code span in, a code span out, no backslashes — the opt-out `text` slots
/// deferred (`add-a-typst-slot-kind-for-pre-formatted-markup.typ`).
#[test]
fn raw_slot_splices_markup_without_escaping() {
    let dir = support::project_with_raw_slot_template();

    let written = support::add_and_read(
        dir.path(),
        "raw-body",
        &[
            "--set",
            "title=Raw body",
            "--set",
            "summary=See `mt add` for details.",
        ],
    );
    assert!(
        written.contains("See `mt add` for details."),
        "the code span must survive verbatim: {written}"
    );
    assert!(
        !written.contains(r"\`"),
        "a raw slot must not be markup-escaped: {written}"
    );

    // Reading back proves it landed as live markup, not inert text: `mt eval`
    // sees the same code span in the harvested body.
    support::run_ok(dir.path(), &["check"]);
}

/// A `raw` slot's filler is trusted to be markup, not guaranteed to be
/// *valid* markup: a value that fails to parse must fail `mt add` itself,
/// before anything is written, with an error naming the slot rather than a
/// bare compiler diagnostic about a file the caller never hand-wrote.
#[test]
fn raw_slot_unparseable_markup_fails_naming_the_slot_before_writing() {
    let dir = support::project_with_raw_slot_template();

    let output = support::mt(dir.path())
        .args(["add", "--name", "broken-raw", "--set", "title=Broken"])
        .args(["--set", "summary=#unclosed["])
        .assert()
        .failure()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("'summary'"),
        "the error must name the slot: {stderr}"
    );
    assert!(
        !dir.path().join("tasks/broken-raw.typ").exists(),
        "a raw slot's markup is checked before anything is written"
    );
}

/// `--describe=json` must report `raw` as the kind so a caller can tell
/// which slots accept raw markup before building a payload.
#[test]
fn describe_reports_raw_kind() {
    let dir = support::project_with_raw_slot_template();
    assert_eq!(support::describe_slot_kind(dir.path(), "summary"), "raw");
}
