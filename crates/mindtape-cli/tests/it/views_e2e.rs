//! End-to-end tests for `mt views`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

fn scaffold_with_views(dir: &std::path::Path) {
    support::install_lib(dir);
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n\n[views.hot]\nfilter = 'priority > 5'\nsort = ['priority:desc']\n\n[views.simple]\nfilter = ''\n",
    )
    .unwrap();
}

#[test]
fn views_lists_known_views() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_views(dir.path());

    let output = support::mt(dir.path()).args(["views"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hot"));
    assert!(stdout.contains("simple"));
}

#[test]
fn views_detail_shows_view_definition() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_views(dir.path());

    let output = support::mt(dir.path())
        .args(["views", "hot"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("priority > 5"));
    assert!(stdout.contains("sort:"));
    assert!(stdout.contains("priority:desc"));
}

#[test]
fn views_detail_unknown_name_suggests_the_near_miss() {
    // `truth/tasks/suggest-a-nearest-match-on-an-unknown-view-name.typ`: a
    // typo close to a declared view gets a "did you mean" alongside the
    // hint; a bare "unknown view: <name>" would leave both to guesswork.
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_views(dir.path());

    let output = support::mt(dir.path())
        .args(["views", "hott"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The embedded floor's own `[views.default]` is in the mix too.
    assert!(
        stderr.contains("error: unknown view: 'hott'\nhint: did you mean hot?\n"),
        "{stderr}"
    );
}

#[test]
fn views_detail_omits_empty_sort_and_columns() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_views(dir.path());

    let output = support::mt(dir.path())
        .args(["views", "simple"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("sort:"));
    assert!(!stdout.contains("cols:"));
}
