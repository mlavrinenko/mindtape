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

/// The app-scoped worktrees config path: `<config-dir>/mindtape/worktrees.toml`.
/// Mirrors [`crate::install`]'s XDG defaults resolution.
fn worktrees_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("mindtape").join("worktrees.toml"))
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
}
