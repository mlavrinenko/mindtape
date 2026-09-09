//! End-to-end tests for `mt ls -n / --limit` (`truth/tasks/ls-limit-flag.typ`):
//! the native truncation path, and what a truncated listing says about the
//! rows it hid.
//!
//! The signal is format-shaped on purpose. A table gets a `truncated:` footer
//! beside the `applied:` trailer; `--json` gets an envelope key, since it is
//! already a document rather than a record stream; and `csv` / `path` /
//! `path0` get nothing at all — a prose line appended to those is corruption
//! for the `csv` reader or `xargs -0` on the other end of the pipe, which is
//! the exact failure `--limit` exists to avoid.
#![allow(clippy::unwrap_used)]

mod support;

use support::stdout;

/// Three listable tasks, titled so a `--sort title` run has a decidable order.
fn scaffold() -> tempfile::TempDir {
    support::project_with_tasks(&[("a.typ", "Alpha"), ("b.typ", "Beta"), ("c.typ", "Gamma")])
}

#[test]
fn limit_truncates_the_table_and_says_what_it_cut() {
    let text = stdout(scaffold().path(), &["ls", "", "-n", "1"]);
    assert!(
        text.contains("truncated: showing 1 of 3"),
        "the table has room for a footer, so it owes the reader the count \
         it hid: {text}"
    );
}

#[test]
fn a_limit_that_hides_nothing_prints_no_footer() {
    // Three tasks under `-n 3`: the cap is met exactly and nothing is hidden,
    // so a footer here would be a claim the reader acts on and shouldn't.
    let text = stdout(scaffold().path(), &["ls", "", "-n", "3"]);
    assert!(!text.contains("truncated"), "got: {text}");
}

#[test]
fn an_unlimited_listing_prints_no_footer() {
    let text = stdout(scaffold().path(), &["ls", ""]);
    assert!(!text.contains("truncated"), "got: {text}");
}

#[test]
fn limit_applies_after_the_sort() {
    // The whole point of truncating last: `-n 1` must yield the top row of
    // the *sorted* list, not whichever task the loader happened to reach
    // first. `Gamma` sorts last ascending, so it is the only descending pick.
    let dir = scaffold();
    let text = stdout(
        dir.path(),
        &[
            "ls",
            "",
            "--sort",
            "title:desc",
            "-n",
            "1",
            "--format",
            "path",
        ],
    );
    assert_eq!(text.lines().collect::<Vec<_>>(), vec!["tasks/c.typ"]);
}

#[test]
fn json_reports_the_truncation_in_the_envelope() {
    let text = stdout(scaffold().path(), &["ls", "", "-n", "1", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(support::at(&json, "/tasks").as_array().unwrap().len(), 1);
    assert_eq!(support::at(&json, "/truncated/shown"), 1);
    assert_eq!(support::at(&json, "/truncated/total"), 3);
    assert!(
        !text.contains("truncated: showing"),
        "the prose footer is table-only; --json carries the fact structurally"
    );
}

#[test]
fn json_reports_a_complete_listing_as_null() {
    let text = stdout(scaffold().path(), &["ls", "", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(
        support::at(&json, "/truncated").is_null(),
        "the key is always present, so `.truncated` is one read: {text}"
    );
}

#[test]
fn csv_is_truncated_without_a_trailing_prose_line() {
    let dir = scaffold();
    let text = stdout(dir.path(), &["ls", "", "-n", "1", "--format", "csv"]);
    let rows: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(rows.len(), 2, "header plus exactly one record: {text}");
    assert!(!text.contains("truncated"), "got: {text}");
}

#[test]
fn path_is_truncated_without_a_trailing_prose_line() {
    let dir = scaffold();
    let text = stdout(dir.path(), &["ls", "", "-n", "2", "--format", "path"]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "two paths and nothing else: {text}");
    assert!(
        lines.iter().all(|line| line.ends_with(".typ")),
        "got: {text}"
    );
}

#[test]
fn path0_is_truncated_and_stays_self_terminated() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .args(["ls", "", "-n", "2", "--format", "path0"])
        .output()
        .unwrap();
    let raw = &out.stdout;
    assert!(
        raw.ends_with(b"\0") && !raw.ends_with(b"\n"),
        "a footer here would land inside the last `xargs -0` argument: {raw:?}"
    );
    assert_eq!(raw.iter().filter(|&&byte| byte == 0).count(), 2);
}

#[test]
fn find_forwards_the_limit_and_its_footer() {
    // `find` is sugar over `ls` and flattens the same flag struct, so the
    // footer has to arrive with the flag rather than being wired per command.
    let dir = scaffold();
    let text = stdout(dir.path(), &["find", "a", "-n", "1"]);
    assert!(text.contains("truncated: showing 1 of 3"), "got: {text}");
}
