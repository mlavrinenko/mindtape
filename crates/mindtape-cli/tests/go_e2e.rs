//! End-to-end test for `mt go`: verifies the binary can open a task.
#![allow(clippy::unwrap_used)]

mod support;

#[test]
fn go_prints_opened_message() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());

    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();

    // Write a config that uses `true` as the editor (always succeeds).
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n[go]\neditor = \"true\"\n",
    )
    .unwrap();

    // File a task via `mt add` so it has the right format.
    support::mt(dir.path())
        .args(["add", "--name", "x", "--set", "title=Test"])
        .assert()
        .success();

    let output = support::mt(dir.path()).args(["go", "x"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("opened"), "mt go should print 'opened ...'");
}
