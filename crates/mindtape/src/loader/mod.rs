//! Discover the project root and load tasks by live-evaluating their files.
//!
//! # Files
//!
//! - `app_dirs.rs`: where `mt` keeps its per-user state.
//! - `cache.rs`: the persistent content-hash task cache.
//! - `derive.rs`: folder-derived virtual tags.
//! - `errors.rs`: the per-file failures and Typst warnings a load reports.
//! - `many.rs`, `many_tests.rs`: several projects loaded at once, each on its
//!   own thread.

pub mod app_dirs;
pub mod derive;
mod errors;
mod many;

pub use errors::{LoadError, LoadWarning};
pub use many::{Loaded, load_many};

use std::path::{Path, PathBuf};

use typst_harvest::{HVal, Harvest, HarvestError, HarvestWorld, SourceSnapshot, batch_harvest};

use crate::config::Config;
use crate::loader::derive::RootDeriver;
use crate::model::Task;
use crate::model::task_spans::TaskCallSpans;

// The persistent task cache is a loader implementation detail with no public
// API, so it lives as a private submodule here rather than on the crate's
// documented `lib.rs` surface.
pub(crate) mod cache;

use cache::{Cached, CachedOutcome, TaskCache};

/// The outcome of loading a project: the tasks found and any per-file errors.
#[derive(Debug, Clone, Default)]
pub struct LoadOutcome {
    /// Tasks successfully harvested, sorted by id.
    pub tasks: Vec<Task>,
    /// Files that could not be evaluated or harvested.
    pub errors: Vec<LoadError>,
    /// Typst warnings raised while evaluating files this run, in file order.
    /// A file served from the cache replays the ones stored with it.
    pub warnings: Vec<LoadWarning>,
}

/// Walk up from `start` for the project root (the dir containing `.mindtape/`).
#[must_use]
pub fn find_root(start: &Path) -> Option<PathBuf> {
    find_root_bounded(start, None)
}

/// [`find_root`] with the walk optionally bounded: `stop`, when given, is the
/// last directory examined.
///
/// Unbounded is what `mt` wants — a project marker may sit any distance above
/// the working directory. It is not what a test wants: "no marker anywhere
/// above this tempdir" is then a fact about the machine running the suite, and
/// a developer who keeps a `~/.mindtape` project with `$TMPDIR` under `$HOME`
/// fails an assertion the fixture looks like it controls.
fn find_root_bounded(start: &Path, stop: Option<&Path>) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join(".mindtape").is_dir() {
            return Some(dir);
        }
        // Two statements, not `if stop == dir || !dir.pop()`: a mutant that
        // turns that `||` into `&&` short-circuits `dir.pop()` away and loops
        // on the same directory forever (a reproducible TIMEOUT). Each step
        // here either returns or advances the walk.
        if Some(dir.as_path()) == stop {
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// One task file to harvest: its path, project-relative path, the tags its
/// folder location derives, and its nesting depth under the root.
struct LoadItem {
    file: PathBuf,
    rel: PathBuf,
    folder_tags: Vec<String>,
    depth: u8,
}

/// A cache miss to harvest: the item, the source already read for hashing (so a
/// hit and a miss both build the body without a second read), and the content
/// hash to key its fresh entry under. `hash` is `None` for an unreadable file —
/// the harvest surfaces the error and nothing is cached.
struct Miss<'a> {
    item: &'a LoadItem,
    source: String,
    hash: Option<String>,
}

/// Load every task under the configured roots.
///
/// The warm path stats and content-hashes each file and serves unchanged files
/// straight from the on-disk [`TaskCache`], skipping Typst evaluation entirely.
/// Only the cache misses (new or edited files) are live-evaluated, in parallel
/// ([`batch_harvest`]) over one shared [`SourceSnapshot`] — so a common `@local`
/// prelude is parsed once and evaluation fans out across cores.
#[must_use]
pub fn load_project(root: &Path, config: &Config) -> LoadOutcome {
    let overrides = package_overrides(root, config);
    let items = collect_items(root, config);
    let cache = TaskCache::open(root, config, &overrides);

    let mut outcome = LoadOutcome::default();
    let misses = split_hits(&items, &cache, &mut outcome);
    harvest_misses(&misses, root, &overrides, &cache, &mut outcome);

    outcome.tasks.sort_by(|left, right| left.id.cmp(&right.id));
    outcome
        .warnings
        .sort_by(|left, right| left.file.cmp(&right.file));
    // Graph-derived ranks (`topo`, `blocks`) are facts about the whole set, so
    // they are stamped on here — after every task is in and in id order — and
    // never per file. Both are cheap graph walks over already-parsed `links:`,
    // so no cache entry holds them: a task served from the cache gets the same
    // ranks a freshly harvested one does.
    crate::graph::dep_rank::annotate(&mut outcome.tasks, config);
    // A checker's answer is about now, not about the file, so it is asked on
    // every load and never cached.
    crate::waits::annotate(&mut outcome.tasks, config, root);
    outcome
}

