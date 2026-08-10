//! Tests for `mt worktrees`: the `@name` sugar rewrite, the registry loader,
//! and the subprocess-verified stdout/env-isolation paths.
#![allow(clippy::unwrap_used)]

use std::ffi::OsString;
use std::path::PathBuf;

use super::desugar_with;

fn argv(parts: &[&str]) -> Vec<OsString> {
    parts.iter().map(OsString::from).collect()
}

fn strs(argv: &[OsString]) -> Vec<String> {
    argv.iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

/// Stub resolver: any token maps to `/wt/<token>`. Mirrors the injected
/// `resolve` signature, so the `Result` wrapper is deliberate.
#[allow(clippy::unnecessary_wraps)]
fn stub(token: &str) -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(format!("/wt/{token}")))
}

#[test]
fn leading_at_rewrites_to_worktree_flag() {
    let out = desugar_with(argv(&["mt", "@sn", "ls"]), stub).unwrap();
    assert_eq!(strs(&out), vec!["mt", "-w", "/wt/sn", "ls"]);
}

#[test]
fn leading_at_with_no_subcommand_still_rewrites() {
    let out = desugar_with(argv(&["mt", "@sn"]), stub).unwrap();
    assert_eq!(strs(&out), vec!["mt", "-w", "/wt/sn"]);
}

#[test]
fn non_sugar_first_arg_is_untouched() {
    let out = desugar_with(argv(&["mt", "ls", "@view"]), stub).unwrap();
    assert_eq!(strs(&out), vec!["mt", "ls", "@view"]);
}

#[test]
fn at_only_after_subcommand_is_untouched() {
    // The view sigil in filter position must survive.
    let out = desugar_with(argv(&["mt", "ls", "@broken", "--json"]), stub).unwrap();
    assert_eq!(strs(&out), vec!["mt", "ls", "@broken", "--json"]);
}

#[test]
fn empty_argv_stays_empty() {
    assert!(desugar_with(Vec::new(), stub).unwrap().is_empty());
}

#[test]
fn resolution_failure_propagates() {
    let fail = |t: &str| -> anyhow::Result<PathBuf> { Err(anyhow::anyhow!("nope: {t}")) };
    let err = desugar_with(argv(&["mt", "@bad", "ls"]), fail).unwrap_err();
    assert!(err.to_string().contains("nope: bad"));
}

// ===== Regression tests for surviving cargo-mutants =====

// --- is_project (true / false) ---

#[test]
fn is_project_true_when_dot_mindtape_exists() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join(".mindtape")).unwrap();
    assert!(super::is_project(dir.path()));
}

#[test]
fn is_project_false_without_dot_mindtape() {
    let dir = tempfile::TempDir::new().unwrap();
    assert!(!super::is_project(dir.path()));
}

// --- worktrees_path (None / Some(Default::default())) ---

#[test]
fn worktrees_path_is_some_with_correct_suffix() {
    let path = super::worktrees_path();
    let p = path.as_ref().expect("worktrees_path should be Some");
    assert!(
        p.ends_with("mindtape/worktrees.toml"),
        "expected .../mindtape/worktrees.toml, got {p:?}",
    );
}

// --- unknown_worktree (delete !) ---

#[test]
fn unknown_worktree_shows_aliases_when_nonempty() {
    let registry =
        mindtape::WorktreeRegistry::parse("[[worktrees]]\nname = \"sn\"\npath = \"/wt/sn\"\n")
            .unwrap();
    let err = super::unknown_worktree("bad", &registry, None);
    let msg = err.to_string();
    assert!(msg.contains("known aliases: sn"), "{msg}");
}

#[test]
fn unknown_worktree_omits_aliases_when_empty() {
    let registry = mindtape::WorktreeRegistry::default();
    let err = super::unknown_worktree("bad", &registry, None);
    let msg = err.to_string();
    assert!(!msg.contains("known aliases"), "{msg}");
}

// ===== Subprocess tests (stdout / env isolation) =====
//
// Each test spawns itself as a child process with `$MINDTAPE_CONFIG_HOME`
// pointing at a temp directory and a sentinel env var set. The child
// branch runs the real function; the parent branch asserts the child's
// exit status and, for stdout tests, the captured output.

const CHILD: &str = "_MT_WT_CHILD";

/// Spawn `test_name` as a child, optionally writing `config` to
/// `mindtape/worktrees.toml` under a temp `$MINDTAPE_CONFIG_HOME`.
fn run_child(test_name: &str, config: Option<&str>) -> (tempfile::TempDir, std::process::Output) {
    let dir = tempfile::TempDir::new().unwrap();
    if let Some(body) = config {
        let wt_dir = dir.path().join("mindtape");
        std::fs::create_dir_all(&wt_dir).unwrap();
        std::fs::write(wt_dir.join("worktrees.toml"), body).unwrap();
    }
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .env(mindtape::CONFIG_HOME_ENV, dir.path())
        .env(CHILD, "1")
        .args([test_name, "--nocapture"])
        .output()
        .unwrap();
    (dir, out)
}

