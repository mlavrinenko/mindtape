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
    /// Heading path (e.g. "Header 1 > Subheader 1.1").
    pub milestone: Option<String>,
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
#[derive(Debug, Clone, Serialize)]
pub struct TaskView {
    pub title: String,
    pub is_done: bool,
    pub position: i32,
    pub file_path: PathBuf,
    pub file_title: Option<String>,
    pub due: Option<String>,
    pub tags: Vec<String>,
    /// Heading path (e.g. "Header 1 > Subheader 1.1").
    pub milestone: Option<String>,
}

/// Summary of an indexed file, returned by `list_files()`.
#[derive(Debug, Clone, Serialize)]
pub struct FileView {
    pub relative_path: PathBuf,
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

/// A matched binding from search.
#[derive(Debug, Clone, Serialize)]
pub struct BindingView {
    pub name: String,
    pub value: String,
    pub file_path: PathBuf,
    pub file_title: Option<String>,
}

/// Combined full-text search results across tasks and bindings.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResults {
    pub tasks: Vec<TaskView>,
    pub bindings: Vec<BindingView>,
}

/// Agenda view showing tasks grouped by time horizon.
#[derive(Debug, Clone, Serialize)]
pub struct AgendaView {
    pub overdue: Vec<TaskView>,
    pub today: Vec<TaskView>,
    pub this_week: Vec<TaskView>,
}

/// A cross-file reference: one file importing/depending on another.
#[derive(Debug, Clone, PartialEq)]
pub struct FileReference {
    pub id: Option<i64>,
    pub source_file_id: i64,
    pub target_path: PathBuf,
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

    /// Full-text search across task titles and binding names/values.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn search(&self, query: &str, limit: Option<usize>) -> Result<SearchResults, StoreError>;

    /// Query agenda view: overdue, today, and this week tasks.
    /// `today` should be an ISO date string (YYYY-MM-DD).
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database operation fails.
    fn query_agenda(&self, today: &str) -> Result<AgendaView, StoreError>;

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
}
