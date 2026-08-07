//! End-to-end tests for `mt ls --format path` / `path0`
//! (`truth/tasks/emit-a-plain-path-list-from-ls.typ`): the plain path list
//! that composes with another program without a `jq` dependency.
#![allow(clippy::unwrap_used)]

mod support;

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
    assert_eq!(lines, vec!["tasks/a.typ", "tasks/b.typ"]);
    assert!(
        !stdout.contains("applied:"),
        "no trailer for a program format"
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
    assert_eq!(parts, vec!["tasks/a.typ", "tasks/b.typ"]);
}

/// The spec's whole premise: paths render exactly as the `file` column does
/// today, so the two agree by construction.
#[test]
fn path_format_matches_the_file_column_exactly() {
    let dir = scaffold();

    let csv = support::mt(dir.path())
        .args(["ls", "", "--format", "csv", "--columns", "file"])
        .output()
        .unwrap();
    let csv_stdout = String::from_utf8_lossy(&csv.stdout);
    // Skip the header row and drop the blank line `render_csv`'s own CRLF
    // terminator plus the outer `println!` together leave at the end.
    let mut csv_cells: Vec<&str> = csv_stdout
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .collect();
    csv_cells.sort_unstable();

    let path = support::mt(dir.path())
        .args(["ls", "", "--format", "path"])
        .output()
        .unwrap();
    let path_stdout = String::from_utf8_lossy(&path.stdout);
    let mut path_lines: Vec<&str> = path_stdout.lines().collect();
    path_lines.sort_unstable();

    assert_eq!(csv_cells, path_lines);
}

#[test]
fn path_format_composes_with_a_real_pipe() {
    // `xargs -n1 echo` proves the output is directly consumable, not just
    // shaped like it — the recipe the spec exists for.
    let dir = scaffold();
    let mt_out = support::mt(dir.path())
        .args(["ls", "", "--format", "path"])
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