/// Serve every cache hit into `outcome` and return the misses to harvest. Each
/// file is read once here: its bytes key the cache, and its source builds the
/// task body on a hit (no second read).
fn split_hits<'a>(
    items: &'a [LoadItem],
    cache: &TaskCache,
    outcome: &mut LoadOutcome,
) -> Vec<Miss<'a>> {
    let mut misses = Vec::new();
    for item in items {
        let source = std::fs::read_to_string(&item.file).ok();
        let hash = source
            .as_deref()
            .map(|src| TaskCache::content_hash(src.as_bytes()));
        match hash.as_deref().and_then(|key| cache.get(key)) {
            Some(hit) => replay(item, source.as_deref(), hit, outcome),
            None => misses.push(Miss {
                item,
                source: source.unwrap_or_default(),
                hash,
            }),
        }
    }
    misses
}

/// Serve one cache hit: replay its warnings against this file, and assemble
/// its task when it holds a marker.
fn replay(item: &LoadItem, source: Option<&str>, hit: Cached, outcome: &mut LoadOutcome) {
    outcome
        .warnings
        .extend(hit.warnings.into_iter().map(|warning| LoadWarning {
            file: item.rel.clone(),
            ..warning
        }));
    if let CachedOutcome::Marker(marker) = hit.outcome {
        let body = source.unwrap_or_default();
        outcome.tasks.push(assemble_task(item, &marker, body));
    }
}

/// Harvest the cache misses in parallel over one shared snapshot, record each
/// into `outcome`, and cache every clean outcome. Harvest errors are recorded
/// but never cached — the file is re-tried next run.
fn harvest_misses(
    misses: &[Miss],
    root: &Path,
    overrides: &[(String, PathBuf)],
    cache: &TaskCache,
    outcome: &mut LoadOutcome,
) {
    if misses.is_empty() {
        return;
    }
    let snapshot = SourceSnapshot::new();
    let results = batch_harvest(misses, |miss| {
        build_world(&miss.item.file, root, overrides)
            .map_err(HarvestError::World)
            .map(|world| world.with_shared_sources(&snapshot))
    });
    for (miss, result) in misses.iter().zip(results) {
        record_miss(miss, result, cache, outcome);
    }
}

/// Build a [`Task`] from a harvested marker and the file's source: identity from
/// the stem (an explicit `id:` still wins), folder-derived tags injected, and
/// the prose body sliced from the source. The one path both a cache hit and a
/// fresh harvest funnel through, so they cannot diverge.
fn assemble_task(item: &LoadItem, marker: &HVal, source: &str) -> Task {
    let id = file_stem(&item.file);
    let mut task = Task::from_marker(id, item.rel.clone(), marker);
    task.derived_tags = item.folder_tags.clone();
    task.depth = item.depth;
    task.spans = TaskCallSpans::parse(source);
    task.body = task.spans.body(source);
    task
}

/// Build a [`Task`] for a file at project-relative `rel` from an already
/// harvested `marker` and the source it came from — the [`assemble_task`] path
/// for a file that [`collect_items`] never walked, because it is not on disk
/// yet ([`crate::validate::candidate`]).
pub(crate) fn task_at(
    root: &Path,
    config: &Config,
    rel: &Path,
    marker: &HVal,
    source: &str,
) -> Task {
    assemble_task(&item_at(root, config, rel), marker, source)
}

/// The [`LoadItem`] a file at project-relative `rel` would be walked as: the
/// folder-derived tags and nesting depth of the first configured root that
/// contains it. A path under no root derives neither — the same tags and depth
/// a file written outside the roots would carry once it landed there, which is
/// nothing.
fn item_at(root: &Path, config: &Config, rel: &Path) -> LoadItem {
    let derived = config.tasks.roots.iter().find_map(|spec| {
        let rel_within = rel.strip_prefix(spec.path()).ok()?;
        let deriver = RootDeriver::new(spec, &config.tasks.derive_tags);
        Some((
            deriver.tags_for(rel_within),
            crate::loader::derive::depth(rel_within),
        ))
    });
    let (folder_tags, depth) = derived.unwrap_or_default();
    LoadItem {
        file: root.join(rel),
        rel: rel.to_path_buf(),
        folder_tags,
        depth,
    }
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
            let depth = crate::loader::derive::depth(rel_within);
            let rel = file.strip_prefix(root).unwrap_or(&file).to_path_buf();
            items.push(LoadItem {
                file,
                rel,
                folder_tags,
                depth,
            });
        }
    }
    items
}

