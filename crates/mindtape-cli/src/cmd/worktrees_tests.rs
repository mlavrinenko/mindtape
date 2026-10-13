//! Tests for `mt worktrees`: the `@name` sugar rewrite, the registry loader,
//! and the subprocess-verified stdout/env-isolation paths.
#![allow(clippy::unwrap_used)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

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
fn stub(token: &str) -> crate::diag::Result<PathBuf> {
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
    let fail = |t: &str| -> crate::diag::Result<PathBuf> {
        Err(crate::diag::Diag::failed("nope").value(t))
    };
    let err = desugar_with(argv(&["mt", "@bad", "ls"]), fail).unwrap_err();
    assert!(err.to_string().contains("nope: 'bad'"));
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

fn registry_with_sn() -> mindtape::WorktreeRegistry {
    mindtape::WorktreeRegistry::parse("[[worktrees]]\nname = \"sn\"\npath = \"/wt/sn\"\n").unwrap()
}

#[test]
fn unknown_worktree_shows_aliases_when_nonempty() {
    let err = super::unknown_worktree("bad", &registry_with_sn(), None);
    assert_eq!(
        crate::diag::plain(&err),
        "error: unknown worktree: '@bad'\n\
         hint: add it to worktrees.toml\n\
         note: known worktrees: sn\n\
         note: no app config directory resolves\n"
    );
}

#[test]
fn unknown_worktree_omits_aliases_when_empty() {
    let registry = mindtape::WorktreeRegistry::default();
    let err = super::unknown_worktree("bad", &registry, Some(Path::new("/c/worktrees.toml")));
    assert_eq!(
        crate::diag::plain(&err),
        "error: unknown worktree: '@bad'\n\
         hint: add it to worktrees.toml\n\
         note: worktrees are configured in: '/c/worktrees.toml'\n"
    );
}

#[test]
fn unknown_worktree_suggests_a_near_miss() {
    let err = super::unknown_worktree("sm", &registry_with_sn(), None);
    let msg = crate::diag::plain(&err);
    assert!(msg.contains("hint: did you mean @sn?"), "{msg}");
}

#[test]
fn unknown_worktree_omits_suggestion_on_a_far_miss() {
    let err = super::unknown_worktree("zzzzzzzz", &registry_with_sn(), None);
    let msg = crate::diag::plain(&err);
    assert!(!msg.contains("did you mean"), "{msg}");
}

// ===== Subprocess tests (stdout / env isolation) =====
//
// Each test spawns itself as a child process with `$MINDTAPE_CONFIG_HOME`
// pointing at a temp directory and a sentinel env var set. The child
// branch runs the real function; the parent branch asserts the child's
// exit status and, for stdout tests, the captured output.

const CHILD: &str = "_MT_WT_CHILD";

/// [`mindtape::test_support::rerun_isolated_with`] under a temp
/// `$MINDTAPE_CONFIG_HOME` holding `files` (`rel`, `contents`).
fn run_child(files: &[(&str, &str)]) -> Option<(tempfile::TempDir, std::process::Output)> {
    if std::env::var(CHILD).is_ok() {
        return None;
    }
    let dir = mindtape::test_support::tempdir_with_files(files);
    let out = mindtape::test_support::rerun_isolated_with(CHILD, |child| {
        child.env(mindtape::CONFIG_HOME_ENV, dir.path());
    })?;
    Some((dir, out))
}

/// `config` as the worktree registry, `mindtape/worktrees.toml`.
fn registry(config: &str) -> [(&str, &str); 1] {
    [("mindtape/worktrees.toml", config)]
}

const ALIAS_CFG: &str = "[[worktrees]]\nname = \"sn\"\npath = \"/wt/sn\"\n";

// --- load_registry (Default mutants; guard mutants) ---

#[test]
fn load_registry_missing_file_returns_empty_and_path() {
    if run_child(&[]).is_some() {
        return;
    }
    let (registry, path) = super::load_registry().unwrap();
    assert!(registry.is_empty(), "missing file → empty registry");
    let p = path.expect("path should be Some when config dir resolves");
    assert!(p.ends_with("mindtape/worktrees.toml"), "{p:?}");
}

#[test]
fn load_registry_valid_file_returns_entries() {
    if run_child(&registry(ALIAS_CFG)).is_some() {
        return;
    }
    let (registry, path) = super::load_registry().unwrap();
    assert!(!registry.is_empty(), "valid file → non-empty registry");
    assert!(path.is_some(), "path should be present");
    assert_eq!(registry.alias_names(), vec!["sn"]);
}

#[test]
fn load_registry_non_notfound_error_propagates() {
    if run_child(&[("mindtape", "i am a file not a dir")]).is_some() {
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
    if run_child(&registry(ALIAS_CFG)).is_some() {
        return;
    }
    let path = super::resolve_token("sn").unwrap();
    assert_eq!(path, PathBuf::from("/wt/sn"));
}

// --- run (Ok(())) + print_json (()) ---

#[test]
fn run_with_json_prints_output() {
    if let Some((_dir, out)) = run_child(&registry(ALIAS_CFG)) {
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
    if run_child(&registry("not valid toml {{{")).is_some() {
        return;
    }
    let result = super::run(&super::WorktreesArgs { json: false });
    assert!(result.is_err(), "malformed config should cause error");
}

#[test]
fn run_without_config_names_the_config_dir() {
    if let Some((_dir, out)) = run_child(&[]) {
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
    if let Some(out) = mindtape::test_support::rerun_isolated_with(CHILD, |child| {
        child.env(mindtape::CONFIG_HOME_ENV, "");
    }) {
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
    let cfg = "[[worktrees]]\nname = \"sn\"\npath = \"/wt/sn\"\n\
                   [[worktrees]]\nbase = \"/wt\"\n";
    if let Some((_dir, out)) = run_child(&registry(cfg)) {
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
