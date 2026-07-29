//! `@name` worktree sugar and the `mt worktrees` command.
//!
//! The registry itself ([`mindtape::WorktreeRegistry`]) is pure; this module is
//! the IO and CLI edge: locate and read the app-scoped `worktrees.toml`, resolve
//! a leading `@token` into a path, and rewrite it to `-w <path>` before clap
//! parses — so `mt @sn ls` reuses the existing worktree-override plumbing and
//! every command honours it uniformly.
//!
//! Only the first post-program argument is treated as sugar; `@name` anywhere
//! else (the view sigil, `mt ls @view`) is left untouched, so the two `@` uses
//! stay disjoint by position.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use mindtape::{WorktreeEntry, WorktreeRegistry};

/// Arguments for `mt worktrees`.
#[derive(Debug, clap::Args)]
pub struct WorktreesArgs {
    /// Emit JSON instead of the plain listing.
    #[arg(long)]
    pub json: bool,
}

/// Run `mt worktrees`: list the configured aliases and bases.
///
/// # Errors
/// Returns an error if the config file exists but fails to parse.
pub fn run(args: &WorktreesArgs) -> Result<()> {
    let (registry, path) = load_registry()?;
    if args.json {
        print_json(&registry);
        return Ok(());
    }
    if registry.is_empty() {
        match &path {
            Some(p) => println!(
                "no worktrees configured (add [[worktrees]] entries to {})",
                p.display()
            ),
            None => {
                println!("no worktrees configured (could not resolve the app config directory)");
            }
        }
        return Ok(());
    }
    for entry in registry.entries() {
        match entry {
            WorktreeEntry::Alias { name, path } => println!("@{name} → {}", path.display()),
            WorktreeEntry::Base { base } => println!("@<name> → {}/<name>", base.display()),
        }
    }
    Ok(())
}

/// Emit the registry as a JSON array of `{kind, …}` objects.
fn print_json(registry: &WorktreeRegistry) {
    let items: Vec<serde_json::Value> = registry
        .entries()
        .iter()
        .map(|entry| match entry {
            WorktreeEntry::Alias { name, path } => serde_json::json!({
                "kind": "alias", "name": name, "path": path.display().to_string(),
            }),
            WorktreeEntry::Base { base } => serde_json::json!({
                "kind": "base", "base": base.display().to_string(),
            }),
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&items).unwrap_or_default()
    );
}

/// Rewrite a leading `@name` argument into `-w <path>` before clap parses.
///
/// # Errors
/// Returns an error if the leading `@token` cannot be resolved to a worktree.
pub fn desugar_argv(argv: Vec<OsString>) -> Result<Vec<OsString>> {
    desugar_with(argv, resolve_token)
}

/// The pure core of [`desugar_argv`]: `resolve` maps a token to a path, injected
/// so the rewrite is testable without touching the filesystem.
fn desugar_with<F>(argv: Vec<OsString>, resolve: F) -> Result<Vec<OsString>>
where
    F: Fn(&str) -> Result<PathBuf>,
{
    let mut iter = argv.into_iter();
    let Some(prog) = iter.next() else {
        return Ok(Vec::new());
    };
    let mut rest: Vec<OsString> = iter.collect();
    // Detach the token from `rest` before mutating it.
    let token = rest
        .first()
        .and_then(|arg| arg.to_str())
        .and_then(|s| s.strip_prefix('@'))
        .map(str::to_owned);
    let mut out = vec![prog];
    if let Some(token) = token {
        let path = resolve(&token)?;
        rest.remove(0);
        out.push(OsString::from("-w"));
        out.push(path.into_os_string());
    }
    out.extend(rest);
    Ok(out)
}

/// Resolve a `@token` (without the `@`) to a worktree path via the app-scoped
/// registry, erroring with the known aliases and config path on a miss.
fn resolve_token(token: &str) -> Result<PathBuf> {
    let (registry, path) = load_registry()?;
    match registry.resolve(token, dirs::home_dir().as_deref(), is_project)? {
        Some(resolved) => Ok(resolved),
        None => Err(unknown_worktree(token, &registry, path.as_deref())),
    }
}

/// A base candidate counts as a worktree when it holds a `.mindtape/` directory.
fn is_project(path: &Path) -> bool {
    path.join(".mindtape").is_dir()
}

/// Build a helpful "unknown worktree" error naming the known aliases and where
/// worktrees are configured.
fn unknown_worktree(
    token: &str,
    registry: &WorktreeRegistry,
    path: Option<&Path>,
) -> anyhow::Error {
    let mut msg = format!("unknown worktree @{token}");
    let aliases = registry.alias_names();
    if !aliases.is_empty() {
        msg.push_str(&format!("\n  known aliases: {}", aliases.join(", ")));
    }
    match path {
        Some(p) => msg.push_str(&format!("\n  configure worktrees in {}", p.display())),
        None => msg.push_str("\n  could not resolve the app config directory"),
    }
    anyhow!(msg)
}

/// The app-scoped worktrees config path: `<config home>/mindtape/worktrees.toml`.
/// Mirrors [`crate::install`]'s XDG defaults resolution.
fn worktrees_path() -> Option<PathBuf> {
    mindtape::config_home().map(|dir| dir.join("mindtape").join("worktrees.toml"))
}

/// Load the registry from `worktrees.toml`, returning it alongside the expected
/// path (`None` only when the config dir cannot be resolved). A missing file is
/// an empty registry, not an error; only a malformed file fails.
fn load_registry() -> Result<(WorktreeRegistry, Option<PathBuf>)> {
    let Some(path) = worktrees_path() else {
        return Ok((WorktreeRegistry::default(), None));
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let registry = WorktreeRegistry::parse(&text)
                .with_context(|| format!("parse {}", path.display()))?;
            Ok((registry, Some(path)))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Ok((WorktreeRegistry::default(), Some(path)))
        }
        Err(err) => Err(err).with_context(|| format!("read {}", path.display())),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
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
    fn run_child(
        test_name: &str,
        config: Option<&str>,
    ) -> (tempfile::TempDir, std::process::Output) {
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
            let (_dir, out) =
                run_child("load_registry_valid_file_returns_entries", Some(ALIAS_CFG));
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
}
