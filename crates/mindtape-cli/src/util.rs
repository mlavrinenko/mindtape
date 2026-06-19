//! Shared project discovery for the CLI commands.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use mindtape::{
    Config, Filter, LoadOutcome, Task, config_path, find_root, load_config, load_project, select,
};

use crate::handle::{DEFAULT_CHECK_LEN, check_chars, parse_handle};

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
/// Accepts:
/// - A handle (`<ordinal><check>`, e.g. `"5d"`): resolves against the **default
///   view** (configured default filter, no extra sort — exactly what bare
///   `mindtape list` shows).  The ordinal is 1-based.  The check letter must
///   match the task's stable id hash; a mismatch means the list has drifted and
///   the error names the current handle and title at that ordinal.
/// - A slug (bare file stem, e.g. `"implement-auth"`).
/// - A relative path (resolved against CWD).
/// - An absolute path.
///
/// Errors if the reference doesn't match any loaded task.
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

    // Try handle resolution first: `<digits><letters>` with no `/` or `.`.
    if let Some((ordinal, check)) = parse_handle(id) {
        return resolve_by_handle(project, ordinal, check);
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

/// Build the default list view: apply the configured default filter, no extra
/// sort.  This replicates what bare `mindtape list` shows.
fn default_view(project: &Project) -> Result<Vec<Task>> {
    let filter = project
        .config
        .query
        .default_filter
        .as_deref()
        .map(Filter::parse)
        .transpose()?;
    let tasks = select(project.outcome.tasks.clone(), filter.as_ref())?;
    Ok(tasks)
}

/// Resolve a task by `ordinal` (1-based) and `check` string against the default
/// view.  Errors if ordinal is out of range or the check does not match.
fn resolve_by_handle(project: &Project, ordinal: usize, check: &str) -> Result<(Task, PathBuf)> {
    let tasks = default_view(project)?;
    let total = tasks.len();

    let index = ordinal
        .checked_sub(1)
        .filter(|&i| i < total)
        .with_context(|| {
            format!("handle ordinal {ordinal} out of range (default view has {total} tasks)")
        })?;

    let task = tasks
        .get(index)
        .with_context(|| format!("handle ordinal {ordinal} out of range"))?;
    let expected_check = check_chars(&task.id, DEFAULT_CHECK_LEN);
    if check != expected_check {
        let current_handle = format!("{ordinal}{expected_check}");
        bail!(
            "handle check mismatch: you passed {ordinal}{check} but that position is now \
             {current_handle} ({title:?}); the default list view may have changed",
            title = task.title,
        );
    }

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
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use mindtape::{Config, LoadError, LoadOutcome, Task, load_project};

    use super::{Project, resolve_task, warn_errors};
    use crate::handle::{DEFAULT_CHECK_LEN, check_chars};

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

    // Minimal task for unit tests — no file on disk needed for pure logic tests.
    fn stub_task(id: &str, title: &str) -> Task {
        Task {
            id: id.to_owned(),
            file: format!("tasks/{id}.typ").into(),
            title: title.to_owned(),
            status: None,
            priority: None,
            grade: None,
            kind: None,
            estimate: BTreeMap::new(),
            depends_on: Vec::new(),
            relates_to: Vec::new(),
            supersedes: Vec::new(),
            duplicates: Vec::new(),
            tags: Vec::new(),
            due: None,
            start: None,
            reviewed: None,
            extras: BTreeMap::new(),
        }
    }

    const TASK_SRC: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: \"high\",
  ),
  extras: (:),
))
";

    fn make_project_with_tasks(dir: &Path, task_ids: &[&str]) -> Project {
        std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
        std::fs::create_dir_all(dir.join("tasks")).unwrap();
        for id in task_ids {
            std::fs::write(dir.join("tasks").join(format!("{id}.typ")), TASK_SRC).unwrap();
        }
        let config = Config::default();
        let outcome = load_project(dir, &config);
        Project {
            root: dir.to_path_buf(),
            config,
            outcome,
        }
    }

    fn stub_project(tasks: Vec<Task>) -> Project {
        let outcome = LoadOutcome {
            tasks,
            errors: vec![],
        };
        Project {
            root: ".".into(),
            config: Config::default(),
            outcome,
        }
    }

    #[test]
    fn resolve_by_slug_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project_with_tasks(dir.path(), &["alpha"]);
        let (task, _) = resolve_task(&project, "alpha").unwrap();
        assert_eq!(task.id, "alpha");
    }

    #[test]
    fn resolve_handle_correct_check_succeeds() {
        let task_a = stub_task("alpha", "Alpha Task");
        let task_b = stub_task("beta", "Beta Task");
        let project = stub_project(vec![task_a.clone(), task_b.clone()]);

        // Build the correct check char for ordinal 2 (beta).
        let check = check_chars("beta", DEFAULT_CHECK_LEN);
        let handle = format!("2{check}");
        let (resolved, _) = resolve_task(&project, &handle).unwrap();
        assert_eq!(resolved.id, "beta");
    }

    #[test]
    fn resolve_handle_wrong_check_errors_with_current_handle() {
        let task_a = stub_task("alpha", "Alpha Task");
        let project = stub_project(vec![task_a]);

        // Compute correct check then use a wrong one.
        let correct = check_chars("alpha", DEFAULT_CHECK_LEN);
        // Pick a letter that is definitely wrong.
        let wrong_char = if correct == "a" { 'b' } else { 'a' };
        let bad_handle = format!("1{wrong_char}");

        let err = resolve_task(&project, &bad_handle).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("check mismatch"),
            "error should mention mismatch: {msg}"
        );
        assert!(
            msg.contains("Alpha Task"),
            "error should name the title: {msg}"
        );
    }

    #[test]
    fn resolve_handle_out_of_range_errors() {
        let task_a = stub_task("alpha", "Alpha Task");
        let project = stub_project(vec![task_a]);
        let err = resolve_task(&project, "9z").unwrap_err();
        assert!(
            err.to_string().contains("out of range"),
            "error should say out of range: {err}"
        );
    }
}
