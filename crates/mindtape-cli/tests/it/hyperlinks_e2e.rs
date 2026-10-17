//! End-to-end tests for `--hyperlinks`: the OSC 8 sequence reaches the
//! table's `slug#` and `file` columns when forced on, but never a default
//! (opted-out) run, CSV, or JSON — and the config/flag precedence resolves as
//! documented.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::write_file;

/// Every OSC 8 *open* target in `line` — the payload between `ESC ] 8 ; ;`
/// and the following `ESC \` that is non-empty (an empty payload is a close
/// marker, `\x1b]8;;\x1b\\`, not an open one).
fn hyperlink_targets(line: &str) -> Vec<&str> {
    line.split("\u{1b}]8;;")
        .skip(1)
        .filter_map(|frag| frag.split("\u{1b}\\").next())
        .filter(|uri| !uri.is_empty())
        .collect()
}

/// [`support::project_with_task`], but with a caller-chosen config so the
/// `[ls] hyperlinks` precedence tests can set it without the flag.
fn project_with_config(config: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(config);
    write_file(
        dir.path(),
        "tasks/test.typ",
        mindtape::typst_task!("(title: \"Rewrite auth\", status: wip(2026, 6, 1))\n"),
    );
    dir
}

fn ls_stdout(dir: &std::path::Path, args: &[&str]) -> String {
    let out = support::mt(dir)
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8_lossy(&out).into_owned()
}

#[test]
fn hyperlinks_always_wraps_the_file_column() {
    let dir = support::project_with_task("Rewrite auth");
    let out = ls_stdout(
        dir.path(),
        &[
            "ls",
            "--hyperlinks",
            "always",
            "--columns",
            "slug#,title,file",
        ],
    );
    assert!(
        out.contains("\u{1b}]8;;file://"),
        "expected an OSC 8 hyperlink sequence: {out:?}"
    );
}

/// `--columns slug#,title` has no `file` column at all, so any OSC 8 sequence
/// that shows up can only have come from the `slug#` column.
#[test]
fn hyperlinks_always_wraps_the_slug_column() {
    let dir = support::project_with_task("Rewrite auth");
    let out = ls_stdout(
        dir.path(),
        &["ls", "--hyperlinks", "always", "--columns", "slug#,title"],
    );
    assert!(
        out.contains("\u{1b}]8;;file://"),
        "expected an OSC 8 hyperlink sequence on the slug# column: {out:?}"
    );
}

#[test]
fn hyperlinks_off_the_slug_column_has_no_escape() {
    let dir = support::project_with_task("Rewrite auth");
    let out = ls_stdout(dir.path(), &["ls", "--columns", "slug#,title"]);
    assert!(
        !out.contains('\u{1b}'),
        "hyperlinks default to never: {out:?}"
    );
}

#[test]
fn hyperlinks_default_never_emits_no_escape() {
    let dir = support::project_with_task("Rewrite auth");
    let out = ls_stdout(dir.path(), &["ls", "--columns", "slug#,title,file"]);
    assert!(
        !out.contains('\u{1b}'),
        "hyperlinks default to never: {out:?}"
    );
}

/// Both `slug#` and `file` link the same row, so for a single-task listing they
/// must carry the identical target (the same task's own file).
#[test]
fn slug_and_file_columns_link_to_the_same_target() {
    let dir = support::project_with_task("Rewrite auth");
    let out = ls_stdout(
        dir.path(),
        &[
            "ls",
            "--hyperlinks",
            "always",
            "--columns",
            "slug#,title,file",
        ],
    );
    let data_line = out.lines().nth(1).expect("must have one data row");
    let targets = hyperlink_targets(data_line);
    let [slug_target, file_target] = targets.as_slice() else {
        panic!("expected both slug# and file to open a hyperlink: {data_line:?}");
    };
    assert_eq!(
        slug_target, file_target,
        "slug# and file must target the same file for a given row: {data_line:?}"
    );
}

