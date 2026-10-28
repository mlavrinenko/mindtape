//! End-to-end tests for `mt ls --format path` / `path0`
//! (`truth/tasks/emit-a-plain-path-list-from-ls.typ`): the plain path list
//! that composes with another program without a `jq` dependency.
#![allow(clippy::unwrap_used)]

use crate::support;

fn scaffold() -> tempfile::TempDir {
    support::project_with_tasks(&[("a.typ", "Alpha"), ("b.typ", "Beta")])
}

#[test]
fn path_format_lists_one_path_per_line_with_no_header_or_trailer() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .args(["ls", "", "--format", "path"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut lines: Vec<&str> = stdout.lines().collect();
    lines.sort_unstable();
    assert_eq!(lines, ["tasks/a.typ", "tasks/b.typ"].map(support::native));
    assert!(
        !stdout.contains("of 2 tasks"),
        "no footer for a program format"
    );
}

#[test]
fn path0_format_is_nul_delimited_and_self_terminated() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .args(["ls", "", "--format", "path0"])
        .output()
        .unwrap();
    let raw = &out.stdout;
    assert!(
        !raw.ends_with(b"\n"),
        "path0 must not carry a stray trailing newline (it would land inside \
         the last `xargs -0` argument): {raw:?}"
    );
    assert!(
        raw.ends_with(b"\0"),
        "every record, including the last, is NUL-terminated: {raw:?}"
    );
    let mut parts: Vec<&str> = std::str::from_utf8(raw)
        .unwrap()
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();
    parts.sort_unstable();
    assert_eq!(parts, ["tasks/a.typ", "tasks/b.typ"].map(support::native));
}

/// The spec's whole premise: paths render exactly as the table's `file`
/// column does, so the two agree by construction — and they have to agree
/// from a subdirectory too, which is the only place the claim has content.
/// Run from `tasks/`, both spell every path `../tasks/…`.
#[test]
fn path_format_matches_the_table_file_column_exactly() {
    let dir = scaffold();
    let cwd = dir.path().join("tasks");

    let table = support::mt(dir.path())
        .current_dir(&cwd)
        .args(["ls", "", "--format", "table", "--columns", "file"])
        .output()
        .unwrap();
    let table_stdout = String::from_utf8_lossy(&table.stdout);
    // Drop the `FILE` header row, and the blank line and footer under the
    // rows; each remaining row is the one column, padded to width.
    let mut table_cells: Vec<&str> = table_stdout
        .lines()
        .skip(1)
        .map(str::trim_end)
        .take_while(|line| !line.is_empty())
        .collect();
    table_cells.sort_unstable();
    assert_eq!(
        table_cells,
        ["../tasks/a.typ", "../tasks/b.typ"].map(support::native)
    );

    let path = support::mt(dir.path())
        .current_dir(&cwd)
        .args(["ls", "", "--format", "path"])
        .output()
        .unwrap();
    let path_stdout = String::from_utf8_lossy(&path.stdout);
    let mut path_lines: Vec<&str> = path_stdout.lines().collect();
    path_lines.sort_unstable();

    assert_eq!(table_cells, path_lines);
}

