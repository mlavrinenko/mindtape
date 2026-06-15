//! `mindtape init` / `mindtape install-lib` — project scaffold and library install.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The Typst package version installed by `install-lib`. Mirrors `lib/typst.toml`.
const TYPST_PKG_VERSION: &str = "0.2.0";
const PRELUDE: &str = include_str!("../../../lib/prelude.typ");
const DEFAULT_CONFIG: &str = include_str!("default-config.toml");

/// Arguments for `init`.
#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Overwrite an existing `.mindtape/config.toml`.
    #[arg(long)]
    pub force: bool,
}

/// Run the `init` command — scaffold `.mindtape/config.toml` in the current directory.
///
/// # Errors
/// Returns an error if the config cannot be written.
pub fn run(args: &InitArgs) -> Result<()> {
    scaffold_config(Path::new("."), args.force)
}

/// Run the `install-lib` command — install `@local/mindtape` into the Typst packages dir.
///
/// # Errors
/// Returns an error if the library cannot be written.
pub fn run_install_lib() -> Result<()> {
    install_lib()
}

pub(crate) fn install_lib() -> Result<()> {
    let dir = package_dir()?;
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let manifest = format!(
        "[package]\nname = \"mindtape\"\nversion = \"{TYPST_PKG_VERSION}\"\nentrypoint = \"prelude.typ\"\n"
    );
    std::fs::write(dir.join("typst.toml"), manifest).context("write typst.toml")?;
    std::fs::write(dir.join("prelude.typ"), PRELUDE).context("write prelude.typ")?;
    println!(
        "installed @local/mindtape:{TYPST_PKG_VERSION} -> {}",
        dir.display()
    );
    Ok(())
}

pub(crate) fn package_dir() -> Result<PathBuf> {
    let mut path = dirs::data_local_dir().context("determine local data directory")?;
    path.push("typst");
    path.push("packages");
    path.push("local");
    path.push("mindtape");
    path.push(TYPST_PKG_VERSION);
    Ok(path)
}

pub(crate) fn scaffold_config(root: &Path, force: bool) -> Result<()> {
    let dir = root.join(".mindtape");
    let config = dir.join("config.toml");
    if config.exists() && !force {
        println!(
            "{} already exists (use --force to overwrite)",
            config.display()
        );
        return Ok(());
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    std::fs::write(&config, DEFAULT_CONFIG)
        .with_context(|| format!("write {}", config.display()))?;
    println!("wrote {}", config.display());
    Ok(())
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
