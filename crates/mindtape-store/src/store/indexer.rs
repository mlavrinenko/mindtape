//! Indexing orchestration: evaluate a `.typ` file, convert to store records,
//! and persist via the `Store` trait.

use std::path::Path;

use log::{debug, trace};
use sha2::{Digest, Sha256};

use super::{
    FileBinding, PropertyKind, Store, StoreError, TaskFile, TaskProperty, TaskRecord,
};
use mindtape_eval::{self as eval, format_date, EvalResult};

// ---------------------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------------------

/// Compute SHA-256 hash of a file's contents, returned as hex string.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let bytes = std::fs::read(path)?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{hash:x}"))
}

// ---------------------------------------------------------------------------
// Conversion: eval types -> store types
// ---------------------------------------------------------------------------

/// Convert an `EvalResult` into store domain types.
pub fn to_store_records(
    result: &EvalResult,
    relative_path: &Path,
    content_hash: &str,
) -> (TaskFile, Vec<TaskRecord>, Vec<Vec<TaskProperty>>, Vec<FileBinding>) {
    let task_file = TaskFile {
        id: None,
        relative_path: relative_path.to_path_buf(),
        title: result.title.clone(),
        eval_hash: content_hash.to_string(),
        updated_at: String::new(), // filled by SQLite default
    };

    let mut task_records = Vec::new();
    let mut all_props = Vec::new();

    for task in &result.tasks {
        task_records.push(TaskRecord {
            id: None,
            task_file_id: 0, // filled during insert
            title: task.title.clone(),
            is_done: task.done,
            position: task.position as i32,
        });

        let mut props = Vec::new();
        if let Some(dt) = &task.due {
            props.push(TaskProperty {
                id: None,
                task_id: 0, // filled during insert
                kind: PropertyKind::Due,
                key: "due".to_string(),
                value: format_date(dt),
            });
        }
        for tag in &task.tags {
            props.push(TaskProperty {
                id: None,
                task_id: 0,
                kind: PropertyKind::Tag,
                key: "tag".to_string(),
                value: tag.clone(),
            });
        }
        all_props.push(props);
    }

    let bindings: Vec<FileBinding> = result
        .bindings
        .iter()
        .map(|(name, vtype, vjson)| FileBinding {
            id: None,
            task_file_id: 0,
            name: name.clone(),
            value_type: vtype.clone(),
            value_json: vjson.clone(),
        })
        .collect();

    (task_file, task_records, all_props, bindings)
}

// ---------------------------------------------------------------------------
// Indexing orchestration
// ---------------------------------------------------------------------------

/// Index a single `.typ` file: evaluate, extract, and persist to the store.
///
/// Skips re-indexing if the file's content hash hasn't changed.
/// Returns `Ok(true)` if the file was indexed, `Ok(false)` if skipped.
///
/// This version works with generic `World` trait and does NOT track cross-file
/// dependencies. Use `index_file_with_deps` for dependency tracking.
///
/// # Errors
///
/// Returns `StoreError` if evaluation, hashing, or database operations fail.
pub fn index_file(
    store: &mut dyn Store,
    world: &dyn typst::World,
    file_path: &Path,
    project_root: &Path,
) -> Result<bool, StoreError> {
    let relative = file_path
        .strip_prefix(project_root)
        .map_err(|e| StoreError::Path(e.to_string()))?;

    let hash = hash_file(file_path).map_err(|e| StoreError::Io(e.to_string()))?;

    if let Some(stored_hash) = store.get_file_hash(relative)?
        && stored_hash == hash
    {
        trace!("hash unchanged, skipping {}", relative.display());
        return Ok(false);
    }

    debug!("indexing {}", relative.display());
    let result = eval::eval_file_full(world)?;

    let (task_file, tasks, props, bindings) = to_store_records(&result, relative, &hash);

    let file_id = store.upsert_task_file(&task_file)?;
    store.upsert_tasks(file_id, &tasks, &props)?;
    store.upsert_bindings(file_id, &bindings)?;

    Ok(true)
}

/// Index a single `.typ` file with dependency tracking.
///
/// This version works with `MindTapeWorld` and tracks cross-file references.
/// Skips re-indexing if the file's content hash hasn't changed.
/// Returns `Ok(true)` if the file was indexed, `Ok(false)` if skipped.
///
/// # Errors
///
/// Returns `StoreError` if evaluation, hashing, or database operations fail.
pub fn index_file_with_deps(
    store: &mut dyn Store,
    world: &eval::world::MindTapeWorld,
    file_path: &Path,
    project_root: &Path,
) -> Result<bool, StoreError> {
    let relative = file_path
        .strip_prefix(project_root)
        .map_err(|e| StoreError::Path(e.to_string()))?;

    let hash = hash_file(file_path).map_err(|e| StoreError::Io(e.to_string()))?;

    if let Some(stored_hash) = store.get_file_hash(relative)?
        && stored_hash == hash
    {
        trace!("hash unchanged, skipping {}", relative.display());
        return Ok(false);
    }

    debug!("indexing (with deps) {}", relative.display());
    let result = eval::eval_file_full_with_deps(world)?;

    let (task_file, tasks, props, bindings) = to_store_records(&result, relative, &hash);

    let file_id = store.upsert_task_file(&task_file)?;
    store.upsert_tasks(file_id, &tasks, &props)?;
    store.upsert_bindings(file_id, &bindings)?;
    store.upsert_file_references(file_id, &result.dependencies)?;

    Ok(true)
}
