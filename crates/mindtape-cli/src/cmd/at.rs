//! `@name` sugar and the `mt at` command.
//!
//! `at.toml` itself ([`mindtape::AtConfig`]) is pure; this module is the IO and
//! CLI edge: locate and read the app-scoped file, resolve a leading `@token`,
//! and rewrite it before clap parses. A project becomes `-w <path>`, so
//! `mt @sn ls` reuses the existing worktree-override plumbing and every command
//! honours it uniformly. A set is held for the commands that call
//! `project_set::open_set`, and `util::open` refuses it.
//!
//! Only the first post-program argument is treated as sugar; `@name` anywhere
//! else (the view sigil, `mt ls @view`) is left untouched, so the two `@` uses
//! stay disjoint by position.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use mindtape::{AtConfig, AtError, DiskFs, Member, Target};

use crate::diag::{Diag, Hint, Note, Result};
use crate::project_set::{HeldSet, init_held_set};

/// Arguments for `mt at`.
#[derive(Debug, clap::Args)]
pub struct AtArgs {
    /// The `@name` to open up, without the `@`: lists the projects it holds.
    pub name: Option<String>,
    /// Print JSON instead of a list.
    #[arg(long)]
    pub json: bool,
}

/// Run `mt at`: list the entries of `at.toml`, or the projects one holds.
///
/// # Errors
/// Returns an error if `at.toml` exists but fails to parse, or `name` names
/// nothing or resolves badly.
pub fn run(args: &AtArgs) -> Result<()> {
    let (config, path) = load_at()?;
    if let Some(name) = &args.name {
        return show_one(&config, path.as_deref(), name, args.json);
    }
    show_all(&config, path.as_deref(), args.json);
    Ok(())
}

/// `mt at`: every entry with its `include`.
fn show_all(config: &AtConfig, path: Option<&Path>, json: bool) {
    let entries: Vec<_> = config
        .names()
        .into_iter()
        .filter_map(|name| config.entry(name).map(|entry| (name, entry)))
        .collect();
    if json {
        let items: Vec<serde_json::Value> = entries
            .iter()
            .map(|(name, entry)| {
                serde_json::json!({
                    "name": name,
                    "include": entry.include,
                    "exclude": entry.exclude,
                    "forks": forks_word(entry.forks),
                })
            })
            .collect();
        print_json(&serde_json::Value::Array(items));
        return;
    }
    if entries.is_empty() {
        match path {
            Some(p) => println!(
                "nothing configured (add [at.<name>] entries to {})",
                p.display()
            ),
            None => println!("nothing configured (could not resolve the app config directory)"),
        }
        return;
    }
    for (name, entry) in entries {
        println!("@{name} → {}", entry.include.join(", "));
    }
}

/// `mt at <name>`: what `@name` resolves to.
fn show_one(config: &AtConfig, path: Option<&Path>, name: &str, json: bool) -> Result<()> {
    let target = resolve(config, name, |pick| format!("mt at {pick}"))?
        .ok_or_else(|| unknown_name(name, config, path))?;
    match target {
        Target::Project(root) if json => print_json(&serde_json::json!({
            "name": name, "kind": "project", "path": root.display().to_string(),
        })),
        Target::Project(root) => println!("@{name} → {}", root.display()),
        Target::Set(members) if json => print_json(&serde_json::json!({
            "name": name, "kind": "set", "members": members.iter().map(member_json).collect::<Vec<_>>(),
        })),
        Target::Set(members) => {
            println!(
                "@{name} holds {}",
                mindtape::count(members.len(), "project")
            );
            for line in member_lines(&members, dirs::home_dir().as_deref()) {
                println!("{line}");
            }
        }
    }
    Ok(())
}

/// One line per member: the name padded to the longest, the path (`~/…` under
/// `home`), and a `(fork)` mark.
fn member_lines(members: &[Member], home: Option<&Path>) -> Vec<String> {
    let width = members
        .iter()
        .map(|m| m.name.chars().count())
        .max()
        .unwrap_or(0);
    members
        .iter()
        .map(|member| {
            let fork = if member.fork { "  (fork)" } else { "" };
            let shown = shown_path(&member.path, home);
            format!("{:<width$}  {shown}{fork}", member.name)
        })
        .collect()
}

