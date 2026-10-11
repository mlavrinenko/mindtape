//! E2E test for `mt _cli`: runs the compiled binary in a subprocess.
//!
//! The only thing this can show that a unit test cannot is that `run` actually
//! writes the dump to stdout — `replace run with Ok(())` is invisible in
//! process, because the side effect is a `println!`. The dump's *contents*
//! (the `schema` key, the `bin` name) are asserted in `cli/tree_json.rs` against
//! `dump_json()` directly, so they are deliberately not re-asserted here.
#![allow(clippy::unwrap_used)]

use crate::support;

#[test]
fn cli_dump_prints_the_json_to_stdout() {
    // `_cli` reads no config, but it still runs through `support::mt` (in a
    // throwaway directory) so no e2e spawn anywhere is the un-isolated one.
    let dir = tempfile::tempdir().unwrap();
    let output = support::mt(dir.path())
        .arg("_cli")
        .arg("--json")
        .output()
        .unwrap();

    assert!(output.status.success());
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("stdout must carry the JSON dump, not nothing");
}

/// Run `mt _cli --check-argv <line>` with the status verbs `done` and `wip`
/// declared, returning whether `mt` accepted the line and what it said.
fn check_argv(line: &str) -> (bool, String) {
    let dir = tempfile::tempdir().unwrap();
    let output = support::mt(dir.path())
        .args(["_cli", "--statuses", "done,wip", "--check-argv", line])
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status.success(), said)
}

#[test]
fn check_argv_names_the_closest_flag_through_the_worktree_sugar() {
    let (ok, said) = check_argv("mt @name ls --jsn");
    assert!(!ok);
    assert!(
        said.contains("mt ls has no flag --jsn; closest: --json"),
        "{said}"
    );
}

#[test]
fn check_argv_names_the_closest_subcommand() {
    let (ok, said) = check_argv("mt config shw");
    assert!(!ok);
    assert!(
        said.contains("mt config has no subcommand shw; closest: show"),
        "{said}"
    );
    let (ok, said) = check_argv("mt lss");
    assert!(!ok);
    assert!(said.contains("no subcommand lss; closest: ls"), "{said}");
}

#[test]
fn check_argv_accepts_a_status_verb_and_a_missing_required_argument() {
    for line in [
        "mt done x -m \"why\"",
        "mt done",
        "mt ls --sort",
        "mt ls 'status != \"done\"' --sort priority:desc -n 5",
        "mt <cmd> --help",
        "mt ls --skip <next_skip>",
        "mt config <sub>",
    ] {
        let (ok, said) = check_argv(line);
        assert!(ok, "{line}: {said}");
    }
}

#[test]
fn check_argv_refuses_a_flag_a_status_verb_lacks() {
    let (ok, said) = check_argv("mt done x --nope");
    assert!(!ok);
    assert!(said.contains("--nope"), "{said}");
}