/// `file` names one value across both machine formats. Run from `tasks/`,
/// `--format csv` and `--format json` both report the project-root-relative
/// `tasks/a.typ` — the value `mt import` reads back and the `path` filter
/// field compares — while the table and `--format path` show `../tasks/a.typ`
/// for the same task. Before this, CSV rendered the display and the two
/// machine formats disagreed by whichever directory the producer stood in
/// (`truth/tasks/csv-and-json-disagree-about-what-the-file-field-means.typ`).
#[test]
fn csv_and_json_report_the_same_root_relative_file_from_a_subdirectory() {
    let dir = scaffold();
    let cwd = dir.path().join("tasks");

    let csv = support::mt(dir.path())
        .current_dir(&cwd)
        .args(["ls", "", "--format", "csv", "--columns", "file"])
        .output()
        .unwrap();
    let csv_stdout = String::from_utf8_lossy(&csv.stdout);
    // Skip the header row.
    let mut csv_cells: Vec<&str> = csv_stdout
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .collect();
    csv_cells.sort_unstable();
    assert_eq!(csv_cells, vec!["tasks/a.typ", "tasks/b.typ"]);

    let json = support::mt(dir.path())
        .current_dir(&cwd)
        .args(["ls", "", "--format", "json"])
        .output()
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    let mut json_files: Vec<&str> = support::at(&parsed, "/tasks")
        .as_array()
        .unwrap()
        .iter()
        .map(|task| support::at(task, "/file").as_str().unwrap())
        .collect();
    json_files.sort_unstable();

    assert_eq!(csv_cells, json_files);
}

/// The `file` display and the `path` filter field are two different values,
/// and this is the run that proves it: from `tasks/`, the column and
/// `--format path` render `../tasks/a.typ` (openable from where the reader is
/// standing) while the filter still compares the root-relative
/// `tasks/a.typ` — so a view or `[[ls.rules]]` entry means the same thing
/// wherever `mt` runs from
/// (`truth/tasks/the-file-filter-field-and-the-file-column-name-different-paths.typ`).
#[test]
fn the_path_field_stays_root_relative_while_the_display_follows_the_cwd() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .current_dir(dir.path().join("tasks"))
        .args(["ls", "path == \"tasks/a.typ\"", "--format", "path"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [support::native("../tasks/a.typ")]
    );
}

/// The rename's whole point: the old spelling fails loudly rather than
/// silently. Before it, a filter built from what the listing showed returned
/// zero rows and exit 0 — indistinguishable from a correct filter over an
/// empty result set. `file` is simply not a filter field any more, so it
/// fails the same way any other unknown field does, with no pointer back to
/// `path`.
#[test]
fn the_file_filter_field_fails_loudly_as_unknown() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .current_dir(dir.path().join("tasks"))
        .args(["ls", "file == \"../tasks/a.typ\"", "--format", "path"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "an unknown field must fail, not list nothing: {out:?}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown field: 'file'"), "{stderr}");
}

/// The same trap on the sort side, which a saved `[query] default_sort` or a
/// `--sort` flag reaches instead of the filter parser.
#[test]
fn the_file_sort_field_fails_loudly_as_unknown() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .args(["ls", "", "--sort", "file"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown sort field: 'file'"), "{stderr}");
}

#[test]
fn path_format_prints_no_bytes_for_no_match_and_one_final_newline_otherwise() {
    let dir = scaffold();
    let listing = |filter: &str| support::stdout(dir.path(), &["ls", filter, "--format", "path"]);
    assert_eq!(listing(r#"title == "nothing""#), "");
    let all = listing("");
    assert!(all.ends_with('\n') && !all.ends_with("\n\n"), "{all:?}");
}

#[test]
fn path_format_composes_with_a_real_pipe() {
    // `xargs -n1 echo` proves the output is directly consumable, not just
    // shaped like it — the recipe the spec exists for. xargs reads `\` as an
    // escape, so on Windows the recipe asks for `/` with `--path-separator`.
    let dir = scaffold();
    let mt_out = support::mt(dir.path())
        .args(["--path-separator", "/", "ls", "", "--format", "path"])
        .output()
        .unwrap();
    assert!(mt_out.status.success());

    let mut xargs = std::process::Command::new("xargs")
        .args(["-n1", "echo"])
        .current_dir(dir.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        xargs
            .stdin
            .take()
            .unwrap()
            .write_all(&mt_out.stdout)
            .unwrap();
    }
    let xargs_out = xargs.wait_with_output().unwrap();
    let echoed = String::from_utf8_lossy(&xargs_out.stdout);
    assert!(echoed.contains("tasks/a.typ"));
    assert!(echoed.contains("tasks/b.typ"));
}