/// Fold one harvested miss into `outcome` and update the cache. A clean harvest
/// caches its outcome (a `mindtape.task` marker, or its confirmed absence) and,
/// when a marker is present, assembles the [`Task`]; a build or eval failure
/// becomes a [`LoadError`] and is never cached.
fn record_miss(
    miss: &Miss,
    result: Result<Harvest, HarvestError>,
    cache: &TaskCache,
    outcome: &mut LoadOutcome,
) {
    match result {
        Ok(harvested) => {
            let warnings = LoadWarning::from_diagnostics(
                &miss.item.rel,
                &typst_path(&miss.item.rel),
                &harvested.warnings,
            );
            let marker = harvested
                .with_marker("mindtape.task")
                .next()
                .map(|found| &found.value);
            if let Some(hash) = &miss.hash {
                cache.put(hash, marker, &warnings, &harvested.dependencies);
            }
            if let Some(marker) = marker {
                outcome
                    .tasks
                    .push(assemble_task(miss.item, marker, &miss.source));
            }
            outcome.warnings.extend(warnings);
        }
        Err(err) => {
            if let HarvestError::Eval { error, warnings } = &err {
                outcome.warnings.extend(LoadWarning::from_diagnostics(
                    &miss.item.rel,
                    &error.main,
                    warnings,
                ));
            }
            outcome.errors.push(LoadError::harvest(
                miss.item.rel.clone(),
                &err,
                &miss.source,
            ));
        }
    }
}

/// `rel` in Typst's path form: `/`-separated on every platform.
fn typst_path(rel: &Path) -> String {
    rel.to_string_lossy().replace('\\', "/")
}

fn build_world(
    file: &Path,
    root: &Path,
    overrides: &[(String, PathBuf)],
) -> Result<HarvestWorld, String> {
    let mut world = HarvestWorld::with_root(file, root)
        .map_err(|err| err.to_string())?
        .with_package_root(package_root());
    for (name, dir) in overrides {
        world = world.with_local_package(name, dir.clone());
    }
    Ok(world)
}

/// The directory an unoverridden `@local/<name>` resolves under: the app's own
/// data directory (`$MINDTAPE_DATA_HOME`, else the platform one) joined with
/// Typst's `typst/packages` layout.
///
/// Named rather than left to Typst's own lookup because that lookup goes
/// through `dirs`, which reads the environment on Unix only — on Windows it
/// calls a known-folder API no variable can move. `mt install` already writes
/// through [`crate::data_home`], so leaving the read on the platform default
/// let the two disagree: point `$MINDTAPE_DATA_HOME` at a directory and the
/// package landed there while every task file kept importing the machine's own.
///
/// `None` — the variable set but empty, which
/// [`crate::app_dirs`] defines as "no such directory" — resolves no global
/// package at all rather than falling back, so an explicit disable is honoured
/// on a read the same way it already is on a write.
pub fn package_root() -> Option<PathBuf> {
    crate::data_home().map(|dir| dir.join("typst").join("packages"))
}

/// The `@local/<name>` → in-tree lib overrides for this project: Mindtape's own
/// package when `<root>/lib` is the `mindtape` package, plus every configured
/// `[[typst.local-packages]]` entry (which wins on a name clash). Resolving by
/// the `lib/` manifest name — not its mere presence — keeps `mindtape` from
/// hijacking a foreign worktree whose `lib/` is some other package.
///
/// Public because it is the project's single answer to "which `@local`
/// packages resolve here", and every world Mindtape builds has to give the
/// same one. `mt eval` builds its own world outside this loader and used to
/// decide for itself, off the mere existence of `lib/typst.toml`; the two
/// drifted, and `mt eval` alone failed in any worktree whose `lib/` was a
/// foreign package
/// (`truth/tasks/resolve-the-mindtape-package-by-manifest-name-in-mt-eval.typ`).
pub fn package_overrides(root: &Path, config: &Config) -> Vec<(String, PathBuf)> {
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
mod tests;

#[cfg(test)]
mod tests_cache;

#[cfg(test)]
mod tests_prelude;

#[cfg(test)]
mod tests_eval_line;
