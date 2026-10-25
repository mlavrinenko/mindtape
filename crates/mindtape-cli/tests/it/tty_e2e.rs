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
//!
//! Unix only. The harness is util-linux `script` driving a pty, plus `stty` and
//! `/dev/null`; Windows' `ConPTY` has no drop-in command-line equivalent, so
//! there is nothing here to port rather than rewrite. The behaviours under test
//! are platform-independent (a width from `terminal_size`, an environment
//! variable, an `IsTerminal` check), so verifying them on Linux and macOS
//! covers the logic — what stays unverified on Windows is the pty plumbing
//! underneath, which is `crossterm`'s.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

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
    support::isolate(&mut command, dir)
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

#[test]
fn hyperlinks_auto_needs_a_tty_and_a_non_dumb_term() {
    // The harness's own ambient `TERM` cannot be trusted either way (observed
    // "dumb" in this sandbox), so both arms set `TERM` explicitly rather than
    // inheriting it — unlike `no_color_env_var_disables_colour_on_a_tty`,
    // whose `auto` never reads `TERM` at all.
    let dir = support::project_with_task(LONG_TITLE);
    let args = "ls --hyperlinks auto --columns slug#,title,file";
    let non_dumb_term = pty_output(dir.path(), args, 200, &[("TERM", "xterm-256color")]);
    let dumb_term = pty_output(dir.path(), args, 200, &[("TERM", "dumb")]);

    assert!(
        non_dumb_term.contains("\u{1b}]8;;file://"),
        "a real TTY with a non-dumb TERM must enable auto hyperlinks\nstdout:\n{non_dumb_term}"
    );
    // Check the OSC 8 signature specifically, not "any escape byte": the
    // trailer line still gets its ordinary dim colour styling on a TTY
    // regardless of TERM (colour's `auto` never reads `TERM`), so a bare
    // `contains('\x1b')` would false-positive on that unrelated escape.
    assert!(
        !dumb_term.contains("\u{1b}]8;;"),
        "TERM=dumb must suppress auto hyperlinks even on a TTY\nstdout:\n{dumb_term:?}"
    );
}

/// Run `mt <args>` inside `dir` with stdout on an ordinary pipe — the shape
/// every captured run has, and the one term of the `file` column's floor
/// expression a pty cannot produce.
fn piped_output(dir: &Path, args: &[&str]) -> String {
    let mut command = Command::new(PathBuf::from(env!("CARGO_BIN_EXE_mt")));
    support::isolate(&mut command, dir)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let out = command.output().expect("mt must run");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The embedded floor's `[ls.columns.file] when`: the `file` column stays
/// opt-in on a terminal, where the `slug#` stem hash is the address a reader
/// wants, and turns itself on in captured output, where a quoted hash can turn
/// ambiguous and the path is the stable thing to quote.
///
/// Only reachable end to end: an in-process test resolves the expression
/// against an injected environment, which cannot tell whether the real
/// `IsTerminal` check is wired to it at all.
#[test]
fn the_file_column_turns_itself_on_when_stdout_is_captured() {
    let dir = support::project_with_task(LONG_TITLE);
    let on_pty = pty_output(dir.path(), "ls", 200, &[]);
    let piped = piped_output(dir.path(), &["ls"]);

    assert!(
        !on_pty.contains("FILE"),
        "a terminal keeps the file column opt-in\nstdout:\n{on_pty}"
    );
    assert!(
        piped.contains("FILE"),
        "captured output must carry the path column\nstdout:\n{piped}"
    );
}

/// The `named()` term of the same expression, on the side a pty is needed for:
/// asking for the column is what turns it on where the captured-table terms are
/// both false. It is the opt-in the docs hand a reader on a terminal, and it
/// reaches the real `IsTerminal` check the same way its sibling above does.
#[test]
fn naming_the_file_column_shows_it_on_a_terminal() {
    let dir = support::project_with_task(LONG_TITLE);
    let named = pty_output(dir.path(), "ls --columns slug#,title,file", 200, &[]);

    assert!(
        named.contains("FILE"),
        "--columns must show the file column on a terminal\nstdout:\n{named}"
    );
}

/// CSV resolves the same column set as the table, so the floor's expression
/// scopes itself with `format() == "table"`. A script parsing `mt ls --format
/// csv` must not silently gain a column — and it already carries the stable
/// `id`, so it would gain nothing by it either.
#[test]
fn the_file_column_rule_leaves_captured_csv_alone() {
    let dir = support::project_with_task(LONG_TITLE);
    let csv = piped_output(dir.path(), &["ls", "--format", "csv"]);
    let header = csv.lines().next().unwrap_or_default().to_owned();

    assert!(
        header.contains("id"),
        "CSV addresses a task by its stable id\nheader: {header}"
    );
    assert!(
        !header.contains("file"),
        "the table-scoped rule must not reach CSV\nheader: {header}"
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

/// `[add] editor` of a clone is held like `[go] editor`: until `mt allow`,
/// `--edit` opens `$VISUAL` instead, and the held command is named.
#[test]
fn interactive_add_holds_a_project_editor_until_mt_allow() {
    let dir = support::project_with_slot_template();
    let held = fake_editor(dir.path(), "title: \"Held\"\nwhy: |-\n  why");
    let visual = dir.path().join("visual.sh");
    std::fs::write(
        &visual,
        "#!/bin/sh\ncat > \"$1\" <<'MT_EOF'\ntitle: \"Visual\"\nwhy: |-\n  why\nMT_EOF\n",
    )
    .unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&visual, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let config = dir.path().join(".mindtape/config.toml");
    let text = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("{text}\n[add]\neditor = {held:?}\n")).unwrap();
    let env = [("VISUAL", visual.to_str().unwrap())];

    let out = pty_output(dir.path(), "add --name first --edit", 200, &env);

    assert!(out.contains("wrote"), "{out}");
    assert!(out.contains("warning:"), "{out}");
    assert!(out.contains(&held), "{out}");
    let first = std::fs::read_to_string(dir.path().join("tasks/first.typ")).unwrap();
    assert!(first.contains("Visual"), "{first}");

    support::run_ok(dir.path(), &["allow"]);
    let out = pty_output(dir.path(), "add --name second --edit", 200, &env);

    assert!(out.contains("wrote"), "{out}");
    assert!(!out.contains("warning:"), "{out}");
    let second = std::fs::read_to_string(dir.path().join("tasks/second.typ")).unwrap();
    assert!(second.contains("Held"), "{second}");
}
