//! End-to-end tests for `mt review`'s decision branches: refusing a terminal
//! task, the same-day idempotent no-op, and an unresolvable id. `review.rs`'s
//! own inline unit tests only exercise `build_edit` (the Typst-edit half);
//! `date_pin_e2e.rs::mindtape_now_stamps_the_pinned_date_on_review` covers the
//! happy-path stamp. None of those reach `run()`'s early-return branches,
//! which need a real project on disk and a real `cwd`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

mod support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), CONFIG).unwrap();
    support::install_lib(dir);
    std::fs::write(
        dir.join("tasks/wire-oauth.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Wire OAuth\",\n  status: wip(2026, 6, 1),\n)\n",
    )
    .unwrap();
}

#[test]
fn refuses_to_review_a_terminal_task() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    support::mt(dir.path())
        .args(["done", "wire-oauth"])
        .assert()
        .success();

    let output = support::mt(dir.path())
        .args(["review", "wire-oauth"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr).into_owned();
    assert!(stderr.contains("terminal status"), "{stderr}");

    // A refusal must not touch the file: no `reviewed:` slot appears.
    let written = std::fs::read_to_string(dir.path().join("tasks/wire-oauth.typ")).unwrap();
    assert!(!written.contains("reviewed:"), "{written}");
}

#[test]
fn reviewing_twice_the_same_day_is_a_no_op_after_the_first() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    support::mt(dir.path())
        .env("MINDTAPE_NOW", "2026-09-05")
        .args(["review", "wire-oauth"])
        .assert()
        .success();
    let first = std::fs::read_to_string(dir.path().join("tasks/wire-oauth.typ")).unwrap();
    assert!(first.contains("reviewed: on(2026, 9, 5)"), "{first}");

    let output = support::mt(dir.path())
        .env("MINDTAPE_NOW", "2026-09-05")
        .args(["review", "wire-oauth"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout).into_owned();
    assert!(stdout.contains("already reviewed today"), "{stdout}");

    let second = std::fs::read_to_string(dir.path().join("tasks/wire-oauth.typ")).unwrap();
    assert_eq!(
        first, second,
        "an already-reviewed-today run must not rewrite the file"
    );
}

/// A stamp that would leave the task invalid is refused before the write, and
/// the file is byte-identical afterwards. Refusing is the whole answer to
/// "should a rejected rewrite be restored": there is nothing to restore,
/// because nothing was written. The `reviewed` field is required only of
/// non-terminal tasks here, so the fixture's real violation is the tag — a
/// pre-existing one the stamp does not create but does have to carry.
#[test]
fn a_rejected_stamp_leaves_the_file_untouched() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        format!("{CONFIG}\n[tasks.tags]\nallowed = [\"ok\"]\n"),
    )
    .unwrap();
    support::install_lib(dir.path());
    let body = "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Wire OAuth\",\n  status: wip(2026, 6, 1),\n  \
         tags: (\"nope\",),\n)\n";
    let file = dir.path().join("tasks/wire-oauth.typ");
    std::fs::write(&file, body).unwrap();

    let output = support::mt(dir.path())
        .args(["review", "wire-oauth"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr).into_owned();
    assert!(stderr.contains("nothing was written"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        body,
        "a refused stamp must leave the file byte-identical"
    );
}

#[test]
fn reviewing_an_unresolvable_id_fails() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    let output = support::mt(dir.path())
        .args(["review", "does-not-exist"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr).into_owned();
    assert!(stderr.contains("does-not-exist"), "{stderr}");
}
