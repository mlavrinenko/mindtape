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
    support::run_ok(dir.path(), &["allow"]);

    let output = support::mt(dir.path()).args(["go", "x"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("opened"), "mt go should print 'opened ...'");
}

/// A project with `config` and one task `x` filed.
fn project_with_config(config: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(config);
    support::run_ok(dir.path(), &["add", "--name", "x", "--set", "title=Test"]);
    dir
}

/// A project whose `[go] editor` is `editor`, allowed, with one task `x` filed.
fn project_with_editor(editor: &str) -> tempfile::TempDir {
    let dir = project_with_config(&format!(
        "[tasks]\nroots = [\"tasks\"]\n[go]\neditor = \"{editor}\"\n"
    ));
    support::run_ok(dir.path(), &["allow"]);
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

#[cfg(unix)]
/// A script under `dir` that appends `name` to `marks.txt`, returned as an
/// editor command, so a test reads which editor ran.
fn marking_editor(dir: &std::path::Path, name: &str) -> String {
    let script = dir.join(format!("{name}.sh"));
    std::fs::write(
        &script,
        format!("echo {name} >> \"{}/marks.txt\"\n", dir.display()),
    )
    .unwrap();
    format!("sh {}", script.display())
}

#[cfg(unix)]
/// A project with task `x` and two marking editors: one called `name`, and
/// `visual`. Returns the project and the two editor commands, in that order.
fn project_with_marking_editors(name: &str) -> (tempfile::TempDir, String, String) {
    let dir = project_with_config("[tasks]\nroots = [\"tasks\"]\n");
    let named = marking_editor(dir.path(), name);
    let visual = marking_editor(dir.path(), "visual");
    (dir, named, visual)
}

#[cfg(unix)]
/// `mt go x` in `dir` with `$VISUAL` set to `visual` and, when given, the
/// config home `home`. Asserts the run succeeded and the editors marked
/// `marks.txt` as `expected` so far, and returns stderr.
fn go_marking(
    dir: &std::path::Path,
    visual: &str,
    home: Option<&std::path::Path>,
    expected: &str,
) -> String {
    let mut command = support::mt(dir);
    command.args(["go", "x"]).env("VISUAL", visual);
    if let Some(home) = home {
        command.env(mindtape::CONFIG_HOME_ENV, home);
    }
    let out = command.output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(out.status.success(), "{stderr}");
    let marks = std::fs::read_to_string(dir.join("marks.txt")).unwrap_or_default();
    assert_eq!(marks, expected, "{stderr}");
    stderr
}

#[cfg(unix)]
/// A clone's `[go] editor` is a program `mt` would run, so it waits for
/// `mt allow` like a checker does: until then `$VISUAL` opens the task and
/// one warning names the held command.
#[test]
fn a_project_editor_waits_for_mt_allow_and_visual_opens_the_task() {
    let (dir, held, visual) = project_with_marking_editors("held");
    let config = format!("[tasks]\nroots = [\"tasks\"]\n[go]\neditor = {held:?}\n");
    write_file(dir.path(), ".mindtape/config.toml", &config);

    let stderr = go_marking(dir.path(), &visual, None, "visual\n");

    assert_eq!(stderr.matches("warning:").count(), 1, "{stderr}");
    assert!(stderr.contains(&held), "{stderr}");
    assert!(
        stderr.contains("hint: read them, then run mt allow"),
        "{stderr}"
    );

    support::run_ok(dir.path(), &["allow"]);
    let stderr = go_marking(dir.path(), &visual, None, "visual\nheld\n");
    assert!(!stderr.contains("warning:"), "{stderr}");
}

#[cfg(unix)]
/// The user's own `<config home>/mindtape/defaults.toml` is theirs, so its
/// `[go] editor` runs with no `mt allow`, ahead of `$VISUAL`.
#[test]
fn an_editor_from_the_users_own_config_runs_without_mt_allow() {
    let (dir, mine, visual) = project_with_marking_editors("mine");
    let home = tempdir();
    write_file(
        home.path().join("mindtape"),
        "defaults.toml",
        format!("[go]\neditor = {mine:?}\n"),
    );

    let stderr = go_marking(dir.path(), &visual, Some(home.path()), "mine\n");

    assert!(!stderr.contains("warning:"), "{stderr}");
}

/// `mt go x` in `dir` with no `$VISUAL` or `$EDITOR`: the exit code and stderr.
fn go_without_fallback(dir: &std::path::Path) -> (Option<i32>, String) {
    let out = support::mt(dir)
        .args(["go", "x"])
        .env_remove("EDITOR")
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// With no `$VISUAL` or `$EDITOR`, a held `[go] editor` is refused with the
/// reason and a hint at `mt allow`, not at setting an editor that is already
/// set, while a blank one holds nothing back and still asks for an editor.
#[test]
fn a_held_editor_with_no_fallback_points_at_mt_allow_and_a_blank_one_does_not() {
    let cases = [
        (
            "true",
            "approval: 'true'\nhint: read it, then run mt allow",
            "no editor configured",
        ),
        ("  ", "no editor configured", "mt allow"),
    ];
    for (editor, expected, absent) in cases {
        let dir = project_with_config(&format!(
            "[tasks]\nroots = [\"tasks\"]\n[go]\neditor = \"{editor}\"\n"
        ));

        let (code, stderr) = go_without_fallback(dir.path());

        assert_eq!(code, Some(1), "{stderr}");
        assert!(stderr.contains(expected), "{stderr}");
        assert!(!stderr.contains(absent), "{stderr}");
    }
}
