//! `mt install` — set up the USER ENVIRONMENT: the `@local/mindtape` Typst
//! package plus the XDG global `defaults.toml` app-config layer.
//!
//! Distinct from `mt init`, which scaffolds a PROJECT's `.mindtape/config.toml`.
//! By default performs both installs; `--no-lib`/`--no-defaults` skip either
//! half. `mt install-lib` remains a hidden, deprecated alias for the lib-only
//! path (`mt install --no-defaults`).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::init::install_lib;

/// Arguments for `install`.
#[derive(Debug, clap::Args)]
pub struct InstallArgs {
    /// Skip installing the `@local/mindtape` Typst package.
    #[arg(long)]
    pub no_lib: bool,

    /// Skip installing the XDG global `defaults.toml`.
    #[arg(long)]
    pub no_defaults: bool,

    /// Overwrite an existing XDG `defaults.toml`.
    #[arg(long)]
    pub force: bool,
}

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
    Ok(())
}

/// `dirs::config_dir()/mindtape` — the directory holding the XDG global
/// `defaults.toml`. Mirrors [`crate::config_layers::discover_layers`]'s XDG
/// layer resolution (`mindtape` crate).
fn xdg_defaults_dir() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("determine XDG config directory")?;
    path.push("mindtape");
    Ok(path)
}

/// Write the XDG global `defaults.toml` in `dir`, seeded from the embedded
/// config floor (`mindtape::embedded_defaults()`) — the same commented text
/// `mt config show --defaults` prints, so there is no third copy. Leaves an
/// existing file alone unless `force`.
pub(crate) fn write_defaults(dir: &Path, force: bool) -> Result<()> {
    let path = dir.join("defaults.toml");
    if path.exists() && !force {
        println!(
            "{} already exists (use --force to overwrite)",
            path.display()
        );
        return Ok(());
    }
    std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    std::fs::write(&path, mindtape::embedded_defaults())
        .with_context(|| format!("write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