/// `path` as `~/…` when it lies under `home`, else whole.
fn shown_path(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|h| path.strip_prefix(h).ok()) {
        Some(rest) => {
            let parts: Vec<_> = rest.iter().map(|c| c.to_string_lossy()).collect();
            format!("~/{}", parts.join("/"))
        }
        None => path.display().to_string(),
    }
}

fn member_json(member: &Member) -> serde_json::Value {
    serde_json::json!({
        "name": member.name, "path": member.path.display().to_string(), "fork": member.fork,
    })
}

fn forks_word(forks: mindtape::Forks) -> &'static str {
    match forks {
        mindtape::Forks::Skip => "false",
        mindtape::Forks::Keep => "true",
        mindtape::Forks::Only => "only",
    }
}

fn print_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
}

/// Rewrite a leading `@name` argument before clap parses: a project into
/// `-w <path>`, a set into nothing, held for `project_set::open_set`. A bare
/// `@name` gains the `board` command.
///
/// # Errors
/// Returns an error if the leading `@token` cannot be resolved.
pub fn desugar_argv(argv: Vec<OsString>) -> Result<Vec<OsString>> {
    let name = leading_token(&argv);
    let command = argv.get(2..).and_then(command_word);
    let (out, held) = desugar_with(argv, |token| resolve_token(token, command.as_deref()))?;
    // A project turned into `-w <path>`; remember the name for a command's
    // hint to restate. A set is held, and its hint reads the held name.
    if let (None, Some(name)) = (&held, name) {
        crate::selector::init_at_name(&name);
    }
    init_held_set(held);
    Ok(out)
}

/// The `@token` (without the `@`) right after the program name, if any.
fn leading_token(argv: &[OsString]) -> Option<String> {
    argv.get(1)
        .and_then(|arg| arg.to_str())
        .and_then(|s| s.strip_prefix('@'))
        .map(str::to_owned)
}

/// The first argument in `args` that is no flag: the command a `@token`
/// precedes.
fn command_word(args: &[OsString]) -> Option<String> {
    args.iter()
        .filter_map(|arg| arg.to_str())
        .find(|arg| !arg.starts_with('-'))
        .map(str::to_owned)
}

/// The pure core of [`desugar_argv`]: `resolve` maps a token to what it names,
/// injected so the rewrite is testable without touching the filesystem.
pub(crate) fn desugar_with<F>(
    argv: Vec<OsString>,
    resolve: F,
) -> Result<(Vec<OsString>, Option<HeldSet>)>
where
    F: Fn(&str) -> Result<Target>,
{
    let mut iter = argv.into_iter();
    let Some(prog) = iter.next() else {
        return Ok((Vec::new(), None));
    };
    let mut rest: Vec<OsString> = iter.collect();
    // Detach the token from `rest` before mutating it.
    let token = rest
        .first()
        .and_then(|arg| arg.to_str())
        .and_then(|s| s.strip_prefix('@'))
        .map(str::to_owned);
    let mut out = vec![prog];
    let mut held = None;
    if let Some(token) = token {
        let target = resolve(&token)?;
        rest.remove(0);
        // A bare `mt @x`, flags aside, runs `mt @x board`.
        if rest
            .iter()
            .all(|arg| arg.to_str().is_some_and(|s| s.starts_with('-')))
        {
            rest.insert(0, OsString::from("board"));
        }
        match target {
            Target::Project(path) => {
                out.push(OsString::from("-w"));
                out.push(path.into_os_string());
            }
            Target::Set(members) => {
                held = Some(HeldSet {
                    name: token,
                    command: command_word(&rest),
                    members: members.into_iter().map(|m| (m.name, m.path)).collect(),
                });
            }
        }
    }
    out.extend(rest);
    Ok((out, held))
}

