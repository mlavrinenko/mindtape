//! End-to-end tests for `mt ls -n / --limit / --skip`
//! (`truth/tasks/ls-limit-flag.typ`,
//! `truth/tasks/add-skip-and-a-next-command-hint-to-ls-paging.typ`): the
//! native truncation path, `--skip` paging past what a prior page showed, and
//! what a truncated listing says about the rows it hid.
//!
//! The signal is format-shaped on purpose. A table gets a `more not shown` line
//! under its footer; `--json` gets an envelope key, since it is already a
//! document rather than a record stream; and `csv` / `path` /
//! `path0` get nothing at all — a prose line appended to those is corruption
//! for the `csv` reader or `xargs -0` on the other end of the pipe, which is
//! the exact failure `--limit` exists to avoid.
#![allow(clippy::unwrap_used)]

use crate::support;

use support::stdout;

/// Three listable tasks, titled so a `--sort title` run has a decidable order.
fn scaffold() -> tempfile::TempDir {
    support::project_with_tasks(&[("a.typ", "Alpha"), ("b.typ", "Beta"), ("c.typ", "Gamma")])
}

#[test]
fn limit_truncates_the_table_and_says_what_it_cut() {
    let text = stdout(scaffold().path(), &["ls", "", "-n", "1"]);
    assert!(
        text.contains(
            "your filter · 1 of 3 tasks\n2 more not shown · next page: mt ls '' -n 1 --skip 1"
        ),
        "the table has room for a footer, so it owes the reader the count \
         it hid and the command that fetches the rest: {text}"
    );
}

#[test]
fn skip_drops_rows_before_the_limit_and_names_the_next_page() {
    let text = stdout(scaffold().path(), &["ls", "", "-n", "1", "--skip", "1"]);
    // `Beta` sorts second (default order falls back to id ascending: a, b,
    // c), so `--skip 1` before `-n 1` lands on it, not `Alpha` again.
    assert!(text.contains("Beta"), "got: {text}");
    assert!(!text.contains("Alpha"), "got: {text}");
    assert!(
        text.contains("1 more not shown · next page: mt ls '' -n 1 --skip 2"),
        "the hint advances past this page's own --skip, not past 0: {text}"
    );
}

#[test]
fn skip_past_the_match_count_prints_no_footer() {
    // Every row is already behind this `--skip`, so there is no next page —
    // an empty result is complete in itself, not a truncation.
    let text = stdout(scaffold().path(), &["ls", "", "-n", "1", "--skip", "3"]);
    assert!(!text.contains("next page"), "got: {text}");
}

#[test]
fn a_limit_that_hides_nothing_prints_no_footer() {
    // Three tasks under `-n 3`: the cap is met exactly and nothing is hidden,
    // so a footer here would be a claim the reader acts on and shouldn't.
    let text = stdout(scaffold().path(), &["ls", "", "-n", "3"]);
    assert!(!text.contains("next page"), "got: {text}");
}

#[test]
fn an_unlimited_listing_prints_no_footer() {
    let text = stdout(scaffold().path(), &["ls", ""]);
    assert!(!text.contains("next page"), "got: {text}");
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
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        [support::native("tasks/c.typ")]
    );
}

#[test]
fn json_reports_the_truncation_in_the_envelope() {
    let text = stdout(scaffold().path(), &["ls", "", "-n", "1", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(support::at(&json, "/tasks").as_array().unwrap().len(), 1);
    assert_eq!(support::at(&json, "/truncated/shown"), 1);
    assert_eq!(support::at(&json, "/truncated/total"), 3);
    assert_eq!(support::at(&json, "/truncated/next_skip"), 1);
    assert!(
        !text.contains("more not shown"),
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
    assert!(!text.contains("next page"), "got: {text}");
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
    assert!(
        text.contains("2 more not shown · next page: mt ls 'contains(\"a\")' -n 1 --skip 1"),
        "got: {text}"
    );
}

/// `mt <selector…> ls -n 1` from an unrelated directory, with `cfg` as the
/// config home and `env` set: its stdout.
fn paged(selector: &[&str], cfg: &std::path::Path, env: &[(&str, &std::path::Path)]) -> String {
    let here = tempfile::tempdir().unwrap();
    let mut command = support::mt(here.path());
    command.env(mindtape::CONFIG_HOME_ENV, cfg);
    command.envs(env.iter().copied());
    let out = command
        .args(selector)
        .args(["ls", "-n", "1"])
        .output()
        .unwrap();
    let (code, stdout, stderr) = support::parts_of(&out);
    assert_eq!(code, Some(0), "{stderr}");
    stdout
}

#[test]
fn a_named_project_pages_under_its_name() {
    let project = scaffold();
    let cfg = tempfile::tempdir().unwrap();
    support::write_at_toml(cfg.path(), &[("solo", &[project.path()])]);
    let text = paged(&["@solo"], cfg.path(), &[]);
    assert!(
        text.ends_with("next page: mt @solo ls -n 1 --skip 1\n"),
        "got: {text}"
    );
}

#[test]
fn a_worktree_flag_pages_with_the_flag_as_typed() {
    let project = scaffold();
    let cfg = tempfile::tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    let text = paged(&["-w", path], cfg.path(), &[]);
    let hint = format!("next page: mt -w {path} ls -n 1 --skip 1\n");
    assert!(text.ends_with(&hint), "got: {text}");
}

#[test]
fn a_worktree_path_with_spaces_is_quoted_in_the_hint() {
    let project = scaffold();
    let parent = tempfile::tempdir().unwrap();
    let spaced = parent.path().join("my proj");
    std::fs::rename(project.path(), &spaced).unwrap();
    let cfg = tempfile::tempdir().unwrap();
    let text = paged(&["-w", spaced.to_str().unwrap()], cfg.path(), &[]);
    let hint = format!(
        "next page: mt -w '{}' ls -n 1 --skip 1\n",
        spaced.to_string_lossy()
    );
    assert!(text.ends_with(&hint), "got: {text}");
}

#[test]
fn the_worktree_env_stays_out_of_the_hint() {
    // The environment carries over when the hint is pasted, so restating it
    // would only pin a path the reader may have since changed.
    let project = scaffold();
    let cfg = tempfile::tempdir().unwrap();
    let text = paged(&[], cfg.path(), &[("MINDTAPE_WORKTREE", project.path())]);
    assert!(
        text.ends_with("next page: mt ls -n 1 --skip 1\n"),
        "got: {text}"
    );
}
