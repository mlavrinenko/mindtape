//! `mt install` — set up the USER ENVIRONMENT: the `@local/mindtape` Typst
//! package, the XDG global `defaults.toml` app-config layer, the JSON Schemas
//! for `.mindtape/config.toml` and `at.toml`, and the shell completion hooks.
//!
//! Distinct from `mt init`, which scaffolds a PROJECT's `.mindtape/config.toml`.
//! By default performs all four installs; `--no-lib`/`--no-defaults`/
//! `--no-schema`/`--no-completions` skip any of them.
//!
//! The completion hook is the odd one out in what it CONTAINS: the other three
//! files carry content, this one carries a call. Each is the couple of lines
//! that ask `mt` itself for the completion on every tab press
//! ([`crate::cli::completion`]), never a generated completion script — a script
//! written here would be a copy of the parser and of this project's config,
//! stale from the next flag and wrong in every other project. It also means an
//! `mt` upgrade needs no re-install: the hook is re-evaluated at shell start.
//!
//! # Files
//!
//! - `hint.rs` — remedies for load failures whose diagnostic names none.

pub(crate) mod hint;

use std::path::{Path, PathBuf};

use crate::cmd::init::{install_lib, write_announced};
use crate::diag::{Diag, Result};

/// Arguments for `install`.
#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)] // independent skip/force switches, not state
pub struct InstallArgs {
    /// Skip installing the `@local/mindtape` Typst package.
    #[arg(long)]
    pub no_lib: bool,

    /// Skip writing your user-wide `defaults.toml`.
    #[arg(long)]
    pub no_defaults: bool,

    /// Skip installing the JSON Schemas for `.mindtape/config.toml` and
    /// `at.toml`.
    #[arg(long)]
    pub no_schema: bool,

    /// Skip installing the shell completion hooks.
    #[arg(long)]
    pub no_completions: bool,

    /// Overwrite files that already exist.
    ///
    /// Those are your `defaults.toml`, the JSON Schemas and the completion
    /// hooks. The package is written on every install, unless its directory
    /// is a symlink: `--force` replaces the link with this binary's copy.

    #[arg(long)]
    pub force: bool,
}

/// Filename of the JSON Schema installed under the XDG config dir, and the
/// target of `mt init`'s default `#:schema` directive.
const SCHEMA_FILE_NAME: &str = "config.schema.json";

/// Filename of the `at.toml` JSON Schema, beside `at.toml` itself, so that
/// file's `#:schema ./at.schema.json` line finds it.
const AT_SCHEMA_FILE_NAME: &str = "at.schema.json";

/// One installer of a file under the config dir: `(dir, force)`.
type ConfigWriter = fn(&Path, bool) -> Result<()>;

/// Run the `install` command.
///
/// # Errors
/// Returns an error if either half (lib or defaults) fails to write.
pub fn run(args: &InstallArgs) -> Result<()> {
    if !args.no_lib {
        install_lib(args.force)?;
    }
    let config_writers: [(bool, ConfigWriter); 2] = [
        (args.no_defaults, write_defaults),
        (args.no_schema, write_schema),
    ];
    for (skip, write) in config_writers {
        if !skip {
            write(&xdg_defaults_dir()?, args.force)?;
        }
    }
    if !args.no_completions {
        let dir = completions_dir()?;
        write_completions(&dir, args.force)?;
        let shell = std::env::var("SHELL").ok();
        println!(
            "{}",
            completion_hint(shell_from_env(shell.as_deref()), &dir)
        );
    }
    Ok(())
}

/// The couple of lines each shell needs to ask `mt` for its own completions.
///
/// Keyed by the name `clap_complete`'s `CompleteEnv` answers to, because that
/// is the name the hook has to pass back in `$COMPLETE`; the test beside this
/// module asserts the keys are exactly `Shells::builtins()`, so a shell the
/// library learns arrives here by failing rather than by being noticed.
///
/// Each guards on `mt` being on `$PATH`: these files are sourced from a
/// startup file, and a shell that errors at launch because a binary moved is a
/// worse failure than no completions.
const HOOKS: &[(&str, &str)] = &[
    (
        "bash",
        "command -v mt >/dev/null 2>&1 && source <(COMPLETE=bash mt)\n",
    ),
    (
        "zsh",
        "command -v mt >/dev/null 2>&1 && source <(COMPLETE=zsh mt)\n",
    ),
    (
        "fish",
        "if command -q mt\n    COMPLETE=fish mt | source\nend\n",
    ),
    (
        "elvish",
        "if (has-external mt) { eval (E:COMPLETE=elvish mt | slurp) }\n",
    ),
    (
        "powershell",
        "if (Get-Command mt -ErrorAction SilentlyContinue) {\n  \
         $env:COMPLETE = \"powershell\"\n  \
         mt | Out-String | Invoke-Expression\n  \
         Remove-Item Env:\\COMPLETE\n}\n",
    ),
];

/// The shells [`HOOKS`] installs for, for the test that holds the table
/// against `clap_complete`'s own list.
#[cfg(test)]
pub(crate) fn hook_shells() -> impl Iterator<Item = &'static str> {
    HOOKS.iter().map(|(shell, _)| *shell)
}

