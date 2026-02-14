//! SQLite-backed store for indexed tasks, properties, and bindings.
//!
//! Defines the `Store` trait (anti-corruption layer) and domain types.
//! The store layer uses plain Rust types — no typst dependencies — so future
//! backends (`DuckDB`, etc.) can implement the same trait without pulling in
//! the Typst crate ecosystem.

mod indexer;
mod sqlite;

use std::path::{Path, PathBuf};

use mindtape_eval::EvalError;
use thiserror::Error;

// Re-export submodules' public items at the `store` level.
pub use indexer::{hash_file, index_file, to_store_records};
pub use sqlite::SqliteStore;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// A `.typ` file that has been evaluated and indexed.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskFile {
    pub id: Option<i64>,
    pub relative_path: PathBuf,
    pub title: Option<String>,
    pub eval_hash: String,
    pub updated_at: String,
}

/// A task extracted from a checklist item in a `TaskFile`.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskRecord {
    pub id: Option<i64>,
    pub task_file_id: i64,
    pub title: String,
    pub is_done: bool,
    pub position: i32,
}

/// A property attached to a task.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskProperty {
    pub id: Option<i64>,
    pub task_id: i64,
    pub kind: PropertyKind,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKind {
    Due,
    Tag,
    Id,
}

impl PropertyKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PropertyKind::Due => "due",
            PropertyKind::Tag => "tag",
            PropertyKind::Id => "id",
        }
    }

    pub fn try_from_str(kind_str: &str) -> Option<Self> {
        match kind_str {
            "due" => Some(PropertyKind::Due),
            "tag" => Some(PropertyKind::Tag),
            "id" => Some(PropertyKind::Id),
            _ => None,
        }
    }
}

/// A `#let` binding exported by a `TaskFile`'s module scope.
#[derive(Debug, Clone, PartialEq)]
pub struct FileBinding {
    pub id: Option<i64>,
    pub task_file_id: i64,
    pub name: String,
    pub value_type: String,
    pub value_json: String,
}

/// Filters for querying tasks.
#[derive(Debug, Default)]
pub struct TaskFilter {
    pub done: Option<bool>,
    pub tag: Option<String>,
    pub due_before: Option<String>,
    pub file_path: Option<PathBuf>,
    pub folder: Option<PathBuf>,
    pub limit: Option<usize>,
}

/// A task with its file context and properties, returned by queries.
#[derive(Debug, Clone)]
pub struct TaskView {
    pub title: String,
    pub is_done: bool,
    pub position: i32,
    pub file_path: PathBuf,
    pub file_title: Option<String>,
    pub due: Option<String>,
    pub tags: Vec<String>,
}

/// Summary of an indexed file, returned by `list_files()`.
#[derive(Debug, Clone)]
pub struct FileView {
    pub relative_path: PathBuf,
    pub title: Option<String>,
    pub task_count: i64,
    pub updated_at: String,
}

/// Aggregate index statistics, returned by `get_stats()`.
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub file_count: i64,
    pub task_count: i64,
    pub done_count: i64,
    pub pending_count: i64,
    pub last_updated: Option<String>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("migration failed: {0}")]
    Migration(String),

    #[error("path error: {0}")]
    Path(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("eval error: {0}")]
    Eval(#[from] EvalError),
}

// ---------------------------------------------------------------------------
// Store trait
// ---------------------------------------------------------------------------

pub trait Store {
    /// Insert or update a task file record. Returns the file's row ID.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn upsert_task_file(&mut self, file: &TaskFile) -> Result<i64, StoreError>;

    /// Replace all tasks and their properties for a given file.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn upsert_tasks(
        &mut self,
        file_id: i64,
        tasks: &[TaskRecord],
        props: &[Vec<TaskProperty>],
    ) -> Result<(), StoreError>;

    /// Replace all bindings for a given file.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn upsert_bindings(
        &mut self,
        file_id: i64,
        bindings: &[FileBinding],
    ) -> Result<(), StoreError>;

    /// Remove a task file and all associated data (cascades).
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn remove_task_file(&mut self, path: &Path) -> Result<(), StoreError>;

    /// Query tasks with optional filters.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskView>, StoreError>;

    /// Get the stored `eval_hash` for a file path, if it exists.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn get_file_hash(&self, path: &Path) -> Result<Option<String>, StoreError>;

    /// List all indexed files with task counts.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn list_files(&self) -> Result<Vec<FileView>, StoreError>;

    /// Get aggregate index statistics.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn get_stats(&self) -> Result<IndexStats, StoreError>;
}
