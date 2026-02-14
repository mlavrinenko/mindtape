//! Indexing orchestration: evaluate a `.typ` file, convert to store records,
//! and persist via the `Store` trait.

use std::path::Path;

use sha2::{Digest, Sha256};

use super::{
    FileBinding, PropertyKind, Store, StoreError, TaskFile, TaskProperty, TaskRecord,
};
use mindtape_eval::{self as eval, EvalResult};

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
                value: format!(
                    "{:04}-{:02}-{:02}",
                    dt.year().unwrap_or(0),
                    dt.month().unwrap_or(0),
                    dt.day().unwrap_or(0),
                ),
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

    if let Some(stored_hash) = store.get_file_hash(relative)? {
        if stored_hash == hash {
            return Ok(false);
        }
    }

    let result = eval::eval_file_full(world)?;

    let (task_file, tasks, props, bindings) = to_store_records(&result, relative, &hash);

    let file_id = store.upsert_task_file(&task_file)?;
    store.upsert_tasks(file_id, &tasks, &props)?;
    store.upsert_bindings(file_id, &bindings)?;

    Ok(true)
}
