//! Discover the project root and load tasks by live-evaluating their files.
//!
//! # Files
//!
//! - `app_dirs.rs`: where `mt` keeps its per-user state.
//! - `cache.rs`: the persistent content-hash task cache.
//! - `derive.rs`: folder-derived virtual tags.
//! - `embedded.rs`, `embedded_tests.rs`: the prelude packages this crate
//!   embeds, one immutable copy per supported version under `preludes/`.
//! - `errors.rs`: the per-file failures and Typst warnings a load reports.
//! - `many.rs`, `many_tests.rs`: several projects loaded at once, each on its
//!   own thread.
//! - `pins.rs`: the version-aware `[[typst.local-packages]]` provider.

pub mod app_dirs;
pub mod derive;
mod embedded;
mod errors;
mod many;
mod pins;

pub(crate) use pins::StatePackages;

pub use app_dirs::package_root;
pub use embedded::{
    PRELUDE, PRELUDE_SNAPSHOTS, PRELUDE_SUPPORTED, PRELUDE_VERSION, PreludeSnapshot,
    prelude_is_older, prelude_manifest, prelude_snapshot, prelude_supports,
};
pub use errors::{LoadError, LoadWarning};
pub use many::{Loaded, load_many, map_parallel};
pub use pins::unsupported_prelude_version;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use typst_harvest::{HVal, Harvest, HarvestError, HarvestWorld, SourceSnapshot, batch_harvest};

use crate::config::Config;
use crate::loader::derive::RootDeriver;
use crate::model::Task;
use crate::model::task_spans::TaskCallSpans;
use crate::slash_path;

// The persistent task cache is a loader implementation detail with no public
// API, so it lives as a private submodule here rather than on the crate's
// documented `lib.rs` surface.
pub(crate) mod cache;

#[cfg(test)]
mod tests_one;

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
    let mut outcome = harvest_items(&collect_items(root, config), root, config);
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
    crate::estimate_queries::annotate(&mut outcome.tasks, config, root);
    crate::collect::annotate(&mut outcome.tasks, config);
    outcome
}

/// Load the one task file at `path`, project-relative or absolute, under
/// `config`, through the same content-hash cache [`load_project`] uses, with
/// its waits and collectors annotated as [`load_project`] does. The fields
/// which need the whole set (`topo`, `blocks`, `children`, `progress`) stay
/// empty: no other task is loaded to derive them from.
///
/// `Ok(None)` is a file with no `task.with` call, which [`load_project`] skips
/// silently. A load failure is the [`LoadError`] [`load_project`] would list.
///
/// # Errors
///
/// The file's load failure, or a path that is not a `.typ` file inside a
/// configured task root: a file there would carry no folder tags and so
/// differ from its [`load_project`] task.
pub fn load_one(root: &Path, config: &Config, path: &Path) -> Result<Option<Task>, LoadError> {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let in_a_root = config.tasks.roots.iter().any(|spec| {
        let dir = spec.dir();
        dir.is_empty() || rel.strip_prefix(&dir).is_ok()
    });
    if !in_a_root || rel.extension().and_then(|ext| ext.to_str()) != Some("typ") {
        return Err(LoadError {
            file: rel.to_path_buf(),
            message: "not a `.typ` file inside a task root".to_owned(),
            ..LoadError::default()
        });
    }
    let mut outcome = harvest_items(&[item_at(root, config, rel)], root, config);
    if let Some(error) = outcome.errors.into_iter().next() {
        return Err(error);
    }
    crate::waits::annotate(&mut outcome.tasks, config, root);
    crate::collect::annotate(&mut outcome.tasks, config);
    Ok(outcome.tasks.into_iter().next())
}

/// Serve `items` from the cache and live-evaluate the misses, unsorted and
/// before any whole-set annotation.
fn harvest_items(items: &[LoadItem], root: &Path, config: &Config) -> LoadOutcome {
    let overrides = package_overrides(root, config);
    let cache = TaskCache::open(root, config, &overrides);
    let mut outcome = LoadOutcome::default();
    let misses = split_hits(items, &cache, &mut outcome);
    harvest_misses(&misses, root, &overrides, &cache, &mut outcome);
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
    let (folder_tags, depth) = derivation(config, rel);
    LoadItem {
        file: root.join(rel),
        rel: rel.to_path_buf(),
        folder_tags,
        depth,
    }
}