#[test]
fn hyperlinks_always_csv_carries_no_escape() {
    let dir = support::project_with_task("Rewrite auth");
    let out = ls_stdout(
        dir.path(),
        &[
            "ls",
            "--hyperlinks",
            "always",
            "--columns",
            "slug#,title,file",
            "--format",
            "csv",
        ],
    );
    assert!(
        !out.contains('\u{1b}'),
        "CSV must never carry escape sequences, even with hyperlinks forced on: {out:?}"
    );
}

#[test]
fn hyperlinks_always_json_carries_no_escape() {
    let dir = support::project_with_task("Rewrite auth");
    let out = ls_stdout(dir.path(), &["ls", "--hyperlinks", "always", "--json"]);
    assert!(
        !out.contains('\u{1b}'),
        "JSON must never carry escape sequences, even with hyperlinks forced on: {out:?}"
    );
}

#[test]
fn config_hyperlinks_always_enables_without_the_flag() {
    let dir = project_with_config("[tasks]\nroots = [\"tasks\"]\n[ls]\nhyperlinks = \"always\"\n");
    let out = ls_stdout(dir.path(), &["ls", "--columns", "slug#,title,file"]);
    assert!(
        out.contains("\u{1b}]8;;file://"),
        "[ls] hyperlinks = \"always\" must enable hyperlinks with no CLI flag: {out:?}"
    );
}

#[test]
fn flag_never_overrides_a_config_opt_in() {
    let dir = project_with_config("[tasks]\nroots = [\"tasks\"]\n[ls]\nhyperlinks = \"always\"\n");
    let out = ls_stdout(
        dir.path(),
        &[
            "ls",
            "--hyperlinks",
            "never",
            "--columns",
            "slug#,title,file",
        ],
    );
    assert!(
        !out.contains('\u{1b}'),
        "--hyperlinks never must win over [ls] hyperlinks = \"always\": {out:?}"
    );
}

#[test]
fn custom_hyperlink_target_template_is_honoured() {
    let dir = project_with_config(
        "[tasks]\nroots = [\"tasks\"]\n[ls]\nhyperlinks = \"always\"\n\
         hyperlink-target = \"editor://open?f={rel}\"\n",
    );
    let out = ls_stdout(dir.path(), &["ls", "--columns", "slug#,title,file"]);
    assert!(
        out.contains("\u{1b}]8;;editor://open?f=tasks/test.typ\u{1b}\\"),
        "custom hyperlink-target template must drive the URI: {out:?}"
    );
}

/// The help names the column the table prints, `slug#`, not a bare `#`.
#[test]
fn the_help_names_the_slug_and_file_columns() {
    let dir = support::project_with_task("Rewrite auth");
    for command in ["ls", "find", "flip"] {
        let help = ls_stdout(dir.path(), &[command, "--help"]);
        assert!(
            help.contains("`slug#` and `file` columns"),
            "mt {command} --help must name the slug# and file columns: {help}"
        );
    }
}

/// A command that prints no task table has no use for the flag, so it doesn't
/// offer it and refuses it.
#[test]
fn commands_without_a_task_table_do_not_offer_the_flag() {
    let dir = support::project_with_task("Rewrite auth");
    for command in ["at", "board", "check", "add"] {
        let help = ls_stdout(dir.path(), &[command, "--help"]);
        assert!(
            !help.contains("--hyperlinks"),
            "mt {command} --help must not list --hyperlinks: {help}"
        );
        let (code, stderr) = support::run_stderr(dir.path(), &[command, "--hyperlinks", "always"]);
        assert_eq!(
            code,
            Some(2),
            "mt {command} --hyperlinks must be a usage error: {stderr}"
        );
    }
}

/// The flag moved from the top level to the commands that use it, so it no
/// longer goes before the subcommand.
#[test]
fn the_flag_goes_after_the_command_not_before_it() {
    let dir = support::project_with_task("Rewrite auth");
    let (code, stderr) = support::run_stderr(dir.path(), &["--hyperlinks", "always", "ls"]);
    assert_eq!(code, Some(2), "{stderr}");
}
