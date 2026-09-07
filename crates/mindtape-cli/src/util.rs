//! Shared project discovery for the CLI commands.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use mindtape::{
    Config, Filter, LoadOutcome, Readiness, Task, ViewSpec, config_path, find_root, load_config,
    load_project, select,
};

use crate::handle::{check_chars, check_len, parse_handle};
use crate::render::link::HyperlinkChoice;

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

/// Process-global `--hyperlinks` flag, resolved once from `main`. `None`
/// means the flag was not passed, deferring to `[ls] hyperlinks` — unlike
/// `WORKTREE`, this cannot be resolved to a final on/off decision until a
/// project's config is loaded, which is why [`crate::render::link::init`]
/// itself runs from [`open`] rather than from `main`.
static HYPERLINKS_FLAG: OnceLock<Option<HyperlinkChoice>> = OnceLock::new();

/// Record the `--hyperlinks` flag for this process. Call once from `main`;
/// later calls are ignored.
pub fn init_hyperlinks_flag(flag: Option<HyperlinkChoice>) {
    let _ = HYPERLINKS_FLAG.set(flag);
}

/// The cached flag, or `None` if [`init_hyperlinks_flag`] never ran (e.g.
/// tests) or the flag itself was not passed.
fn hyperlinks_flag() -> Option<HyperlinkChoice> {
    HYPERLINKS_FLAG.get().copied().flatten()
}

/// Resolve the effective `--hyperlinks` choice: the flag wins over `[ls]
/// hyperlinks` (which itself defaults to `never`) when passed.
fn resolve_hyperlinks(
    flag: Option<HyperlinkChoice>,
    config_choice: mindtape::HyperlinkChoice,
) -> HyperlinkChoice {
    flag.unwrap_or_else(|| config_choice.into())
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
/// Also records the `(root, cwd)` pair for [`crate::render::file_path`]'s
/// `file` column, mirroring the `WORKTREE`/`COLOR_ENABLED` one-shot pattern; a
/// cwd read failure is silently skipped, leaving the column's root-relative
/// fallback in place rather than failing project discovery over it. Resolves
/// and caches the hyperlink decision too ([`resolve_hyperlinks`],
/// [`crate::render::link::init`]) — it has to happen here rather than `main`,
/// since it also consults this project's `[ls] hyperlinks`.
///
/// # Errors
/// Returns an error if no `.mindtape/` project is found or the config is invalid.
pub fn open() -> Result<Project> {
    let root = resolve_root()?;
    if let Ok(cwd) = std::env::current_dir() {
        crate::render::file_path::init(&root, &cwd);
    }
    let config =
        load_config(&root).with_context(|| format!("load {}", config_path(&root).display()))?;
    crate::render::link::init(resolve_hyperlinks(hyperlinks_flag(), config.ls.hyperlinks));
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
///
/// Shared with [`crate::flip_handles`], which reports what a flip did to those
/// ordinals: it has to ask the same question this resolver answers, or it
/// would print a handle the next command refuses.
///
/// # Errors
/// Returns an error if the resolved filter or sort spec fails to parse.
pub(crate) fn default_view(project: &Project) -> Result<Vec<Task>> {
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
    // Length first, and strictly: under `[handle] check-len = 2` a bare `2y` is
    // not a short spelling of `2yq`, it is a handle that belongs to whichever
    // task hashes to `y` at length 1 — a different question with a different
    // answer. Prefix-matching it would hand back the collision rate the project
    // raised the length to avoid.
    let len = check_len(&project.config);
    let expected_check = check_chars(&task.id, len);
    let given = check.chars().count();
    if given != len {
        bail!(
            "handle {ordinal}{check} carries {given} check character(s); this project's handles \
             carry {len} ({ordinal}{expected_check} is the handle at that position, {title:?}). \
             A handle of another length is refused, never matched as a prefix",
            title = task.title,
        );
    }
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

/// Print per-file load errors to stderr as warnings, then — when the errors say
/// the prelude is not installed — one hint naming `mt install`.
pub fn warn_errors(outcome: &LoadOutcome) {
    for error in &outcome.errors {
        eprintln!("warning: {}: {}", error.file.display(), error.message);
    }
    if let Some(hint) = crate::install_hint::missing_prelude_hint(outcome) {
        eprintln!("{hint}");
    }
}

/// Canonicalize task source before it lands on disk or stdout. Runs typstyle
/// (idempotent, so a clean file is unchanged beyond the edited span) when
/// `[format] enabled` — the default — else passes the surgical edit through
/// verbatim. Every `mt` write path funnels through here so a fill or injected
/// argument never leaves non-canonical whitespace behind.
#[must_use]
pub fn canonical_source(config: &Config, source: String) -> String {
    if config.format.enabled {
        mindtape::format_typst(&source)
    } else {
        source
    }
}

/// [`canonical_source`], then write the result to `path`.
///
/// # Errors
/// Returns an error when the file cannot be written.
pub fn write_task(config: &Config, path: &Path, source: String) -> Result<()> {
    std::fs::write(path, canonical_source(config, source))
        .with_context(|| format!("write {}", path.display()))
}

/// Build the `unknown view: <name>` error, extended with a nearest-match
/// suggestion and the full view list
/// (`truth/tasks/suggest-a-nearest-match-on-an-unknown-view-name.typ`).
/// Shared by every CLI view lookup site (`ls_rules.rs`, `views.rs`) so the
/// message and its hint composition are not duplicated across them.
#[must_use]
pub fn unknown_view_error(name: &str, views: &BTreeMap<String, ViewSpec>) -> anyhow::Error {
    anyhow!(
        "unknown view: {name}{}",
        mindtape::suggest::hint("view", name, views.keys())
    )
}

#[cfg(test)]
#[path = "util_tests.rs"]
mod tests;
