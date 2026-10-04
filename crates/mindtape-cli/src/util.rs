//! Shared project discovery for the CLI commands.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use mindtape::{
    Config, Filter, LoadOutcome, Resolution, StemHashes, Task, ViewSpec, config_path, find_root,
    load_config, load_project, write_atomic,
};

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

/// The stem hash of every task the project loaded, closed ones included, so a
/// task carries the same hash in every listing. A hash never equals a loaded
/// stem or a word [`Filter::names_term`] accepts, so neither captures it.
#[must_use]
pub fn stem_hashes(tasks: &[Task], config: &Config) -> StemHashes {
    StemHashes::reserving(tasks.iter().map(|task| task.id.as_str()), |word| {
        Filter::names_term(word, config)
    })
}

/// Resolve a task reference to a loaded `Task` and its absolute path.
///
/// A reference spelled as a path (absolute, holding `/`, or ending `.typ`)
/// resolves as a path from the working directory. Anything else is an exact
/// filename stem first, and a stem hash ([`mindtape::handle`]) only when no
/// stem matches.
///
/// # Errors
/// Returns an error if the path cannot be resolved, a stem hash names several
/// tasks, or no task matches.
pub fn resolve_task(project: &Project, id: &str) -> Result<(Task, PathBuf)> {
    let path = Path::new(id);
    if path.is_absolute() || id.contains('/') || id.ends_with(".typ") {
        return resolve_path(project, path);
    }
    let tasks = &project.outcome.tasks;
    let found = |stem: &str| tasks.iter().find(|task| task.id == stem);
    let stem = if found(id).is_some() {
        id.to_owned()
    } else {
        let hashes = stem_hashes(tasks, &project.config);
        match hashes.resolve(id) {
            Resolution::Found(stem) => stem,
            Resolution::Ambiguous(stems) => bail!(ambiguous(&hashes, id, &stems)),
            Resolution::NoMatch => bail!(no_task(project, id)),
        }
    };
    let task = found(&stem).with_context(|| no_task(project, id))?;
    Ok((task.clone(), project.root.join(&task.file)))
}

/// Resolve a reference spelled as a path, from the working directory.
fn resolve_path(project: &Project, path: &Path) -> Result<(Task, PathBuf)> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("determine current directory")?
            .join(path)
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
    Ok((task.clone(), abs))
}

/// The refusal for a reference no stem or stem hash matches.
fn no_task(project: &Project, id: &str) -> String {
    let ids: Vec<&str> = project
        .outcome
        .tasks
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    format!(
        "no task with id {id:?}{}",
        mindtape::suggest::suggestion(id, &ids)
    )
}

/// The refusal for a stem hash several tasks share, naming each by stem and hash.
fn ambiguous(hashes: &StemHashes, id: &str, stems: &[String]) -> String {
    let candidates: Vec<String> = stems
        .iter()
        .map(|stem| format!("  {} {stem}", hashes.hash_of(stem).unwrap_or_default()))
        .collect();
    format!(
        "stem hash {id:?} names {} tasks; pass a longer hash or the stem:\n{}",
        stems.len(),
        candidates.join("\n")
    )
}

/// Print per-file load errors to stderr as warnings, then whichever remedies
/// those errors earn — `mt install` for an unresolved prelude, `\#` for a
/// literal hash read as markup — each once for the whole run.
pub fn warn_errors(outcome: &LoadOutcome) {
    for error in &outcome.errors {
        eprintln!("warning: {}: {}", error.file.display(), error.message);
    }
    for hint in crate::install_hint::load_hints(outcome) {
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
    write_atomic(path, canonical_source(config, source).as_bytes())
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

/// The path of `to` (root-relative) seen from `from_dir` (root-relative), for
/// writing a link target: `tasks/live.typ` pointing at
/// `tasks/.archive/gone.typ` becomes `.archive/gone.typ`. Shared by `retire`
/// moving a file and `mt link` naming a task in another folder.
pub fn relative_target(from_dir: &Path, to: &Path) -> PathBuf {
    let from: Vec<_> = from_dir.components().collect();
    let to: Vec<_> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in 0..(from.len() - common) {
        out.push("..");
    }
    for component in to.iter().skip(common) {
        out.push(component.as_os_str());
    }
    out
}

#[cfg(test)]
#[path = "util_tests.rs"]
mod tests;
