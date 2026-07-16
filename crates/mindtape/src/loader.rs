//! Discover the project root and load tasks by live-evaluating their files.

use std::path::{Path, PathBuf};

use typst_harvest::{HVal, Harvest, HarvestError, HarvestWorld, SourceSnapshot, batch_harvest};

use crate::config::Config;
use crate::derive::RootDeriver;
use crate::model::Task;

/// A per-file failure encountered while loading the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    /// File that failed, relative to the project root when possible.
    pub file: PathBuf,
    /// Human-readable cause.
    pub message: String,
}

/// The outcome of loading a project: the tasks found and any per-file errors.
#[derive(Debug, Clone, Default)]
pub struct LoadOutcome {
    /// Tasks successfully harvested, sorted by id.
    pub tasks: Vec<Task>,
    /// Files that could not be evaluated or harvested.
    pub errors: Vec<LoadError>,
}

/// Walk up from `start` for the project root (the dir containing `.mindtape/`).
#[must_use]
pub fn find_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join(".mindtape").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// One task file to harvest: its path, project-relative path, and the tags its
/// folder location derives.
struct LoadItem {
    file: PathBuf,
    rel: PathBuf,
    folder_tags: Vec<String>,
}

/// Load every task under the configured roots, live-evaluating each file.
///
/// Files are harvested in parallel ([`batch_harvest`]) over one shared
/// [`SourceSnapshot`], so a common `@local` prelude is parsed once for the whole
/// backlog rather than re-parsed per file, and evaluation fans out across cores.
#[must_use]
pub fn load_project(root: &Path, config: &Config) -> LoadOutcome {
    let overrides = package_overrides(root, config);
    let items = collect_items(root, config);
    let snapshot = SourceSnapshot::new();
    let results = batch_harvest(&items, |item| {
        build_world(&item.file, root, &overrides)
            .map_err(HarvestError::World)
            .map(|world| world.with_shared_sources(&snapshot))
    });

    let mut outcome = LoadOutcome::default();
    for (item, result) in items.iter().zip(results) {
        record_result(item, result, &mut outcome);
    }
    outcome.tasks.sort_by(|left, right| left.id.cmp(&right.id));
    outcome
}

/// Every `.typ` file under the configured roots, paired with its
/// project-relative path and the tags its folder location derives.
fn collect_items(root: &Path, config: &Config) -> Vec<LoadItem> {
    let mut items = Vec::new();
    for spec in &config.tasks.roots {
        let dir = root.join(spec.path());
        let deriver = RootDeriver::new(spec, &config.tasks.derive_tags);
        let mut files = Vec::new();
        collect_typ_files(&dir, &mut files);
        files.sort();
        for file in files {
            let rel_within = file.strip_prefix(&dir).unwrap_or(&file);
            let folder_tags = deriver.tags_for(rel_within);
            let rel = file.strip_prefix(root).unwrap_or(&file).to_path_buf();
            items.push(LoadItem {
                file,
                rel,
                folder_tags,
            });
        }
    }
    items
}

/// Fold one harvested file into `outcome`: a `mindtape.task` marker becomes a
/// [`Task`]; a build or eval failure becomes a [`LoadError`] in that file's slot.
fn record_result(
    item: &LoadItem,
    result: Result<Harvest, HarvestError>,
    outcome: &mut LoadOutcome,
) {
    match result {
        Ok(harvested) => {
            if let Some(marker) = harvested.with_marker("mindtape.task").next() {
                let id = file_stem(&item.file);
                let mut task = Task::from_marker(id, item.rel.clone(), &marker.value);
                task.derived_tags = item.folder_tags.clone();
                task.body = read_body(&item.file);
                outcome.tasks.push(task);
            }
        }
        Err(err) => outcome.errors.push(LoadError {
            file: item.rel.clone(),
            message: err.to_string(),
        }),
    }
}

fn build_world(
    file: &Path,
    root: &Path,
    overrides: &[(String, PathBuf)],
) -> Result<HarvestWorld, String> {
    let mut world = HarvestWorld::with_root(file, root).map_err(|err| err.to_string())?;
    for (name, dir) in overrides {
        world = world.with_local_package(name, dir.clone());
    }
    Ok(world)
}

/// The `@local/<name>` → in-tree lib overrides for this project: `MindTape`'s own
/// package when `<root>/lib` is the `mindtape` package, plus every configured
/// `[[typst.local-packages]]` entry (which wins on a name clash). Resolving by
/// the `lib/` manifest name — not its mere presence — keeps `mindtape` from
/// hijacking a foreign worktree whose `lib/` is some other package.
fn package_overrides(root: &Path, config: &Config) -> Vec<(String, PathBuf)> {
    let mut overrides = Vec::new();
    if let Some(lib) = mindtape_lib_dir(root) {
        overrides.push(("mindtape".to_owned(), lib));
    }
    for pkg in &config.typst.local_packages {
        overrides.push((pkg.name.clone(), root.join(&pkg.path)));
    }
    overrides
}

/// `<root>/lib` only when its `typst.toml` declares `name = "mindtape"`.
fn mindtape_lib_dir(root: &Path) -> Option<PathBuf> {
    let lib = root.join("lib");
    let text = std::fs::read_to_string(lib.join("typst.toml")).ok()?;
    let doc: toml::Table = toml::from_str(&text).ok()?;
    let name = doc.get("package")?.get("name")?.as_str()?;
    (name == "mindtape").then_some(lib)
}

fn file_stem(file: &Path) -> String {
    file.file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}

/// The prose body of `file`: the raw source after the `#show: task.with(…)`
/// marker, whitespace-trimmed. Empty when the file is unreadable (the harvest
/// already succeeded here, so this is only defensive) or holds no `task.with`
/// call. Sourced from disk rather than the harvest so the emitter can append it
/// back verbatim for a stable `ls --json | import` round-trip.
fn read_body(file: &Path) -> String {
    std::fs::read_to_string(file)
        .map(|source| body_after_marker(&source))
        .unwrap_or_default()
}

/// The source after the first `task.with(…)` call's closing paren, trimmed.
/// Parse-plane only (no eval): [`typst_edit::find_method_calls`] locates the
/// call and `Call::range` spans it through the closing `)`, so the tail is the
/// authored body. Trimming drops the blank line the emitter re-adds between the
/// marker and the body, keeping the round-trip stable.
fn body_after_marker(source: &str) -> String {
    typst_edit::find_method_calls(source, "task", "with")
        .into_iter()
        .next()
        .and_then(|call| source.get(call.range.end..))
        .unwrap_or_default()
        .trim()
        .to_owned()
}

fn collect_typ_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_typ_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("typ") {
            out.push(path);
        }
    }
}

/// The first `mindtape.task` marker value in a harvest, if any.
#[must_use]
pub fn first_task_marker(markers: &[typst_harvest::Marker]) -> Option<&HVal> {
    markers
        .iter()
        .map(|marker| &marker.value)
        .find(|value| value.get("marker").and_then(HVal::as_str) == Some("mindtape.task"))
}

#[cfg(test)]
#[path = "loader_tests.rs"]
mod tests;
