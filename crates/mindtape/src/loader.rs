//! Discover the project root and load tasks by live-evaluating their files.

use std::path::{Path, PathBuf};

use typst_harvest::{HVal, HarvestWorld, harvest};

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

/// Load every task under the configured roots, live-evaluating each file.
#[must_use]
pub fn load_project(root: &Path, config: &Config) -> LoadOutcome {
    let lib_override = lib_dir(root);
    let mut outcome = LoadOutcome::default();
    for spec in &config.tasks.roots {
        let dir = root.join(spec.path());
        let deriver = RootDeriver::new(spec, &config.tasks.derive_tags);
        let mut files = Vec::new();
        collect_typ_files(&dir, &mut files);
        files.sort();
        for file in files {
            let rel_within = file.strip_prefix(&dir).unwrap_or(&file);
            let folder_tags = deriver.tags_for(rel_within);
            load_file(
                root,
                &file,
                lib_override.as_deref(),
                folder_tags,
                &mut outcome,
            );
        }
    }
    outcome.tasks.sort_by(|left, right| left.id.cmp(&right.id));
    outcome
}

fn load_file(
    root: &Path,
    file: &Path,
    lib_override: Option<&Path>,
    derived: Vec<String>,
    outcome: &mut LoadOutcome,
) {
    let rel = file.strip_prefix(root).unwrap_or(file).to_path_buf();
    let world = match build_world(file, root, lib_override) {
        Ok(world) => world,
        Err(message) => {
            outcome.errors.push(LoadError { file: rel, message });
            return;
        }
    };
    match harvest(&world) {
        Ok(result) => {
            if let Some(marker) = result.with_marker("mindtape.task").next() {
                let id = file_stem(file);
                let mut task = Task::from_marker(id, rel, &marker.value);
                task.derived_tags = derived;
                outcome.tasks.push(task);
            }
        }
        Err(err) => outcome.errors.push(LoadError {
            file: rel,
            message: err.to_string(),
        }),
    }
}

fn build_world(
    file: &Path,
    root: &Path,
    lib_override: Option<&Path>,
) -> Result<HarvestWorld, String> {
    let world = HarvestWorld::with_root(file, root).map_err(|err| err.to_string())?;
    Ok(match lib_override {
        Some(lib) => world.with_local_package("mindtape", lib.to_path_buf()),
        None => world,
    })
}

fn lib_dir(root: &Path) -> Option<PathBuf> {
    let lib = root.join("lib");
    lib.join("typst.toml").is_file().then_some(lib)
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
#[path = "loader_tests.rs"]
mod tests;