fn assert_child_ok(out: &std::process::Output) {
    assert!(
        out.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

const ALIAS_CFG: &str = "[[worktrees]]\nname = \"sn\"\npath = \"/wt/sn\"\n";

// --- load_registry (Default mutants; guard mutants) ---

#[test]
fn load_registry_missing_file_returns_empty_and_path() {
    if std::env::var(CHILD).is_err() {
        let (_dir, out) = run_child("load_registry_missing_file_returns_empty_and_path", None);
        assert_child_ok(&out);
        return;
    }
    let (registry, path) = super::load_registry().unwrap();
    assert!(registry.is_empty(), "missing file → empty registry");
    let p = path.expect("path should be Some when config dir resolves");
    assert!(p.ends_with("mindtape/worktrees.toml"), "{p:?}");
}

#[test]
fn load_registry_valid_file_returns_entries() {
    if std::env::var(CHILD).is_err() {
        let (_dir, out) = run_child("load_registry_valid_file_returns_entries", Some(ALIAS_CFG));
        assert_child_ok(&out);
        return;
    }
    let (registry, path) = super::load_registry().unwrap();
    assert!(!registry.is_empty(), "valid file → non-empty registry");
    assert!(path.is_some(), "path should be present");
    assert_eq!(registry.alias_names(), vec!["sn"]);
}

#[test]
fn load_registry_non_notfound_error_propagates() {
    if std::env::var(CHILD).is_err() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("mindtape"), "i am a file not a dir").unwrap();
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .env(mindtape::CONFIG_HOME_ENV, dir.path())
            .env(CHILD, "1")
            .args(["load_registry_non_notfound_error_propagates", "--nocapture"])
            .output()
            .unwrap();
        assert_child_ok(&out);
        return;
    }
    let result = super::load_registry();
    assert!(
        result.is_err(),
        "non-NotFound IO error (ENOTDIR) should propagate"
    );
}

// --- resolve_token (Ok(Default::default())) ---

#[test]
fn resolve_token_resolves_alias_correctly() {
    if std::env::var(CHILD).is_err() {
        let (_dir, out) = run_child("resolve_token_resolves_alias_correctly", Some(ALIAS_CFG));
        assert_child_ok(&out);
        return;
    }
    let path = super::resolve_token("sn").unwrap();
    assert_eq!(path, PathBuf::from("/wt/sn"));
}

// --- run (Ok(())) + print_json (()) ---

#[test]
fn run_with_json_prints_output() {
    if std::env::var(CHILD).is_err() {
        let (_dir, out) = run_child("run_with_json_prints_output", Some(ALIAS_CFG));
        assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("\"kind\": \"alias\""),
            "missing alias in JSON output: {stdout}"
        );
        assert!(
            stdout.contains("\"name\": \"sn\""),
            "missing 'sn' in JSON output: {stdout}"
        );
        return;
    }
    let args = super::WorktreesArgs { json: true };
    super::run(&args).unwrap();
}

#[test]
fn run_with_malformed_config_errors() {
    if std::env::var(CHILD).is_err() {
        let (_dir, out) = run_child(
            "run_with_malformed_config_errors",
            Some("not valid toml {{{"),
        );
        assert_child_ok(&out);
        return;
    }
    let result = super::run(&super::WorktreesArgs { json: false });
    assert!(result.is_err(), "malformed config should cause error");
}

#[test]
fn run_without_config_names_the_config_dir() {
    if std::env::var(CHILD).is_err() {
        let (_dir, out) = run_child("run_without_config_names_the_config_dir", None);
        assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("no worktrees configured"),
            "missing empty-registry message: {stdout}"
        );
        assert!(
            stdout.contains("add [[worktrees]] entries to"),
            "missing config-dir hint: {stdout}"
        );
        return;
    }
    super::run(&super::WorktreesArgs { json: false }).unwrap();
}

#[test]
fn run_without_resolvable_config_home_prints_fallback() {
    if std::env::var(CHILD).is_err() {
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .env(mindtape::CONFIG_HOME_ENV, "")
            .env(CHILD, "1")
            .args([
                "run_without_resolvable_config_home_prints_fallback",
                "--nocapture",
            ])
            .output()
            .unwrap();
        assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("could not resolve the app config directory"),
            "missing fallback message: {stdout}"
        );
        return;
    }
    super::run(&super::WorktreesArgs { json: false }).unwrap();
}

#[test]
fn run_lists_aliases_and_bases() {
    if std::env::var(CHILD).is_err() {
        let cfg = "[[worktrees]]\nname = \"sn\"\npath = \"/wt/sn\"\n\
                   [[worktrees]]\nbase = \"/wt\"\n";
        let (_dir, out) = run_child("run_lists_aliases_and_bases", Some(cfg));
        assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("@sn → /wt/sn"),
            "missing alias line: {stdout}"
        );
        assert!(
            stdout.contains("@<name> → /wt/<name>"),
            "missing base line: {stdout}"
        );
        return;
    }
    super::run(&super::WorktreesArgs { json: false }).unwrap();
}
