//! Shared project discovery for the CLI commands.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};
use mindtape::{
    Config, Filter, LoadOutcome, Readiness, Task, config_path, find_root, load_config,
    load_project, select,
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

/// Process-global worktree override, resolved once from the `-w`/`--worktree`
/// flag and `$MINDTAPE_WORKTREE`. `None` means "discover by walking up".
static WORKTREE: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Resolve and cache the worktree override for this process.
///
/// `flag` is the `-w`/`--worktree` value; the flag wins over the
/// `MINDTAPE_WORKTREE` environment variable, which is the fallback. Call once
/// from `main`; later calls are ignored.
pub fn init_worktree(flag: Option<PathBuf>) {
    let _ = WORKTREE.set(pick_override(flag, std::env::var_os("MINDTAPE_WORKTREE")));
}

/// Pick the effective worktree override: the flag if present, else a non-empty
/// `MINDTAPE_WORKTREE` value. An empty env var counts as unset.
fn pick_override(flag: Option<PathBuf>, env: Option<OsString>) -> Option<PathBuf> {
    flag.or_else(|| env.filter(|value| !value.is_empty()).map(PathBuf::from))
}

/// The cached override, or `None` if [`init_worktree`] never ran (e.g. tests).
fn worktree_override() -> Option<PathBuf> {
    WORKTREE.get().cloned().flatten()
}

/// Resolve the project root, honouring the worktree override when set.
///
/// With no override, walks up from the current directory for a `.mindtape/`
/// ancestor. With an override, resolves it against the current directory
/// (absolute paths as-is) and requires a `.mindtape/` directory there.
///
/// # Errors
/// Returns an error if the current directory is unreadable, no project is
/// found, or the override path lacks a `.mindtape/` directory.
pub fn resolve_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("determine current directory")?;
    resolve_root_with(&cwd, worktree_override().as_deref())
}

/// Root-resolution core, with `cwd` and the override passed explicitly so the
/// logic is testable without touching process-global state.
fn resolve_root_with(cwd: &Path, override_path: Option<&Path>) -> Result<PathBuf> {
    let Some(path) = override_path else {
        return find_root(cwd)
            .context("not inside a MindTape project (no .mindtape/ directory found)");
    };
    let base = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    if !base.join(".mindtape").is_dir() {
        bail!(
            "worktree {} is not a MindTape project (no .mindtape/ directory)",
            base.display()
        );
    }
    base.canonicalize()
        .with_context(|| format!("resolve worktree {}", base.display()))
}

/// Find the project from the current directory (or the worktree override) and
/// load all its tasks.
///
/// # Errors
/// Returns an error if no `.mindtape/` project is found or the config is invalid.
pub fn open() -> Result<Project> {
    let root = resolve_root()?;
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
///   `mt ls` shows).  The ordinal is 1-based.  The check letter must
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

/// Build the default list view: [`crate::ls_rules::resolve_bare`]'s bundle
/// (the first matching `[[ls.rules]]` entry, else the default view, else
/// match-all), filtered then sorted the same way `ls.rs` does. This
/// replicates exactly what bare `mt ls` shows — same resolution, same filter
/// and sort — so handle ordinals resolve to the same tasks the user reads
/// off it. Columns play no part; they don't affect which tasks appear or in
/// what order.
fn default_view(project: &Project) -> Result<Vec<Task>> {
    let (bundle, _applied) = crate::ls_rules::resolve_bare(project)?;
    let filter = Filter::parse(&bundle.filter)?;
    // Readiness over the full set, before the filter narrows the view, so a
    // dependency hidden by the default filter still counts. Shared by select and
    // ordering so the handle view matches what `list` prints exactly.
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let mut tasks = select(
        project.outcome.tasks.clone(),
        Some(&filter),
        &project.config,
        &readiness,
    )?;
    crate::ls::sort_tasks(&mut tasks, &bundle.sort, &project.config, &readiness)?;
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
    use std::path::Path;

    use mindtape::test_support::{TASK_SRC, bare_task};
    use mindtape::{Config, LoadError, LoadOutcome, Task, load_project};

    use super::{Project, pick_override, resolve_root_with, resolve_task, warn_errors};
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
        let mut t = bare_task(id);
        t.title = title.to_owned();
        t
    }

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
    fn pick_override_flag_wins_over_env() {
        let picked = pick_override(Some("/flag/path".into()), Some("/env/path".into()));
        assert_eq!(picked, Some(std::path::PathBuf::from("/flag/path")));
    }

    #[test]
    fn pick_override_falls_back_to_env() {
        let picked = pick_override(None, Some("/env/path".into()));
        assert_eq!(picked, Some(std::path::PathBuf::from("/env/path")));
    }

    #[test]
    fn pick_override_empty_env_is_unset() {
        assert_eq!(pick_override(None, Some(String::new().into())), None);
        assert_eq!(pick_override(None, None), None);
    }

    #[test]
    fn resolve_root_override_targets_named_worktree() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
        // cwd is elsewhere; the absolute override still selects the worktree.
        let cwd = tempfile::tempdir().unwrap();
        let root = resolve_root_with(cwd.path(), Some(dir.path())).unwrap();
        assert_eq!(root, dir.path().canonicalize().unwrap());
    }

    #[test]
    fn resolve_root_override_relative_resolves_against_cwd() {
        let cwd = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(cwd.path().join("wt").join(".mindtape")).unwrap();
        let root = resolve_root_with(cwd.path(), Some(Path::new("wt"))).unwrap();
        assert_eq!(root, cwd.path().join("wt").canonicalize().unwrap());
    }

    #[test]
    fn resolve_root_override_without_marker_errors() {
        let dir = tempfile::tempdir().unwrap();
        let err = resolve_root_with(dir.path(), Some(dir.path())).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("no .mindtape/ directory"),
            "error should name the missing marker: {msg}"
        );
    }

    #[test]
    fn resolve_root_no_override_walks_up() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
        let child = dir.path().join("a").join("b");
        std::fs::create_dir_all(&child).unwrap();
        let root = resolve_root_with(&child, None).unwrap();
        assert_eq!(root, dir.path());
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
