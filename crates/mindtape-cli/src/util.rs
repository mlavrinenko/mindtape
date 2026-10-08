//! Shared project discovery for the CLI commands.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;
use mindtape::{
    Config, Filter, LoadError, LoadOutcome, LoadWarning, RefError, StemHashes, Task, ViewSpec,
    config_path, find_root, load_config, load_project, write_atomic,
};

use crate::cmd::install::hint::{
    escaped_hash_hint, is_unknown_variable, names_prelude_package, prelude_diag,
};
use crate::diag::{Command, Diag, Hint, Kind, Note, emit};
use crate::render::link::HyperlinkChoice;

/// A discovered, loaded project: its root, config and load outcome.
#[derive(Debug)]
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

/// The working directory, or the refusal when the OS will not say.
fn current_dir() -> Result<PathBuf> {
    std::env::current_dir().map_err(|err| {
        Diag::failed("can't determine the current directory")
            .io(&err)
            .into()
    })
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
    let cwd = current_dir()?;
    resolve_root_with(&cwd, worktree_override().as_deref())
}

/// Root-resolution core, with `cwd` and the override passed explicitly so the
/// logic is testable without touching process-global state.
fn resolve_root_with(cwd: &Path, override_path: Option<&Path>) -> Result<PathBuf> {
    let Some(path) = override_path else {
        return find_root(cwd).ok_or_else(|| {
            Diag::failed("no .mindtape/ directory here or above")
                .hint(Hint::new(
                    "run mt init to make one here, or pass -w <path> to use another project",
                ))
                .into()
        });
    };
    let base = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    if !base.join(".mindtape").is_dir() {
        return Err(Diag::failed("no .mindtape/ directory in worktree")
            .value(base.display().to_string())
            .hint(Hint::new("run mt init there, or pass another path to -w"))
            .into());
    }
    base.canonicalize().map_err(|err| {
        Diag::failed("can't resolve worktree")
            .value(base.display().to_string())
            .io(&err)
            .into()
    })
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
    let config = load_config(&root)?;
    warn_config(&root, &config);
    crate::render::link::init(resolve_hyperlinks(hyperlinks_flag(), config.ls.hyperlinks));
    let outcome = load_project(&root, &config);
    Ok(Project {
        root,
        config,
        outcome,
    })
}

/// Print each load warning of `config` to stderr, naming its config file.
fn warn_config(root: &Path, config: &Config) {
    for diag in config_warnings(root, config) {
        emit(&diag);
    }
}

