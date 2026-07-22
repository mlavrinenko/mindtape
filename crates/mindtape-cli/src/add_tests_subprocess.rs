//! Every `add` test that needs its own process: the stdin-title fill, the
//! `--no-verify` guard in `run()`, and the outside-roots warning.
//!
//! Each execs the test binary itself as a child to get a controllable stdin,
//! cwd, or capturable stderr — none of which can be asserted from inside the
//! shared test process. Pure-helper tests live in `add_tests.rs`.

use super::super::{fill_title_from_stdin, run, warn_if_outside_roots};
use super::*;

// ── fill_title_from_stdin tests (survivors: body→Ok(||), ||→&&, ==→!=) ──

/// Environment variable that gates the inner side of `fill_title_from_stdin` tests.
const FILL_STDIN_INNER: &str = "_MT_ADD_FILL_STDIN";

/// Spawn an inner test binary instance that reads `input` from its piped stdin.
fn run_fill_stdin_inner(name: &str, input: &[u8]) {
    use std::io::Write;
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .env(FILL_STDIN_INNER, "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .args([name, "--nocapture"])
        .spawn()
        .expect("spawn inner test");
    // A scenario that returns early (pre-set title) never reads stdin, so the
    // child can exit before this write lands, closing the pipe. That BrokenPipe
    // is not a failure — the child's exit status below is the real assertion.
    match child.stdin.take().expect("piped stdin").write_all(input) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
        Err(e) => panic!("write stdin: {e:?}"),
    }
    let output = child.wait_with_output().expect("wait inner test");
    assert!(
        output.status.success(),
        "inner test '{name}' failed:\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Default `AddArgs` for `fill_title_from_stdin` tests (no title, no values).
fn fill_stdin_base_args() -> AddArgs {
    bare_args()
}

/// Pins replace `fill_title_from_stdin` -> Result<()> with Ok(())
///
/// The mutated function is a no-op and never reads stdin, so a piped title
/// would not be stored. The real function reads stdin and sets the title.
#[test]
fn fill_stdin_reads_piped_title() {
    if std::env::var(FILL_STDIN_INNER).is_ok() {
        let mut values = BTreeMap::new();
        fill_title_from_stdin(&mut values, &fill_stdin_base_args()).unwrap();
        assert_eq!(
            values.get("title"),
            Some(&RawValue::Str("piped title".to_owned())),
        );
        return;
    }
    run_fill_stdin_inner("fill_stdin_reads_piped_title", b"piped title\n");
}

/// Pins replace || with && (second guard,
/// `std::io::stdin().is_terminal()`).
///
/// Original:  A || B || C   (returns early when title already set)
/// Mutant:    A && B || C   (only returns early when title AND stdin-terminal)
///
/// When title is pre-set but stdin is a pipe (`is_terminal()` = false),
/// the original returns early (A=true); the mutant computes
/// (A && B) || C = (true && false) || false = false and proceeds to read
/// stdin, overwriting the pre-set title.
#[test]
fn fill_stdin_existing_title_is_not_overwritten() {
    if std::env::var(FILL_STDIN_INNER).is_ok() {
        let mut values = BTreeMap::new();
        values.insert("title".to_owned(), RawValue::Str("pre-set".to_owned()));
        fill_title_from_stdin(&mut values, &fill_stdin_base_args()).unwrap();
        assert_eq!(
            values.get("title"),
            Some(&RawValue::Str("pre-set".to_owned())),
        );
        return;
    }
    run_fill_stdin_inner(
        "fill_stdin_existing_title_is_not_overwritten",
        b"overwrite\n",
    );
}

/// Pins replace || with && (third guard,
/// `args.values.iter().any(|source| source == "-")`).
///
/// Original:  A || B || C   (returns early when "-" is in values)
/// Mutant:    A || B && C   (only returns early when BOTH stdin-terminal
///                           AND "-" in values)
///
/// When stdin is a pipe (`is_terminal()` = false) and "-" is in values,
/// the original returns early (C=true); the mutant computes
/// A || (B && C) = false || (false && true) = false and proceeds to
/// read stdin, setting the title even though stdin is claimed for JSON.
#[test]
fn fill_stdin_skips_when_dash_in_values() {
    if std::env::var(FILL_STDIN_INNER).is_ok() {
        let mut values = BTreeMap::new();
        let mut args = fill_stdin_base_args();
        args.values = vec!["-".to_owned()];
        fill_title_from_stdin(&mut values, &args).unwrap();
        assert!(!values.contains_key("title"));
        return;
    }
    run_fill_stdin_inner("fill_stdin_skips_when_dash_in_values", b"piped title\n");
}

/// Pins replace == with != in
/// `args.values.iter().any(|source| source == "-")`.
///
/// Original:  any(source == "-") — false for non-"-" entries
/// Mutant:    any(source != "-") — true for non-"-" entries, causing
///                                 an unconditional early return
///
/// When values have no "-" entry, the original proceeds to read stdin
/// and sets the title; the mutant returns early and leaves the title unset.
#[test]
fn fill_stdin_reads_when_values_have_non_dash() {
    if std::env::var(FILL_STDIN_INNER).is_ok() {
        let mut values = BTreeMap::new();
        let mut args = fill_stdin_base_args();
        args.values = vec!["data.json".to_owned()];
        fill_title_from_stdin(&mut values, &args).unwrap();
        assert_eq!(
            values.get("title"),
            Some(&RawValue::Str("piped title".to_owned())),
        );
        return;
    }
    run_fill_stdin_inner(
        "fill_stdin_reads_when_values_have_non_dash",
        b"piped title\n",
    );
}

// ── run() no_verify guard test (survivor: delete !) ──

/// Environment variable that gates the inner side of the `run`/`no_verify` test.
const RUN_NO_VERIFY_INNER: &str = "_MT_ADD_RUN_NO_VERIFY";

/// In-tree prelude for the @local/mindtape package override.
const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

/// A template with a depends-on link to a non-existent task: the file passes
/// fill and write but fails post-write validation. When `--no-verify` is set,
/// the original `run()` skips validation, but the `delete !` mutant would run
/// it and error.
const BAD_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: slot("title"),
  links: depends-on("nonexistent.typ"),
)
"#;

/// Pins delete ! in `if !args.no_verify`.
///
/// Original:  `if !args.no_verify { verify_written(...)? }`
/// Mutant:    `if args.no_verify { verify_written(...)? }` — inverts the guard.
///
/// With --no-verify, the original skips post-write validation (and succeeds).
/// The mutant runs it on a task that has an unresolved depends-on link,
/// which fails validation.
#[test]
fn run_with_no_verify_skips_validation() {
    if std::env::var(RUN_NO_VERIFY_INNER).is_ok() {
        let root_dir = std::env::var(RUN_NO_VERIFY_INNER).unwrap();
        let root = Path::new(&root_dir);
        std::env::set_current_dir(root).unwrap();
        let args = AddArgs {
            title: vec![],
            template: None,
            values: vec![],
            set: vec!["title=Test".to_owned()],
            into: ".".to_owned(),
            to: None,
            name: Some("no-verify-test".to_owned()),
            status: None,
            edit: false,
            describe: None,
            list: false,
            print: false,
            no_verify: true,
        };
        let result = run(&args);
        assert!(
            result.is_ok(),
            "run with --no-verify should succeed: {result:?}",
        );
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    // Project config with a single tasks root.
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    // Custom template that produces a task with an unresolvable depends-on.
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        BAD_TEMPLATE,
    )
    .unwrap();
    // Install the @local/mindtape lib so the task can be evaluated.
    std::fs::create_dir_all(dir.path().join("lib")).unwrap();
    std::fs::write(dir.path().join("lib/prelude.typ"), PRELUDE).unwrap();
    std::fs::write(
        dir.path().join("lib/typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("tasks")).unwrap();

    let path = dir.path().to_str().unwrap().to_owned();
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .env(RUN_NO_VERIFY_INNER, &path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .args(["run_with_no_verify_skips_validation", "--nocapture"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "inner test failed:\nstderr: {}",
        String::from_utf8_lossy(&out.stderr),
    );
}

// ── warn_if_outside_roots tests ──

/// Env var gating the inner/outer split for stderr-capturing warn tests.
const WARN_INNER: &str = "_MT_WARN_INNER";

/// shared fixture for the `warn_if_outside_roots` inner/outer tests below.
fn warn_scenario_project(dir: &Path) -> (PathBuf, Project) {
    let root = dir.join("tasks");
    std::fs::create_dir_all(&root).unwrap();
    (root, project_with_root(dir, "tasks"))
}

#[test]
fn warn_if_outside_roots_outside_prints_warning() {
    if std::env::var(WARN_INNER).is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let (_root, project) = warn_scenario_project(dir.path());
        let dest = dir.path().join("outside");
        std::fs::create_dir_all(&dest).unwrap();
        warn_if_outside_roots(&project, &dest);
        return;
    }
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .env(WARN_INNER, "1")
        .args([
            "warn_if_outside_roots_outside_prints_warning",
            "--nocapture",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "inner test failed: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("warning:"),
        "expected warning, stderr: {stderr}",
    );
}

#[test]
fn warn_if_outside_roots_inside_no_warning() {
    if std::env::var(WARN_INNER).is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let (root, project) = warn_scenario_project(dir.path());
        let dest = root.join("sub");
        std::fs::create_dir_all(&dest).unwrap();
        warn_if_outside_roots(&project, &dest);
        return;
    }
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .env(WARN_INNER, "1")
        .args(["warn_if_outside_roots_inside_no_warning", "--nocapture"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "inner test failed: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("warning:"),
        "unexpected warning, stderr: {stderr}",
    );
}
