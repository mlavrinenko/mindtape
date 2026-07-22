//! End-to-end tests for the decisions `mt` only makes on a real terminal:
//! flexing the title column to the terminal width, honouring `NO_COLOR`, and
//! refusing an interactive `add` when either standard stream is piped.
//!
//! All three are invisible to in-process tests — piped stdout is not a TTY, so
//! `terminal_size()` returns `None` and `color_enabled` short-circuits before
//! it ever reads the environment. Running the binary under `script` (a
//! pseudo-terminal, util-linux) makes them observable, which pins
//! `terminal_width`, `apply_flex`, `no_color_set` and `add`'s `stdin && stdout`
//! check against mutants that would otherwise survive.
#![allow(clippy::unwrap_used)]

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Long enough that a 40-column terminal must truncate it, short enough that
/// a 200-column terminal shows it whole.
const LONG_TITLE: &str =
    "A deliberately very long task title that will not fit in a narrow terminal";

/// Run `mt <args>` inside `dir` under a pseudo-terminal `cols` columns wide.
/// `NO_COLOR` is always removed first, so an inherited value cannot skew a run;
/// `env` layers variables back on top.
fn mt_on_pty(dir: &Path, args: &str, cols: usize, env: &[(&str, &str)]) -> Output {
    let mt = PathBuf::from(env!("CARGO_BIN_EXE_mt"));
    let inner = format!("stty cols {cols} rows 24; {} {args}", mt.to_string_lossy());
    let mut command = Command::new("script");
    command
        .args(["-q", "-e", "-c", &inner, "/dev/null"])
        .current_dir(dir)
        .env_remove("NO_COLOR")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in env {
        command.env(key, value);
    }
    command
        .output()
        .expect("script must be available (util-linux)")
}

/// The pty merges the command's own stderr into its stdout, so one read covers
/// both streams.
fn pty_output(dir: &Path, args: &str, cols: usize, env: &[(&str, &str)]) -> String {
    String::from_utf8_lossy(&mt_on_pty(dir, args, cols, env).stdout).into_owned()
}

#[test]
fn narrow_terminal_shrinks_the_flex_title_column() {
    let dir = support::project_with_task(LONG_TITLE);
    let narrow = pty_output(dir.path(), "ls", 40, &[]);
    let wide = pty_output(dir.path(), "ls", 200, &[]);

    assert!(
        wide.contains(LONG_TITLE),
        "a 200-column terminal has room for the whole title\nstdout:\n{wide}"
    );
    assert!(
        !narrow.contains(LONG_TITLE),
        "a 40-column terminal must shrink the title column\nstdout:\n{narrow}"
    );
    assert!(
        narrow.contains('\u{2026}'),
        "the shrunken title is truncated with an ellipsis\nstdout:\n{narrow}"
    );
}

#[test]
fn no_color_env_var_disables_colour_on_a_tty() {
    let dir = support::project_with_task(LONG_TITLE);
    let coloured = pty_output(dir.path(), "ls", 200, &[]);
    let plain = pty_output(dir.path(), "ls", 200, &[("NO_COLOR", "1")]);

    assert!(
        coloured.contains('\u{1b}'),
        "a TTY without NO_COLOR gets ANSI styling\nstdout:\n{coloured:?}"
    );
    assert!(
        !plain.contains('\u{1b}'),
        "NO_COLOR must suppress every escape sequence\nstdout:\n{plain:?}"
    );
}

/// `add` opens the editor only when stdin *and* stdout are terminals.
/// Redirecting only stdin from `/dev/null` leaves stdout on the pty — the one
/// shape that tells `stdin && stdout` apart from `||`, whose reading would
/// spawn an editor on a stream nobody can type into.
#[test]
fn interactive_add_refuses_when_only_stdout_is_a_tty() {
    let dir = support::project_with_task(LONG_TITLE);
    // An editor that cannot be spawned: if the guard wrongly lets the edit
    // through, the run fails on the spawn instead of hanging on a real editor,
    // and the two failures are told apart by their message.
    let editor = [("EDITOR", "mt-no-such-editor"), ("VISUAL", "")];
    let out = pty_output(dir.path(), "add scratch --edit < /dev/null", 200, &editor);

    assert!(
        out.contains("interactive add needs a terminal"),
        "piped stdin must be refused before any editor is spawned\nstdout:\n{out}"
    );
    assert!(
        !dir.path().join("tasks/scratch.typ").exists(),
        "the refused add must not have written a task"
    );
}

/// Install a stand-in `$EDITOR` under `dir` that discards the scaffold and
/// writes `body` in its place, then returns its path. The heredoc delimiter is
/// quoted so the shell expands nothing — `$x$` must reach the buffer intact.
fn fake_editor(dir: &Path, body: &str) -> String {
    let path = dir.join("fake-editor.sh");
    std::fs::write(
        &path,
        format!("#!/bin/sh\ncat > \"$1\" <<'MT_EOF'\n{body}\nMT_EOF\n"),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path.to_string_lossy().into_owned()
}

/// `--edit` is the second way caller text reaches Typst markup: the values come
/// back from a saved YAML buffer, not from `--set`. Escaping is applied to the
/// merged fill map, downstream of both, and this pins that — an escaper wired
/// into the `--set` parser alone would pass every other test and lose here.
///
/// Needs a pty because `add` only opens an editor when both streams are
/// terminals, which is also why this lives beside the other terminal-only cases.
#[test]
fn interactive_add_escapes_markup_from_the_saved_buffer() {
    let dir = support::project_with_slot_template();
    let hazards = support::MARKUP_HAZARDS;
    let editor = fake_editor(
        dir.path(),
        &format!("title: \"{hazards}\"\nwhy: |-\n  {hazards}\n"),
    );

    let out = pty_output(
        dir.path(),
        "add --name edited --edit",
        200,
        &[("EDITOR", editor.as_str()), ("VISUAL", "")],
    );
    assert!(
        out.contains("wrote"),
        "the edited add must file a task: {out}"
    );

    let written = std::fs::read_to_string(dir.path().join("tasks/edited.typ")).unwrap();
    assert!(
        written.contains(r"create-\*star\* \_u\_ \$x\$ \#h \[b\] \<a> >g \@at"),
        "the buffer's text slot must be escaped: {written}"
    );

    let task = support::task_json(dir.path(), "edited");
    assert_eq!(support::at(&task, "/title").as_str(), Some(hazards));
    assert_eq!(support::at(&task, "/links/0/note").as_str(), Some(hazards));
}