/// One `warning:` per load warning of `config`, placed at its config file.
fn config_warnings(root: &Path, config: &Config) -> Vec<Diag> {
    let file = config_path(root);
    let shown = file.strip_prefix(root).unwrap_or(&file);
    config
        .warnings()
        .into_iter()
        .map(|warning| Diag::verbatim(Kind::Warning, warning).at_file(shown))
        .collect()
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

/// Resolve a task reference to a loaded `Task` and its absolute path, with
/// [`mindtape::resolve_ref`]: a path from the working directory, else an exact
/// stem, and a stem hash only when no stem matches.
///
/// # Errors
/// Returns an error if the path cannot be resolved, a stem hash names several
/// tasks, or no task matches.
pub fn resolve_task(project: &Project, id: &str) -> Result<(Task, PathBuf)> {
    let tasks = &project.outcome.tasks;
    let cwd = if mindtape::is_path_ref(id) {
        current_dir()?
    } else {
        PathBuf::new()
    };
    let hashes = stem_hashes(tasks, &project.config);
    match mindtape::resolve_ref(tasks, &project.root, &cwd, &hashes, id) {
        Ok(found) => Ok((found.task.clone(), found.path)),
        Err(RefError::Ambiguous { stems, .. }) => Err(ambiguous(&hashes, id, &stems).into()),
        Err(RefError::NoMatch(_)) => Err(no_task(project, id).into()),
        Err(other) => Err(Diag::from(other).into()),
    }
}

/// The refusal for a reference no stem or stem hash matches: the nearest stem
/// when one is close, else where to list them.
fn no_task(project: &Project, id: &str) -> Diag {
    let ids: Vec<&str> = project
        .outcome
        .tasks
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    let diag = Diag::failed("no task").value(id);
    match mindtape::suggest::nearest(id, &ids) {
        Some(near) => diag.hint(Hint::did_you_mean(near)),
        None => diag.hint(Hint::run(Command::mt().word("ls"), "to see your tasks")),
    }
}

/// The refusal for a stem hash several tasks share, naming each by stem and hash.
fn ambiguous(hashes: &StemHashes, id: &str, stems: &[String]) -> Diag {
    stems.iter().fold(
        Diag::failed("stem hash names several tasks")
            .value(id)
            .hint(Hint::new("pass a longer hash or the stem")),
        |diag, stem| {
            diag.note(Note::new("candidate").name(format!(
                "{} {stem}",
                hashes.hash_of(stem).unwrap_or_default()
            )))
        },
    )
}

/// Print the load problems of `outcome` to stderr as warnings: the command
/// goes on without the files that failed. See [`report_load_problems`].
pub fn warn_errors(outcome: &LoadOutcome) {
    report_load_problems(outcome, Kind::Warning);
}

/// Print each load problem of `outcome` to stderr, then each Typst warning.
///
/// `kind` is how a file that failed to load reads: [`Kind::Warning`] for a
/// command that lists what did load, [`Kind::Failed`] for one that fails on it
/// (`mt check`). Typst's message, its place and its hints come through as
/// Typst words them. Two project-wide failures are said once, not per file: a
/// prelude that does not resolve is one error naming the install command and
/// the config key, and the escaped-hash advice rides on the first body that
/// needs it.
pub fn report_load_problems(outcome: &LoadOutcome, kind: Kind) {
    let mut prelude_missing = false;
    let mut hash_hinted = false;
    for error in &outcome.errors {
        if names_prelude_package(&error.message) {
            if !prelude_missing {
                emit(&prelude_diag(kind, &error.message));
            }
            prelude_missing = true;
            continue;
        }
        let mut diag = load_diag(kind, error);
        if !hash_hinted && is_unknown_variable(&error.message) {
            hash_hinted = true;
            diag = diag.hint(escaped_hash_hint());
        }
        emit(&diag);
    }
    for warning in &outcome.warnings {
        emit(&warning_diag(warning));
    }
}

/// The line for a file that failed to load: its place as `file:line:col`
/// (less when Typst gave less), Typst's message, Typst's hints.
fn load_diag(kind: Kind, error: &LoadError) -> Diag {
    at_place(
        Diag::verbatim(kind, &error.message),
        &error.file,
        error.line,
        error.column,
    )
    .typst_hints(error.hints.iter().cloned())
}

/// The line for a Typst warning on a file that loaded.
fn warning_diag(warning: &LoadWarning) -> Diag {
    at_place(
        Diag::verbatim(Kind::Warning, &warning.message),
        &warning.file,
        warning.line,
        warning.column,
    )
    .typst_hints(warning.hints.iter().cloned())
}

/// `diag` placed at `file:line:col`, or at the bare file when no line is known.
fn at_place(diag: Diag, file: &Path, line: Option<usize>, column: Option<usize>) -> Diag {
    match line {
        Some(line) => diag.at(file, line, column),
        None => diag.at_file(file),
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
        .map_err(|err| Diag::cannot_write(path, &err).into())
}

/// The refusal for a view name the config does not declare: the nearest view
/// when one is close, else where to list them. Shared by every CLI view lookup
/// site (`cmd/ls/rules.rs`, `cmd/views.rs`).
pub fn unknown_view_error(name: &str, views: &BTreeMap<String, ViewSpec>) -> anyhow::Error {
    let known: Vec<&str> = views.keys().map(String::as_str).collect();
    let diag = Diag::usage("unknown view").value(name);
    match mindtape::suggest::nearest(name, &known) {
        Some(near) => diag.hint(Hint::did_you_mean(near)),
        None => diag.hint(Hint::run(Command::mt().word("views"), "to see the views")),
    }
    .into()
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
