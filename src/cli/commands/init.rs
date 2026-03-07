use std::fs;

use anyhow::{Result, bail};
use clap::Parser;

use crate::TYPST_PACKAGE_VERSION;

const PRELUDE_TYP: &str = include_str!("../../../lib/prelude.typ");
const TYPST_TOML: &str = include_str!("../../../lib/typst.toml");

/// Install the `MindTape` Typst library for `#import "@local/mindtape:VERSION"`.
#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Overwrite existing library files
    #[arg(long)]
    pub force: bool,
}

impl InitArgs {
    /// Write the embedded Typst library to the local packages directory.
    ///
    /// # Errors
    /// Returns an error if `HOME` is not set or the target directory cannot be created.
    pub fn run(&self) -> Result<()> {
        let target_dir = typst_package_dir()?;

        if target_dir.exists() && !self.force {
            eprintln!("Already installed at {}", target_dir.display());
            eprintln!("Use --force to overwrite");
            return Ok(());
        }

        fs::create_dir_all(&target_dir)?;
        fs::write(target_dir.join("typst.toml"), TYPST_TOML)?;
        fs::write(target_dir.join("prelude.typ"), PRELUDE_TYP)?;

        eprintln!("Installed to {}", target_dir.display());
        eprintln!("Use in Typst files: #import \"@local/mindtape:{TYPST_PACKAGE_VERSION}\": *");
        Ok(())
    }
}

fn typst_package_dir() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME not set"))?;

    let base = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{home}/.local/share"));

    let dir = std::path::PathBuf::from(base).join(format!(
        "typst/packages/local/mindtape/{TYPST_PACKAGE_VERSION}"
    ));

    if dir.components().count() < 4 {
        bail!("resolved package path looks too short: {}", dir.display());
    }

    Ok(dir)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn embedded_files_are_not_empty() {
        assert!(!PRELUDE_TYP.is_empty());
        assert!(!TYPST_TOML.is_empty());
    }

    #[test]
    fn prelude_contains_due_function() {
        assert!(PRELUDE_TYP.contains("#let due("));
    }

    #[test]
    fn typst_toml_contains_package_name() {
        assert!(TYPST_TOML.contains("name = \"mindtape\""));
    }

    #[test]
    fn init_writes_files_to_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        let pkg_dir = dir.path().join("typst/packages/local/mindtape/0.1.0");

        // Simulate what run() does, but to a temp dir
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join("typst.toml"), TYPST_TOML).unwrap();
        fs::write(pkg_dir.join("prelude.typ"), PRELUDE_TYP).unwrap();

        assert!(pkg_dir.join("typst.toml").exists());
        assert!(pkg_dir.join("prelude.typ").exists());

        let content = fs::read_to_string(pkg_dir.join("prelude.typ")).unwrap();
        assert_eq!(content, PRELUDE_TYP);
    }
}
