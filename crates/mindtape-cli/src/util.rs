//! Shared project discovery for the CLI commands.

use anyhow::{Context, Result};
use mindtape::{Config, LoadOutcome, config_path, find_root, load_config, load_project};

/// A discovered, loaded project: its config and load outcome.
pub struct Project {
    /// Loaded configuration.
    pub config: Config,
    /// Tasks and per-file load errors.
    pub outcome: LoadOutcome,
}

/// Find the project from the current directory and load all its tasks.
///
/// # Errors
/// Returns an error if no `.mindtape/` ancestor exists or the config is invalid.
pub fn open() -> Result<Project> {
    let cwd = std::env::current_dir().context("determine current directory")?;
    let root =
        find_root(&cwd).context("not inside a MindTape project (no .mindtape/ directory found)")?;
    let config =
        load_config(&root).with_context(|| format!("load {}", config_path(&root).display()))?;
    let outcome = load_project(&root, &config);
    Ok(Project { config, outcome })
}

/// Print per-file load errors to stderr as warnings.
pub fn warn_errors(outcome: &LoadOutcome) {
    for error in &outcome.errors {
        eprintln!("warning: {}: {}", error.file.display(), error.message);
    }
}

#[cfg(test)]
mod tests {
    use mindtape::{LoadError, LoadOutcome};

    use super::warn_errors;

    #[test]
    fn warn_errors_is_silent_when_no_errors() {
        warn_errors(&LoadOutcome::default());
    }

    #[test]
    fn warn_errors_does_not_panic_with_errors() {
        let outcome = LoadOutcome {
            tasks: vec![],
            errors: vec![LoadError {
                file: "broken.typ".into(),
                message: "eval failed".to_owned(),
            }],
        };
        warn_errors(&outcome);
    }
}