/// The [`HOOKS`] shell that `$SHELL` names, by its last path segment on
/// either separator and without `.exe`, or bash when it names none. `pwsh` is
/// `PowerShell` 7's binary.
pub(crate) fn shell_from_env(shell: Option<&str>) -> &'static str {
    let name = shell
        .and_then(|path| path.rsplit(['/', '\\']).next())
        .map(|name| name.strip_suffix(".exe").unwrap_or(name));
    let name = if name == Some("pwsh") {
        Some("powershell")
    } else {
        name
    };
    HOOKS
        .iter()
        .map(|(shell, _)| *shell)
        .find(|shell| Some(*shell) == name)
        .unwrap_or("bash")
}

/// What `mt install` prints last: how to load `shell`'s hook from `dir` in
/// that shell's startup file. A file without `.ps1` can't be dot-sourced, so
/// `PowerShell` reads it and evaluates it instead.
pub(crate) fn completion_hint(shell: &str, dir: &Path) -> String {
    let path = quote_path(&dir.join(shell));
    let line = match shell {
        "elvish" => format!("eval (slurp < {path})"),
        "powershell" => format!("Get-Content -Raw {path} | Invoke-Expression"),
        _ => format!("source {path}"),
    };
    format!("For Tab completion, add this to your shell's startup file:\n  {line}")
}

/// `path` as one shell word: bare when it holds only characters no shell
/// splits or expands, else in single quotes, which every hook shell reads
/// literally. A Windows path holds a backslash, so it is always quoted.
pub(crate) fn quote_path(path: &Path) -> String {
    let text = path.display().to_string();
    let bare = text
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '-' | '_'));
    if bare { text } else { format!("'{text}'") }
}

/// `<config home>/mindtape/completions` — one hook file per shell, named after
/// the shell so a startup file can name its own without a lookup table.
fn completions_dir() -> Result<PathBuf> {
    Ok(xdg_defaults_dir()?.join("completions"))
}

/// Write every [`HOOKS`] entry into `dir`. Leaves an existing file alone
/// unless `force`, like the other three installers.
pub(crate) fn write_completions(dir: &Path, force: bool) -> Result<()> {
    for (shell, hook) in HOOKS {
        write_seed(dir, shell, hook, force)?;
    }
    Ok(())
}

/// `<config home>/mindtape` — the directory holding the XDG global
/// `defaults.toml`. Mirrors [`crate::config::layers::discover_layers`]'s XDG
/// layer resolution (`mindtape` crate) — one `mindtape::config_home()`, so
/// `$MINDTAPE_CONFIG_HOME` moves what `install` writes and what the loader
/// reads together.
fn xdg_defaults_dir() -> Result<PathBuf> {
    let mut path = mindtape::config_home()
        .ok_or_else(|| Diag::failed("can't find the global config directory"))?;
    path.push("mindtape");
    Ok(path)
}

/// Write `content` to `dir/name`, creating `dir`, leaving an existing file
/// alone unless `force`, and announcing what it did. Shared by the defaults and
/// schema installs so the seed-or-skip logic lives in one place.
fn write_seed(dir: &Path, name: &str, content: &str, force: bool) -> Result<()> {
    let path = dir.join(name);
    if path.exists() && !force {
        println!(
            "{} already exists (use --force to overwrite)",
            path.display()
        );
        return Ok(());
    }
    write_announced(dir, &path, content)
}

/// Write the XDG global `defaults.toml` in `dir`, seeded from the embedded
/// config floor (`mindtape::embedded_defaults()`) — the same commented text
/// `mt config show --defaults` prints, so there is no third copy. Leaves an
/// existing file alone unless `force`.
pub(crate) fn write_defaults(dir: &Path, force: bool) -> Result<()> {
    write_seed(dir, "defaults.toml", mindtape::embedded_defaults(), force)
}

/// The path `mt install` writes the JSON Schema to (`<XDG config>/mindtape/
/// config.schema.json`) — the target `mt init`'s default `#:schema` directive
/// points at.
///
/// # Errors
/// Returns an error if the XDG config directory cannot be determined.
pub(crate) fn xdg_schema_path() -> Result<PathBuf> {
    Ok(xdg_defaults_dir()?.join(SCHEMA_FILE_NAME))
}

/// Write the two JSON Schemas in `dir`, each generated from its structs, so a
/// schema-aware TOML editor (tombi, Taplo) assists filling values.
/// `config.schema.json` ([`mindtape::config_schema_json`]) is what the
/// `#:schema` directive `mt init` scaffolds points at. `at.schema.json`
/// ([`mindtape::at::schema_json`]) sits beside `at.toml`, which you write
/// yourself. Leaves an existing file alone unless `force`.
pub(crate) fn write_schema(dir: &Path, force: bool) -> Result<()> {
    write_seed(
        dir,
        SCHEMA_FILE_NAME,
        &mindtape::config_schema_json(),
        force,
    )?;
    write_seed(
        dir,
        AT_SCHEMA_FILE_NAME,
        &mindtape::at::schema_json(),
        force,
    )
}

#[cfg(test)]
mod tests;
