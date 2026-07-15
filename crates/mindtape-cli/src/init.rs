//! `mt init` / `mt install-lib` — project scaffold and library install.
//!
//! The scaffold's first line is a `#:schema` comment directive so a
//! schema-aware TOML editor (tombi, Taplo) assists filling `.mindtape/
//! config.toml`. It points, by default, at the schema `mt install` writes under
//! the XDG config dir; `--schema URL|PATH` overrides it (e.g. the published
//! URL for a portable, checked-in config), `--no-schema` drops it.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The Typst package version installed by `install-lib`. Mirrors `lib/typst.toml`.
const TYPST_PKG_VERSION: &str = "0.2.0";
const PRELUDE: &str = include_str!("../../../lib/prelude.typ");
/// The fully-expanded, heavily-commented starter config (`--full`).
const FULL_CONFIG: &str = include_str!("default-config.toml");
/// The default minimal scaffold: just `[tasks] roots` plus a pointer to the
/// embedded floor. Everything else inherits unless overridden.
const SLIM_CONFIG: &str = include_str!("slim-config.toml");

/// Arguments for `init`.
#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Overwrite an existing `.mindtape/config.toml`.
    #[arg(long)]
    pub force: bool,

    /// Write the fully-expanded, commented starter config instead of the
    /// slim default scaffold.
    #[arg(long)]
    pub full: bool,

    /// Value for the top `#:schema` directive: a URL, or a path relative to the
    /// config file. Defaults to a `file://` URL for the schema `mt install`
    /// writes under the XDG config dir.
    #[arg(long, value_name = "URL|PATH")]
    pub schema: Option<String>,

    /// Omit the `#:schema` comment directive from the scaffold.
    #[arg(long)]
    pub no_schema: bool,
}

/// Run the `init` command — scaffold `.mindtape/config.toml` in the current directory.
///
/// # Errors
/// Returns an error if the config cannot be written.
pub fn run(args: &InitArgs) -> Result<()> {
    let template = if args.full { FULL_CONFIG } else { SLIM_CONFIG };
    let text = with_schema_directive(template, args)?;
    scaffold_config(Path::new("."), args.force, &text)
}

/// Prepend the `#:schema` comment directive to the scaffold, unless
/// `--no-schema`. The value is `--schema` when given, else a `file://` URL to
/// the schema `mt install` writes under the XDG config dir; that file may not
/// exist yet, so a hint nudges the user to run `mt install`.
fn with_schema_directive(template: &str, args: &InitArgs) -> Result<String> {
    if args.no_schema {
        return Ok(template.to_owned());
    }
    let value = if let Some(value) = &args.schema {
        value.clone()
    } else {
        let path = crate::install::xdg_schema_path()?;
        if !path.exists() {
            println!(
                "note: schema not installed at {} yet — run `mt install` (the \
                 `#:schema` directive points there)",
                path.display()
            );
        }
        format!("file://{}", path.display())
    };
    Ok(format!("#:schema {value}\n{template}"))
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

pub(crate) fn scaffold_config(root: &Path, force: bool, template: &str) -> Result<()> {
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
    std::fs::write(&config, template).with_context(|| format!("write {}", config.display()))?;
    println!("wrote {}", config.display());
    Ok(())
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
