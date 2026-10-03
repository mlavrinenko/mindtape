//! End-to-end tests for `--hyperlinks`: the OSC 8 sequence reaches the
//! table's `slug#` and `file` columns when forced on, but never a default
//! (opted-out) run, CSV, or JSON — and the config/flag precedence resolves as
//! documented.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

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
    std::fs::write(
        dir.path().join("tasks/test.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"Rewrite auth\", status: wip(2026, 6, 1))\n",
    )
    .unwrap();
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
