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

use crate::diag::Result;
use crate::diag::{Diag, Hint, Note};
use mindtape::{WorktreeEntry, WorktreeRegistry};

/// Arguments for `mt worktrees`.
#[derive(Debug, clap::Args)]
pub struct WorktreesArgs {
    /// Print JSON instead of a list.
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
pub(crate) fn desugar_with<F>(argv: Vec<OsString>, resolve: F) -> Result<Vec<OsString>>
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
    let resolved = registry
        .resolve(token, dirs::home_dir().as_deref(), is_project)
        .map_err(|err| {
            Diag::usage("can't resolve worktree")
                .value(format!("@{token}"))
                .cause(&err)
        })?;
    match resolved {
        Some(resolved) => Ok(resolved),
        None => Err(unknown_worktree(token, &registry, path.as_deref())),
    }
}

/// A base candidate counts as a worktree when it holds a `.mindtape/` directory.
fn is_project(path: &Path) -> bool {
    path.join(".mindtape").is_dir()
}

/// Build a helpful "unknown worktree" error: a nearest-match hint, the known
/// aliases and where worktrees are configured, as notes. A leading `@token` is this worktree-alias sugar,
/// rewritten before clap ever sees it — a bare `[views.<name>]` reference only
/// ever appears in argument position (`mt ls @view`) — so the two error
/// messages never fire for the same input and stay distinguishable by their
/// leading word (`unknown worktree` vs. `unknown view`) alone.
fn unknown_worktree(token: &str, registry: &WorktreeRegistry, path: Option<&Path>) -> Diag {
    let aliases = registry.alias_names();
    let diag = Diag::usage("unknown worktree").value(format!("@{token}"));
    let mut diag = match mindtape::suggest::nearest(token, &aliases) {
        Some(near) => diag.hint(Hint::did_you_mean(format!("@{near}"))),
        None => diag.hint(Hint::new("add it to worktrees.toml")),
    };
    if !aliases.is_empty() {
        diag = diag.note(Note::new("known worktrees").name(aliases.join(", ")));
    }
    match path {
        Some(p) => {
            diag.note(Note::new("worktrees are configured in").value(p.display().to_string()))
        }
        None => diag.note(Note::new("no app config directory resolves")),
    }
}

/// The app-scoped worktrees config path: `<config home>/mindtape/worktrees.toml`.
/// Mirrors [`crate::cmd::install`]'s XDG defaults resolution.
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
            let registry = WorktreeRegistry::parse(&text).map_err(|err| {
                Diag::failed("can't parse the worktrees config")
                    .at_file(&path)
                    .cause(&err)
            })?;
            Ok((registry, Some(path)))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Ok((WorktreeRegistry::default(), Some(path)))
        }
        Err(err) => Err(Diag::cannot_read(&path, &err)),
    }
}

#[cfg(test)]
#[path = "worktrees_tests.rs"]
mod tests;
