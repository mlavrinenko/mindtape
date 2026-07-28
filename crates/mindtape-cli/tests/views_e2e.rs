//! End-to-end tests for `mt views`.
#![allow(clippy::unwrap_used)]

mod support;

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