/// Resolve a `@token` (without the `@`) through the app-scoped `at.toml`,
/// erroring with the known names and config path on a miss. `command` is the
/// command the token precedes, which a clash's hint restates.
fn resolve_token(token: &str, command: Option<&str>) -> Result<Target> {
    let (config, path) = load_at()?;
    let example = |pick: &str| match command {
        Some(word) => format!("mt @{pick} {word}"),
        None => format!("mt @{pick}"),
    };
    resolve(&config, token, example)?.ok_or_else(|| unknown_name(token, &config, path.as_deref()))
}

/// [`AtConfig::resolve`] against the disk, with a failure as a `Diag`.
/// `example` spells the command line that picks one directory of a clash.
fn resolve(
    config: &AtConfig,
    token: &str,
    example: impl Fn(&str) -> String,
) -> Result<Option<Target>> {
    let home = dirs::home_dir();
    config
        .resolve(token, home.as_deref(), &DiskFs)
        .map_err(|err| match &err {
            AtError::Clash { candidates, .. } => clash(token, candidates, home.as_deref(), example),
            _ => Diag::usage("can't resolve")
                .value(format!("@{token}"))
                .cause(&err),
        })
}

/// The refusal of a name several directories answer to: each one with the
/// suffix name that picks it, and a command line that does.
fn clash(
    token: &str,
    candidates: &[Member],
    home: Option<&Path>,
    example: impl Fn(&str) -> String,
) -> Diag {
    let diag = candidates.iter().fold(
        Diag::usage("name matches several directories").value(format!("@{token}")),
        |diag, member| {
            let shown = shown_path(&member.path, home);
            diag.note(Note::new("candidate").name(format!("@{} → {shown}", member.name)))
        },
    );
    match candidates.first() {
        Some(first) => diag.hint(Hint::as_in(
            "pick one by its path suffix",
            example(&first.name),
        )),
        None => diag,
    }
}

/// A helpful "unknown name" error: a nearest-match hint, the known names and
/// where `at.toml` lives, as notes. A leading `@token` is this sugar, rewritten
/// before clap ever sees it — a bare `[views.<name>]` reference only ever
/// appears in argument position (`mt ls @view`) — so the two error messages
/// never fire for the same input and stay distinguishable by their leading
/// word (`unknown name` vs. `unknown view`) alone.
fn unknown_name(token: &str, config: &AtConfig, path: Option<&Path>) -> Diag {
    let names = config.names();
    let diag = Diag::usage("unknown name").value(format!("@{token}"));
    let mut diag = diag.did_you_mean_or_with(
        token,
        &names,
        |near| format!("@{near}"),
        Hint::new("add it to at.toml"),
    );
    if !names.is_empty() {
        diag = diag.note(Note::new("known names").name(names.join(", ")));
    }
    match path {
        Some(p) => diag.note(Note::new("names are configured in").value(p.display().to_string())),
        None => diag.note(Note::new("no app config directory resolves")),
    }
}

/// The app-scoped config path: `<config home>/mindtape/at.toml`.
fn at_path() -> Option<PathBuf> {
    mindtape::config_home().map(|dir| dir.join("mindtape").join("at.toml"))
}

/// The config of an absent `at.toml`: what an empty file parses to.
fn no_at() -> Result<AtConfig> {
    AtConfig::parse("").map_err(|err| Diag::failed("can't build an empty at.toml").cause(&err))
}

/// Load `at.toml`, returning it alongside the expected path (`None` only when
/// the config dir cannot be resolved). A missing file is an empty config, not
/// an error; only a malformed file fails.
pub(crate) fn load_at() -> Result<(AtConfig, Option<PathBuf>)> {
    let Some(path) = at_path() else {
        return Ok((no_at()?, None));
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let config = AtConfig::parse(&text).map_err(|err| {
                Diag::failed("can't parse at.toml")
                    .at_file(&path)
                    .cause(&err)
            })?;
            Ok((config, Some(path)))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok((no_at()?, Some(path))),
        Err(err) => Err(Diag::cannot_read(&path, &err)),
    }
}

#[cfg(test)]
#[path = "at_tests.rs"]
mod tests;
