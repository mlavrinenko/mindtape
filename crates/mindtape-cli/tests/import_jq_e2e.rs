//! End-to-end tests for the jq source adapters (`contrib/jq/*.jq`,
//! `truth/tasks/jq-source-adapters.typ`): shells `jq -f` on a fixture (real
//! subprocess stdin/stdout, same reasoning `import_e2e.rs` gives for
//! shelling `mt` itself), then pipes the adapter's output into `mt import`
//! and validates the imported project with `mt check`.
//!
//! Requires `jq` on `PATH` — added to the flake devShell specifically for
//! this coverage (`flake.nix`'s `devShells.default`). A missing/failing `jq`
//! panics with a clear message rather than silently skipping: this test
//! exists to prove the adapters work, not to be a no-op in an
//! under-provisioned environment.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod support;

/// The repo root, derived from this crate's manifest dir
/// (`crates/mindtape-cli` -> `../..`), so the test finds `contrib/jq/*`
/// regardless of the working directory `cargo test` runs from.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Run `jq -f <script> <input>`, returning stdout. Panics (never silently
/// skips) when `jq` is missing from `PATH` or exits non-zero.
fn run_jq(script: &Path, input: &Path) -> String {
    let output = Command::new("jq")
        .arg("-f")
        .arg(script)
        .arg(input)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "`jq` not runnable (expected on PATH via the flake devShell): {err}; \
                 script={}, input={}",
                script.display(),
                input.display()
            )
        });
    assert!(
        output.status.success(),
        "jq -f {} {} failed:\n{}",
        script.display(),
        input.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("jq stdout is valid UTF-8")
}

/// Assert `jq -f contrib/jq/<source>.jq contrib/jq/fixtures/<source>.in.json`
/// is structurally equal (`serde_json::Value` equality — key order never
/// matters) to `contrib/jq/fixtures/<source>.out.json`, the golden.
fn assert_golden(source: &str) {
    let jq_dir = repo_root().join("contrib/jq");
    let script = jq_dir.join(format!("{source}.jq"));
    let input = jq_dir.join("fixtures").join(format!("{source}.in.json"));
    let expected_path = jq_dir.join("fixtures").join(format!("{source}.out.json"));

    let actual: serde_json::Value =
        serde_json::from_str(&run_jq(&script, &input)).expect("jq stdout is valid JSON");
    let expected: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display())),
    )
    .expect("golden fixture is valid JSON");
    assert_eq!(
        actual, expected,
        "{source}: `jq -f {source}.jq` output does not match {source}.out.json"
    );
}

/// A minimal project: a `tasks/` root and a status legend covering every
/// canonical status name the two adapters emit (`todo`, `done`,
/// `cancelled`) — priority/tags/kind are left at the embedded config
/// floor's defaults (0..9 scale, no framework, unconstrained tags), which
/// both adapters' fixture values already satisfy.
fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n\
         [[tasks.status]]\nvalue = \"todo\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
         [[tasks.status]]\nvalue = \"cancelled\"\nterminal = true\n",
    )
    .unwrap();
    support::install_lib(dir);
}

/// Pipe `jq -f <source>.jq <source>.in.json`'s stdout into `mt import`,
/// then run `mt check` — the full "foreign export becomes valid
/// `MindTape` tasks" pipeline for one source.
fn assert_e2e_import(source: &str) {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    let jq_dir = repo_root().join("contrib/jq");
    let script = jq_dir.join(format!("{source}.jq"));
    let input = jq_dir.join("fixtures").join(format!("{source}.in.json"));
    let canonical = run_jq(&script, &input);

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("import")
        .write_stdin(canonical)
        .assert()
        .success();

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();

    let filed = std::fs::read_dir(dir.path().join("tasks"))
        .unwrap()
        .flatten()
        .count();
    assert!(filed > 0, "{source}: mt import filed nothing");
}

#[test]
fn taskwarrior_jq_matches_golden() {
    assert_golden("taskwarrior");
}

#[test]
fn github_jq_matches_golden() {
    assert_golden("github");
}

#[test]
fn taskwarrior_jq_imports_and_checks() {
    assert_e2e_import("taskwarrior");
}

#[test]
fn github_jq_imports_and_checks() {
    assert_e2e_import("github");
}
