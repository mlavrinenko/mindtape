//! End-to-end test for `mt go`: verifies the binary can open a task.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;

#[test]
fn go_prints_opened_message() {
    let dir = tempdir();
    support::install_lib(dir.path());

    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();

    // Write a config that uses `true` as the editor (always succeeds).
    write_file(
        dir.path(),
        ".mindtape/config.toml",
        "[tasks]\nroots = [\"tasks\"]\n[go]\neditor = \"true\"\n",
    );

    // File a task via `mt add` so it has the right format.
    support::run_ok(dir.path(), &["add", "--name", "x", "--set", "title=Test"]);

    let output = support::mt(dir.path()).args(["go", "x"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("opened"), "mt go should print 'opened ...'");
}

/// A project whose `[go] editor` is `editor`, with one task `x` filed.
fn project_with_editor(editor: &str) -> tempfile::TempDir {
    let config = format!("[tasks]\nroots = [\"tasks\"]\n[go]\neditor = \"{editor}\"\n");
    let dir = support::scaffold_project(&config);
    support::run_ok(dir.path(), &["add", "--name", "x", "--set", "title=Test"]);
    dir
}

/// An editor that cannot run, or runs and exits non-zero, fails `mt go` and
/// names the command it tried. Neither prints `opened`: the task was not
/// opened, so the success message must not claim it was.
#[test]
fn an_editor_that_fails_to_run_fails_go_without_opening() {
    let cases = [
        ("false", "the editor exited with an error"),
        ("mt-no-such-editor-binary", "can't start the editor"),
    ];
    for (editor, reason) in cases {
        let dir = project_with_editor(editor);

        let (ok, text) = support::run_text(dir.path(), &["go", "x"]);
        assert!(!ok, "{text}");
        assert!(text.contains(reason), "{text}");
        assert!(text.contains(editor), "{text}");
        assert!(!text.contains("opened"), "{text}");
    }
}
