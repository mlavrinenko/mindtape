//! SQLite-backed store for indexed tasks, properties, and bindings.
//!
//! Defines the `Store` trait (anti-corruption layer) and domain types.
//! The store layer uses plain Rust types — no typst dependencies — so future
//! backends (`DuckDB`, etc.) can implement the same trait without pulling in
//! the Typst crate ecosystem.

pub(crate) mod filter_expr;
mod indexer;
mod migrations;
mod sqlite;

use std::path::{Path, PathBuf};

use mindtape_eval::EvalError;
use serde::Serialize;
use thiserror::Error;

// Re-export submodules' public items at the `store` level.
pub use indexer::{hash_file, index_file, index_file_with_deps, to_store_records};
pub use sqlite::SqliteStore;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// A `.typ` file that has been evaluated and indexed.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskFile {
    pub id: Option<i64>,
    pub file_path: PathBuf,
    pub watch_root: Option<PathBuf>,
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
    /// Heading path (e.g. "Header 1 > Subheader 1.1").
    pub milestone: Option<String>,
    /// Due date (YYYY-MM-DD), denormalized for efficient sorting/filtering.
    pub due: Option<String>,
    /// Start date (YYYY-MM-DD), denormalized for efficient sorting/filtering.
    pub start: Option<String>,
    /// Rank (positive integer, higher = more important), denormalized.
    pub rank: Option<i64>,
    /// User-assigned task ID (`UUIDv7`), denormalized for efficient sorting/lookup.
    pub task_id: Option<String>,
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
    Start,
    Rank,
    Tag,
    Id,
}

impl PropertyKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PropertyKind::Due => "mindtape.due",
            PropertyKind::Start => "mindtape.start",
            PropertyKind::Rank => "mindtape.rank",
            PropertyKind::Tag => "mindtape.tag",
            PropertyKind::Id => "mindtape.id",
        }
    }

    pub fn try_from_str(kind_str: &str) -> Option<Self> {
        match kind_str {
            "mindtape.due" => Some(PropertyKind::Due),
            "mindtape.start" => Some(PropertyKind::Start),
            "mindtape.rank" => Some(PropertyKind::Rank),
            "mindtape.tag" => Some(PropertyKind::Tag),
            "mindtape.id" => Some(PropertyKind::Id),
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

// ---------------------------------------------------------------------------
// Sort types
// ---------------------------------------------------------------------------

/// Sortable fields for task queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Due,
    Start,
    Rank,
    Id,
    File,
    Position,
    Title,
    Status,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

/// A single sort specification (field + direction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortSpec {
    pub field: SortField,
    pub dir: SortDir,
}

/// Filters for querying tasks.
///
/// All active filters combine with AND logic. Empty `tags` means no tag filter.
#[derive(Debug, Default)]
pub struct TaskFilter {
    pub done: Option<bool>,
    pub folder: Option<PathBuf>,
    /// Filter by watch root (exact match on `task_files.watch_root`).
    pub watch_root: Option<PathBuf>,
    pub limit: Option<usize>,
    /// Sort order. Empty means default (`file_path, position`).
    pub sort: Vec<SortSpec>,
    /// Free-form filter expression (`evalexpr` syntax, translated to SQL).
    pub expr: Option<String>,
}

/// A task with its file context and properties, returned by queries.
#[derive(Debug, Clone, Serialize)]
pub struct TaskView {
    pub title: String,
    pub is_done: bool,
    pub position: i32,
    pub file_path: PathBuf,
    pub file_title: Option<String>,
    pub due: Option<String>,
    pub start: Option<String>,
    pub rank: Option<i64>,
    /// User-assigned task ID (`UUIDv7`).
    pub task_id: Option<String>,
    pub tags: Vec<String>,
    /// Heading path (e.g. "Header 1 > Subheader 1.1").
    pub milestone: Option<String>,
    /// Canonical watched folder path (for display context).
    pub watch_root: Option<PathBuf>,
}

/// Summary of an indexed file, returned by `list_files()`.
#[derive(Debug, Clone, Serialize)]
pub struct FileView {
    pub file_path: PathBuf,
    pub watch_root: Option<PathBuf>,
    pub title: Option<String>,
    pub task_count: i64,
    pub updated_at: String,
}

/// Aggregate index statistics, returned by `get_stats()`.
#[derive(Debug, Clone, Serialize)]
pub struct IndexStats {
    pub file_count: i64,
    pub task_count: i64,
    pub done_count: i64,
    pub pending_count: i64,
    pub last_updated: Option<String>,
}

/// A cross-file reference: one file importing/depending on another.
#[derive(Debug, Clone, PartialEq)]
pub struct FileReference {
    pub id: Option<i64>,
    pub source_file_id: i64,
    pub target_path: PathBuf,
}

/// A file that failed to index, with the error message.
#[derive(Debug, Clone, Serialize)]
pub struct FileError {
    pub file_path: PathBuf,
    pub watch_root: Option<PathBuf>,
    pub error: String,
    pub updated_at: String,
}

/// A file with its incoming and outgoing dependencies.
#[derive(Debug, Clone, Serialize)]
pub struct FileDependencies {
    pub file_path: PathBuf,
    pub file_title: Option<String>,
    /// Files that this file imports (dependencies).
    pub imports: Vec<PathBuf>,
    /// Files that import this file (dependents).
    pub imported_by: Vec<PathBuf>,
}

/// A task with its file context, returned by `find_task_by_id()`.
#[derive(Debug, Clone)]
pub struct TaskWithFile {
    pub task_id: String,
    pub task_title: String,
    pub is_done: bool,
    pub file_path: PathBuf,
    pub file_hash: String,
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

    #[error("filter expression error: {0}")]
    FilterExpr(#[from] filter_expr::FilterExprError),
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
    fn upsert_bindings(&mut self, file_id: i64, bindings: &[FileBinding])
    -> Result<(), StoreError>;

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

    /// Replace all file references for a given source file.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn upsert_file_references(
        &mut self,
        source_file_id: i64,
        target_paths: &[PathBuf],
    ) -> Result<(), StoreError>;

    /// Get dependency information for a specific file.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn get_file_dependencies(&self, path: &Path) -> Result<Option<FileDependencies>, StoreError>;

    /// List all files with their dependency counts.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn list_file_dependencies(&self) -> Result<Vec<FileDependencies>, StoreError>;

    /// Find a task by exact ID or masked ID pattern (e.g., "*37f8" matches IDs ending with "37f8").
    /// Returns error if masked pattern matches 0 or 2+ tasks (ambiguous).
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails or the pattern is ambiguous.
    fn find_task_by_id(&self, id_or_mask: &str) -> Result<TaskWithFile, StoreError>;

    /// Record an indexing error for a file (insert or update).
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn upsert_file_error(
        &mut self,
        path: &Path,
        watch_root: Option<&Path>,
        error: &str,
    ) -> Result<(), StoreError>;

    /// Remove a previously recorded file error.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn remove_file_error(&mut self, path: &Path) -> Result<(), StoreError>;

    /// List all recorded file errors.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn list_file_errors(&self) -> Result<Vec<FileError>, StoreError>;
}