/// The folder-derived tags and nesting depth [`item_at`] would give a file at
/// project-relative `rel`: the first configured root that contains it, with
/// the root path normalized, or neither when no root does.
pub(crate) fn derivation(config: &Config, rel: &Path) -> (Vec<String>, u8) {
    config
        .tasks
        .roots
        .iter()
        .find_map(|spec| {
            let dir = spec.dir();
            let rel_within = if dir.is_empty() {
                rel
            } else {
                rel.strip_prefix(&dir).ok()?
            };
            let deriver = RootDeriver::new(spec, &config.tasks.derive_tags);
            Some((
                deriver.tags_for(rel_within),
                crate::loader::derive::depth(rel_within),
            ))
        })
        .unwrap_or_default()
}

/// Every `.typ` file under the configured roots, paired with its
/// project-relative path and the tags its folder location derives.
fn collect_items(root: &Path, config: &Config) -> Vec<LoadItem> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();
    for spec in &config.tasks.roots {
        let dir = root.join(spec.dir());
        let deriver = RootDeriver::new(spec, &config.tasks.derive_tags);
        let mut files = Vec::new();
        collect_typ_files(&dir, &mut files);
        files.sort();
        for file in files {
            let rel_within = file.strip_prefix(&dir).unwrap_or(&file);
            let folder_tags = deriver.tags_for(rel_within);
            let depth = crate::loader::derive::depth(rel_within);
            let rel = file.strip_prefix(root).unwrap_or(&file).to_path_buf();
            // Overlapping roots name one file once, and the first root that
            // contains it wins, matching `item_at` and a commit state's list.
            if !seen.insert(rel.clone()) {
                continue;
            }
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
                &slash_path(&miss.item.rel),
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

fn build_world(
    file: &Path,
    root: &Path,
    overrides: &[(String, PathBuf)],
) -> Result<HarvestWorld, String> {
    let world = HarvestWorld::with_root(file, root).map_err(|err| err.to_string())?;
    Ok(with_packages(world, overrides))
}

/// `world` resolving `@local` packages the way every world Mindtape builds
/// does: under [`package_root`], with `overrides` (from [`package_overrides`])
/// pinning a package to an in-tree directory for the version its manifest
/// declares, and the installed or embedded copy where the pin does not hold
/// that version.
///
/// A pin is authoritative only for its own version. A task file importing
/// another version falls through to the package directory, so a commit range
/// across a prelude bump reads each tree's imports against the version that
/// tree names while the working tree keeps its pin.
#[must_use]
pub fn with_packages(world: HarvestWorld, overrides: &[(String, PathBuf)]) -> HarvestWorld {
    pins::with_packages_at(world, overrides, package_root())
}

/// [`with_packages`] with a commit state's own package files tried first, so
/// a past tree's task files resolve against that tree's `lib/` rather than the
/// working tree's pin or the machine's install.
#[must_use]
pub(crate) fn with_packages_and(
    world: HarvestWorld,
    overrides: &[(String, PathBuf)],
    state: StatePackages,
) -> HarvestWorld {
    pins::with_packages_and(world, overrides, package_root(), Some(state))
}

/// The `@local/<name>` → in-tree overrides for this project: its
/// `[[typst.local-packages]]` entries, and nothing else. The core names no
/// directory of its own, so Mindtape's own repository pins its `lib/` with an
/// entry like any other project. A pin serves only the version its
/// `typst.toml` declares; [`with_packages`] applies that rule.
///
/// Public because it is the project's single answer to "which `@local`
/// packages resolve here", and every world Mindtape builds, `mt eval`'s
/// included, has to give the same one.
pub fn package_overrides(root: &Path, config: &Config) -> Vec<(String, PathBuf)> {
    config
        .typst
        .local_packages
        .iter()
        .map(|pkg| (pkg.name.clone(), root.join(&pkg.path)))
        .collect()
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
mod tests_pins;

#[cfg(test)]
mod tests_cache;

#[cfg(test)]
mod tests_prelude;

#[cfg(test)]
mod tests_eval_line;
