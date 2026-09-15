//! `mt install` — set up the USER ENVIRONMENT: the `@local/mindtape` Typst
//! package, the XDG global `defaults.toml` app-config layer, and the JSON
//! Schema for `.mindtape/config.toml`.
//!
//! Distinct from `mt init`, which scaffolds a PROJECT's `.mindtape/config.toml`.
//! By default performs all three installs; `--no-lib`/`--no-defaults`/
//! `--no-schema` skip any of them.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::init::install_lib;

/// Arguments for `install`.
#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)] // independent skip/force switches, not state
pub struct InstallArgs {
    /// Skip installing the `@local/mindtape` Typst package.
    #[arg(long)]
    pub no_lib: bool,

    /// Skip installing the XDG global `defaults.toml`.
    #[arg(long)]
    pub no_defaults: bool,

    /// Skip installing the JSON Schema for `.mindtape/config.toml`.
    #[arg(long)]
    pub no_schema: bool,

    /// Overwrite an existing XDG `defaults.toml` / `config.schema.json`.
    #[arg(long)]
    pub force: bool,
}

/// Filename of the JSON Schema installed under the XDG config dir, and the
/// target of `mt init`'s default `#:schema` directive.
const SCHEMA_FILE_NAME: &str = "config.schema.json";

/// Run the `install` command.
///
/// # Errors
/// Returns an error if either half (lib or defaults) fails to write.
pub fn run(args: &InstallArgs) -> Result<()> {
    if !args.no_lib {
        install_lib()?;
    }
    if !args.no_defaults {
        let dir = xdg_defaults_dir()?;
        write_defaults(&dir, args.force)?;
    }
    if !args.no_schema {
        let dir = xdg_defaults_dir()?;
        write_schema(&dir, args.force)?;
    }
    Ok(())
}

/// `<config home>/mindtape` — the directory holding the XDG global
/// `defaults.toml`. Mirrors [`crate::config_layers::discover_layers`]'s XDG
/// layer resolution (`mindtape` crate) — one `mindtape::config_home()`, so
/// `$MINDTAPE_CONFIG_HOME` moves what `install` writes and what the loader
/// reads together.
fn xdg_defaults_dir() -> Result<PathBuf> {
    let mut path = mindtape::config_home().context("determine the global config directory")?;
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
    std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    std::fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
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

/// Write the XDG global `config.schema.json` in `dir`: the JSON Schema for
/// `.mindtape/config.toml`, generated from the config structs
/// ([`mindtape::config_schema_json`]). `mt init` scaffolds a project config
/// whose `#:schema` directive points here, so a schema-aware TOML editor
/// (tombi, Taplo) assists filling values. Leaves an existing file alone unless
/// `force`.
pub(crate) fn write_schema(dir: &Path, force: bool) -> Result<()> {
    write_seed(
        dir,
        SCHEMA_FILE_NAME,
        &mindtape::config_schema_json(),
        force,
    )
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
