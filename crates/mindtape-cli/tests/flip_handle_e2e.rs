//! End-to-end: what `mt flip` says about the handles it just invalidated.
//!
//! The unit half (`src/flip_handles_tests.rs`) pins which line a task earns
//! from its two handles. This half pins the two things only a real binary can
//! show: that the handle `flip` prints is the one the caller's next `mt ls`
//! actually reports, and that a multi-target batch diffs against ONE pre-batch
//! view rather than recomputing per target.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

/// Bare roots — the config floor supplies the legend (`proposed` < `accepted`
/// < `wip` < terminal `done`) and the default view's `!terminal` filter, which
/// are exactly the two things a flip moves a task through.
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// One task at priority 5. Every fixture task shares the priority so the
/// default sort's ready/priority arms tie and its `status_index` arm decides
/// the order — which is what makes a flip move an ordinal at all.
fn task_source(title: &str, status: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  \
         status: {status}(2026, 8, 1),\n  priority: 5,\n)\n\n== Summary\n\nBody.\n"
    )
}

/// Three `proposed` tasks and one `accepted`, so `mt ls` lists them
/// alpha/bravo/charlie/delta and flipping any of the first three to `wip`
/// sends it past delta.
fn project() -> tempfile::TempDir {
    let dir = support::scaffold_project(CONFIG);
    for (stem, status) in [
        ("alpha", "proposed"),
        ("bravo", "proposed"),
        ("charlie", "proposed"),
        ("delta", "accepted"),
    ] {
        let title = format!("Task {stem}");
        std::fs::write(
            dir.path().join("tasks").join(format!("{stem}.typ")),
            task_source(&title, status),
        )
        .unwrap();
    }
    dir
}

/// stdout of a successful `mt <args…>` run in `dir`.
fn out(dir: &Path, args: &[&str]) -> String {
    let bytes = support::mt(dir)
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(bytes).unwrap()
}

/// The `#` column `mt ls` prints for the row naming `title` — the handle the
/// caller reads off the list, sourced from the renderer rather than recomputed
/// here, so a test can only pass by agreeing with what a human would see.
fn handle_of(dir: &Path, title: &str) -> String {
    let listing = out(dir, &["ls"]);
    listing
        .lines()
        .find(|line| line.contains(title))
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or_else(|| panic!("no {title:?} row in:\n{listing}"))
        .to_owned()
}

/// The load-bearing property: the handle `flip` prints is the handle the next
/// command will accept. Asserting only that *some* line appeared would pass on
/// a stale or invented ordinal, so this reads the answer back off `mt ls`.
#[test]
fn the_reported_handle_is_the_one_the_next_ls_shows() {
    let dir = project();
    let before = handle_of(dir.path(), "Task alpha");

    let printed = out(dir.path(), &["wip", &before]);

    let after = handle_of(dir.path(), "Task alpha");
    assert_ne!(before, after, "fixture must actually move the ordinal");
    assert!(
        printed.contains(&format!("handle alpha: {before} → {after}")),
        "expected the refreshed handle beside the transition: {printed}"
    );
}

/// A same-status re-stamp reorders nothing, and the report stays silent —
/// the common case, where a line saying "unchanged" would teach the reader to
/// skip the line that says something changed.
#[test]
fn a_re_stamp_that_moves_nothing_says_nothing() {
    let dir = project();
    let handle = handle_of(dir.path(), "Task alpha");

    let printed = out(dir.path(), &["proposed", &handle]);

    assert!(
        printed.contains("re-stamped"),
        "fixture must be a same-status re-stamp: {printed}"
    );
    assert!(
        !printed.contains("handle "),
        "an unmoved handle must print nothing extra: {printed}"
    );
}

/// `mt done` filters the task out of the floor's `!terminal` default view, so
/// there is no ordinal left to hand back. The line names the filename stem
/// instead, which is what still addresses the task.
#[test]
fn leaving_the_default_view_names_the_stem() {
    let dir = project();
    let handle = handle_of(dir.path(), "Task alpha");

    let printed = out(dir.path(), &["done", &handle]);

    assert!(
        printed.contains(&format!("handle alpha: {handle} → none")),
        "expected the vacated handle named: {printed}"
    );
    assert!(
        printed.contains("left the default view"),
        "expected the reason the handle is gone: {printed}"
    );
    assert!(
        !out(dir.path(), &["ls"]).contains("Task alpha"),
        "fixture must actually leave the default view"
    );
}

/// One view per batch, not one per target: `charlie`'s reported old handle is
/// its handle before the batch began, which is the one the caller is holding.
/// Flipping `bravo` first vacates an ordinal above `charlie`, so a per-target
/// recomputation would report `charlie` moving from an ordinal that was never
/// on anyone's screen — this asserts the pre-batch value exactly, and so fails
/// on that implementation rather than merely preferring this one.
#[test]
fn a_batch_diffs_against_one_pre_batch_view() {
    let dir = project();
    let bravo = handle_of(dir.path(), "Task bravo");
    let charlie = handle_of(dir.path(), "Task charlie");

    let printed = out(dir.path(), &["flip", "wip", &bravo, &charlie]);

    for (stem, was) in [("bravo", &bravo), ("charlie", &charlie)] {
        let now = handle_of(dir.path(), &format!("Task {stem}"));
        assert!(
            printed.contains(&format!("handle {stem}: {was} → {now}")),
            "expected {stem} diffed against the pre-batch view: {printed}"
        );
    }
}
