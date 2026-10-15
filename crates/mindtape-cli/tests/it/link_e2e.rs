//! End-to-end tests for `mt link` over a real project and a real process.
//!
//! `cmd/link/mod.rs`'s inline tests drive `run_with` against an in-process
//! project, which covers the merge/dedupe/replace/note/unknown-kind matrix.
//! What they cannot reach is what needs a spawned binary: the stdin (`-`)
//! adjacency source, the exit status a per-entry failure carries out of the
//! process, and the fact that a written file still passes `mt check`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

fn task(title: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n)\n"
    )
}

fn scaffold() -> tempfile::TempDir {
    let dir = support::scaffold_project(CONFIG);
    for (stem, title) in [
        ("wire-oauth", "Wire OAuth"),
        ("pick-a-provider", "Pick a provider"),
        ("write-docs", "Write docs"),
    ] {
        std::fs::write(
            dir.path().join("tasks").join(format!("{stem}.typ")),
            task(title),
        )
        .unwrap();
    }
    dir
}

fn source(dir: &Path, stem: &str) -> String {
    std::fs::read_to_string(dir.join("tasks").join(format!("{stem}.typ"))).unwrap()
}

/// An edge that would make the linking task invalid is refused before the
/// write, so the file is byte-identical afterwards. `depends-on` is acyclic,
/// and a self-edge is the shortest cycle there is — a rule the planner does not
/// check and only the whole-project validator can answer.
#[test]
fn a_rejected_edge_leaves_the_file_untouched() {
    let dir = scaffold();
    let before = source(dir.path(), "wire-oauth");

    let output = support::mt(dir.path())
        .args(["link", "wire-oauth", "depends-on", "wire-oauth"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "a rejected edge must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("error: ") && stderr.contains("depends-on cycle"),
        "a blocking problem reports as an error: {stderr}"
    );

    assert_eq!(
        source(dir.path(), "wire-oauth"),
        before,
        "a refused edge must leave the file byte-identical"
    );
    support::check_ok(dir.path());
}

#[test]
fn a_written_edge_leaves_the_project_valid() {
    let dir = scaffold();

    support::run_ok(
        dir.path(),
        &["link", "wire-oauth", "depends-on", "pick-a-provider"],
    );

    let written = source(dir.path(), "wire-oauth");
    assert!(
        written.contains(r#"depends-on("pick-a-provider.typ")"#),
        "{written}"
    );
    // The point of serializing through the prelude's own helpers: the file
    // parses, evaluates, and satisfies every rule `mt check` enforces.
    support::check_ok(dir.path());
}

#[test]
fn an_adjacency_map_arrives_on_stdin() {
    let dir = scaffold();

    support::mt(dir.path())
        .args(["link", "--apply", "-"])
        .write_stdin(
            "wire-oauth:\n  depends-on: [pick-a-provider]\n  \
             related:\n    - target: write-docs\n      note: same release\n",
        )
        .assert()
        .success();

    let written = source(dir.path(), "wire-oauth");
    assert!(
        written.contains(r#"depends-on("pick-a-provider.typ")"#),
        "{written}"
    );
    assert!(
        written.contains(r#"related("write-docs.typ")[same release]"#),
        "{written}"
    );
    support::check_ok(dir.path());
}

#[test]
fn a_json_adjacency_map_reads_the_same_way() {
    let dir = scaffold();

    support::mt(dir.path())
        .args(["link", "--apply", "-"])
        .write_stdin(r#"{"wire-oauth": {"depends-on": ["pick-a-provider"]}}"#)
        .assert()
        .success();
    assert!(
        source(dir.path(), "wire-oauth").contains(r#"depends-on("pick-a-provider.typ")"#),
        "{}",
        source(dir.path(), "wire-oauth")
    );
}

#[test]
fn re_applying_the_same_map_writes_nothing() {
    let dir = scaffold();
    let map = "wire-oauth:\n  depends-on: [pick-a-provider, write-docs]\n";

    support::mt(dir.path())
        .args(["link", "--apply", "-"])
        .write_stdin(map)
        .assert()
        .success();
    let first = source(dir.path(), "wire-oauth");

    let output = support::mt(dir.path())
        .args(["link", "--apply", "-"])
        .write_stdin(map)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout).into_owned();
    assert!(stdout.contains("unchanged wire-oauth"), "{stdout}");
    assert_eq!(source(dir.path(), "wire-oauth"), first);
}

#[test]
fn dry_run_prints_the_plan_and_writes_nothing() {
    let dir = scaffold();
    let before = source(dir.path(), "wire-oauth");

    let output = support::mt(dir.path())
        .args([
            "link",
            "wire-oauth",
            "depends-on",
            "pick-a-provider",
            "--dry-run",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout).into_owned();
    assert!(stdout.contains("would rewrite 1 task:"), "{stdout}");
    assert!(
        stdout.contains("+depends-on -> pick-a-provider.typ"),
        "{stdout}"
    );
    assert!(stdout.contains("dry run: nothing written"), "{stdout}");
    assert_eq!(source(dir.path(), "wire-oauth"), before);
}

#[test]
fn an_unknown_kind_exits_non_zero_naming_the_configured_kinds() {
    let dir = scaffold();

    let output = support::mt(dir.path())
        .args(["link", "wire-oauth", "depends-onn", "write-docs"])
        .assert()
        .failure();
    let stderr = support::stderr_of(&output);
    assert!(
        stderr.contains("unknown link kind: 'depends-onn'"),
        "{stderr}"
    );
    assert!(
        stderr.contains("hint: did you mean depends-on?"),
        "{stderr}"
    );
    assert!(!source(dir.path(), "wire-oauth").contains("links:"));
}

#[test]
fn a_missing_target_exits_non_zero() {
    let dir = scaffold();

    let output = support::mt(dir.path())
        .args(["link", "wire-oauth", "depends-on", "no-such-task"])
        .assert()
        .failure();
    let stderr = support::stderr_of(&output);
    assert!(
        stderr.contains("no task: 'no-such-task' of kind 'depends-on'"),
        "{stderr}"
    );
    assert!(!source(dir.path(), "wire-oauth").contains("links:"));
}

/// The rm-style contract, proven across the process boundary: the run exits
/// non-zero for the entry it refused, and the entries it accepted are on disk.
#[test]
fn a_failing_entry_still_lets_the_rest_land() {
    let dir = scaffold();

    support::mt(dir.path())
        .args(["link", "--apply", "-"])
        .write_stdin(
            "wire-oauth:\n  related: [write-docs]\n\
             write-docs:\n  depends-on: [pick-a-provider]\n",
        )
        .assert()
        .failure();

    // `related` with no note was refused; the unrelated entry still landed.
    assert!(!source(dir.path(), "wire-oauth").contains("links:"));
    assert!(
        source(dir.path(), "write-docs").contains(r#"depends-on("pick-a-provider.typ")"#),
        "{}",
        source(dir.path(), "write-docs")
    );
}

#[test]
fn replace_drops_the_edges_the_map_omits() {
    let dir = scaffold();

    support::run_ok(
        dir.path(),
        &["link", "wire-oauth", "depends-on", "pick-a-provider"],
    );
    support::mt(dir.path())
        .args(["link", "--apply", "-", "--replace"])
        .write_stdin("wire-oauth:\n  depends-on: [write-docs]\n")
        .assert()
        .success();

    let written = source(dir.path(), "wire-oauth");
    assert!(
        written.contains(r#"depends-on("write-docs.typ")"#),
        "{written}"
    );
    assert!(!written.contains("pick-a-provider"), "{written}");
    support::check_ok(dir.path());
}

/// Closing a cycle is refused whichever end the validator attributes it to:
/// `mt check` names the cycle on its first member, which need not be the task
/// being written, so the refusal cannot rest on the writer's own violations.
#[test]
fn closing_a_dependency_cycle_is_refused_from_either_end() {
    for (first, second) in [
        ("wire-oauth", "pick-a-provider"),
        ("pick-a-provider", "wire-oauth"),
    ] {
        let dir = scaffold();
        support::run_ok(dir.path(), &["link", first, "depends-on", second]);
        let before = (source(dir.path(), first), source(dir.path(), second));

        let output = support::mt(dir.path())
            .args(["link", second, "depends-on", first])
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{second} -> {first}: {stderr}");
        assert!(stderr.contains("depends-on cycle"), "{stderr}");
        assert_eq!(
            (source(dir.path(), first), source(dir.path(), second)),
            before,
            "a refused cycle must leave both files byte-identical"
        );
        support::check_ok(dir.path());
    }
}

/// A cycle already on disk is not the write's doing, so it never blocks an
/// edge elsewhere.
#[test]
fn a_cycle_already_on_disk_does_not_block_an_unrelated_edge() {
    let dir = scaffold();
    for (stem, target) in [
        ("wire-oauth", "pick-a-provider"),
        ("pick-a-provider", "wire-oauth"),
    ] {
        let text = source(dir.path(), stem).replace(
            "status: wip(2026, 6, 1),",
            &format!("status: wip(2026, 6, 1),\n  links: depends-on(\"{target}.typ\"),"),
        );
        std::fs::write(dir.path().join("tasks").join(format!("{stem}.typ")), text).unwrap();
    }
    support::mt(dir.path()).arg("check").assert().failure();

    support::run_ok(
        dir.path(),
        &["link", "write-docs", "depends-on", "wire-oauth"],
    );
}

#[test]
fn a_dry_run_lists_an_edge_already_on_disk_as_unchanged() {
    let dir = scaffold();
    support::run_ok(
        dir.path(),
        &["link", "wire-oauth", "depends-on", "pick-a-provider"],
    );

    let stdout = support::stdout(
        dir.path(),
        &[
            "link",
            "wire-oauth",
            "depends-on",
            "pick-a-provider",
            "--dry-run",
        ],
    );
    assert!(stdout.contains("would rewrite 0 tasks:"), "{stdout}");
    assert!(stdout.contains("- wire-oauth: unchanged"), "{stdout}");
}

/// A write the OS refuses is reported as its own failure, and the run exits
/// non-zero. The tasks folder is read-only, so the atomic rewrite cannot make
/// its temporary file; the task itself must come through unchanged.
#[cfg(unix)]
#[test]
fn a_refused_write_is_reported_and_leaves_the_task_as_it_was() {
    use std::os::unix::fs::PermissionsExt;

    let dir = scaffold();
    let before = source(dir.path(), "wire-oauth");
    let tasks = dir.path().join("tasks");
    std::fs::set_permissions(&tasks, std::fs::Permissions::from_mode(0o555)).unwrap();

    let output = support::mt(dir.path())
        .args(["link", "wire-oauth", "depends-on", "pick-a-provider"])
        .assert()
        .failure();
    std::fs::set_permissions(&tasks, std::fs::Permissions::from_mode(0o755)).unwrap();

    let stderr = support::stderr_of(&output);
    assert!(stderr.contains("can't write"), "{stderr}");
    assert_eq!(source(dir.path(), "wire-oauth"), before);
}

/// An edge cannot be written into a task whose show rule is aliased: there is
/// no literal `task.with(…)` call to hold the `links:` slot. The entry is
/// refused as a whole, the diagnostic names the file, and the file stays put.
#[test]
fn an_edge_into_an_aliased_task_is_refused_and_leaves_it_untouched() {
    let dir = scaffold();
    let body = support::write_aliased_task(dir.path(), "wire-oauth");

    let (code, stderr) = support::run_stderr(
        dir.path(),
        &["link", "wire-oauth", "depends-on", "pick-a-provider"],
    );
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("no task.with(…) call found"), "{stderr}");
    assert!(stderr.contains("wire-oauth.typ"), "{stderr}");
    assert_eq!(source(dir.path(), "wire-oauth"), body);
}
