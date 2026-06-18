//! Shared project discovery for the CLI commands.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use mindtape::{Config, LoadOutcome, Task, config_path, find_root, load_config, load_project};

/// A discovered, loaded project: its root, config and load outcome.
pub struct Project {
    /// Absolute path to the project root (the directory containing `.mindtape/`).
    pub root: PathBuf,
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
    Ok(Project {
        root,
        config,
        outcome,
    })
}

/// Resolve a task reference to a loaded `Task` and its absolute path.
///
/// Accepts a slug (bare file stem), a relative path (resolved against CWD),
/// or an absolute path. Errors if the reference doesn't match any loaded task.
///
/// # Errors
/// Returns an error if the path cannot be resolved or no task matches.
pub fn resolve_task(project: &Project, id: &str) -> Result<(Task, PathBuf)> {
    let path = Path::new(id);
    let is_path = path.is_absolute() || id.contains('/') || id.ends_with(".typ");
    if is_path {
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .context("determine current directory")?
                .join(id)
        };
        let abs = abs
            .canonicalize()
            .with_context(|| format!("resolve {}", abs.display()))?;
        let task = project
            .outcome
            .tasks
            .iter()
            .find(|t| project.root.join(&t.file).canonicalize().ok().as_deref() == Some(&abs))
            .with_context(|| format!("no task at {}", abs.display()))?;
        return Ok((task.clone(), abs));
    }
    let stem = id.strip_suffix(".typ").unwrap_or(id);
    let task = project
        .outcome
        .tasks
        .iter()
        .find(|t| t.id == stem)
        .with_context(|| format!("no task with id {stem:?}"))?;
    let abs = project.root.join(&task.file);
    Ok((task.clone(), abs))
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
