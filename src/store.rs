//! SQLite-backed store for indexed tasks, properties, and bindings.
//!
//! Defines the `Store` trait (anti-corruption layer) and an `SqliteStore`
//! implementation. The store layer uses plain Rust types — no typst
//! dependencies — so future backends (DuckDB, etc.) can implement the
//! same trait without pulling in the Typst crate ecosystem.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::eval::{self, EvalResult};

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

/// A task extracted from a checklist item in a TaskFile.
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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "due" => Some(PropertyKind::Due),
            "tag" => Some(PropertyKind::Tag),
            "id" => Some(PropertyKind::Id),
            _ => None,
        }
    }
}

/// A `#let` binding exported by a TaskFile's module scope.
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
    Eval(String),
}

// ---------------------------------------------------------------------------
// Store trait
// ---------------------------------------------------------------------------

pub trait Store {
    /// Insert or update a task file record. Returns the file's row ID.
    fn upsert_task_file(&mut self, file: &TaskFile) -> Result<i64, StoreError>;

    /// Replace all tasks and their properties for a given file.
    fn upsert_tasks(
        &mut self,
        file_id: i64,
        tasks: &[TaskRecord],
        props: &[Vec<TaskProperty>],
    ) -> Result<(), StoreError>;

    /// Replace all bindings for a given file.
    fn upsert_bindings(
        &mut self,
        file_id: i64,
        bindings: &[FileBinding],
    ) -> Result<(), StoreError>;

    /// Remove a task file and all associated data (cascades).
    fn remove_task_file(&mut self, path: &Path) -> Result<(), StoreError>;

    /// Query tasks with optional filters.
    fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskView>, StoreError>;

    /// Get the stored eval_hash for a file path, if it exists.
    fn get_file_hash(&self, path: &Path) -> Result<Option<String>, StoreError>;
}

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

const SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS task_files (
    id            INTEGER PRIMARY KEY,
    relative_path TEXT    NOT NULL UNIQUE,
    title         TEXT,
    eval_hash     TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tasks (
    id            INTEGER PRIMARY KEY,
    task_file_id  INTEGER NOT NULL REFERENCES task_files(id) ON DELETE CASCADE,
    title         TEXT    NOT NULL,
    is_done       INTEGER NOT NULL DEFAULT 0,
    position      INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS task_properties (
    id            INTEGER PRIMARY KEY,
    task_id       INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    kind          TEXT    NOT NULL,
    key           TEXT    NOT NULL,
    value         TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS file_bindings (
    id            INTEGER PRIMARY KEY,
    task_file_id  INTEGER NOT NULL REFERENCES task_files(id) ON DELETE CASCADE,
    name          TEXT    NOT NULL,
    value_type    TEXT    NOT NULL,
    value_json    TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_file ON tasks(task_file_id);
CREATE INDEX IF NOT EXISTS idx_props_task ON task_properties(task_id);
CREATE INDEX IF NOT EXISTS idx_props_kind ON task_properties(kind);
CREATE INDEX IF NOT EXISTS idx_bindings_file ON file_bindings(task_file_id);
";

// ---------------------------------------------------------------------------
// SqliteStore
// ---------------------------------------------------------------------------

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Open (or create) a SQLite database at the given path.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// Open an in-memory database (for tests).
    pub fn open_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self, StoreError> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Self::migrate(&conn)?;
        Ok(Self { conn })
    }

    fn migrate(conn: &Connection) -> Result<(), StoreError> {
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|e| StoreError::Migration(e.to_string()))?;

        if version < 1 {
            conn.execute_batch(SCHEMA_V1)
                .map_err(|e| StoreError::Migration(e.to_string()))?;
            conn.pragma_update(None, "user_version", 1)
                .map_err(|e| StoreError::Migration(e.to_string()))?;
        }

        Ok(())
    }
}

impl Store for SqliteStore {
    fn upsert_task_file(&mut self, file: &TaskFile) -> Result<i64, StoreError> {
        let path_str = file.relative_path.to_string_lossy();
        self.conn.execute(
            "INSERT INTO task_files (relative_path, title, eval_hash, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))
             ON CONFLICT(relative_path) DO UPDATE SET
               title = excluded.title,
               eval_hash = excluded.eval_hash,
               updated_at = datetime('now')",
            params![path_str.as_ref(), file.title, file.eval_hash],
        )?;
        let id = self.conn.last_insert_rowid();
        // ON CONFLICT UPDATE may not change last_insert_rowid; query to be sure.
        if id == 0 {
            let id: i64 = self.conn.query_row(
                "SELECT id FROM task_files WHERE relative_path = ?1",
                params![path_str.as_ref()],
                |row| row.get(0),
            )?;
            return Ok(id);
        }
        Ok(id)
    }

    fn upsert_tasks(
        &mut self,
        file_id: i64,
        tasks: &[TaskRecord],
        props: &[Vec<TaskProperty>],
    ) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;

        // Delete existing tasks (cascade deletes properties too).
        tx.execute("DELETE FROM tasks WHERE task_file_id = ?1", params![file_id])?;

        for (i, task) in tasks.iter().enumerate() {
            tx.execute(
                "INSERT INTO tasks (task_file_id, title, is_done, position)
                 VALUES (?1, ?2, ?3, ?4)",
                params![file_id, task.title, task.is_done, task.position],
            )?;
            let task_id = tx.last_insert_rowid();

            if let Some(task_props) = props.get(i) {
                for prop in task_props {
                    tx.execute(
                        "INSERT INTO task_properties (task_id, kind, key, value)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![task_id, prop.kind.as_str(), prop.key, prop.value],
                    )?;
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn upsert_bindings(
        &mut self,
        file_id: i64,
        bindings: &[FileBinding],
    ) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "DELETE FROM file_bindings WHERE task_file_id = ?1",
            params![file_id],
        )?;

        for b in bindings {
            tx.execute(
                "INSERT INTO file_bindings (task_file_id, name, value_type, value_json)
                 VALUES (?1, ?2, ?3, ?4)",
                params![file_id, b.name, b.value_type, b.value_json],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn remove_task_file(&mut self, path: &Path) -> Result<(), StoreError> {
        let path_str = path.to_string_lossy();
        self.conn.execute(
            "DELETE FROM task_files WHERE relative_path = ?1",
            params![path_str.as_ref()],
        )?;
        Ok(())
    }

    fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskView>, StoreError> {
        let mut sql = String::from(
            "SELECT t.id, t.title, t.is_done, t.position, tf.relative_path, tf.title
             FROM tasks t
             JOIN task_files tf ON t.task_file_id = tf.id",
        );
        let mut conditions: Vec<String> = Vec::new();
        let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(done) = filter.done {
            conditions.push(format!("t.is_done = ?{}", bind_values.len() + 1));
            bind_values.push(Box::new(done as i32));
        }

        if let Some(ref tag) = filter.tag {
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM task_properties tp WHERE tp.task_id = t.id AND tp.kind = 'tag' AND tp.value = ?{})",
                bind_values.len() + 1
            ));
            bind_values.push(Box::new(tag.clone()));
        }

        if let Some(ref due_before) = filter.due_before {
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM task_properties tp WHERE tp.task_id = t.id AND tp.kind = 'due' AND tp.value <= ?{})",
                bind_values.len() + 1
            ));
            bind_values.push(Box::new(due_before.clone()));
        }

        if let Some(ref file_path) = filter.file_path {
            conditions.push(format!(
                "tf.relative_path = ?{}",
                bind_values.len() + 1
            ));
            bind_values.push(Box::new(file_path.to_string_lossy().to_string()));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY tf.relative_path, t.position");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            bind_values.iter().map(|b| b.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows: Vec<(i64, String, bool, i32, String, Option<String>)> = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get::<_, i32>(2)? != 0,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut views = Vec::with_capacity(rows.len());
        for (task_id, title, is_done, position, file_path, file_title) in rows {
            let mut due = None;
            let mut tags = Vec::new();

            let mut prop_stmt = self.conn.prepare_cached(
                "SELECT kind, value FROM task_properties WHERE task_id = ?1",
            )?;
            let prop_rows = prop_stmt.query_map(params![task_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for prop in prop_rows {
                let (kind, value) = prop?;
                match kind.as_str() {
                    "due" => due = Some(value),
                    "tag" => tags.push(value),
                    _ => {}
                }
            }

            views.push(TaskView {
                title,
                is_done,
                position,
                file_path: PathBuf::from(file_path),
                file_title,
                due,
                tags,
            });
        }

        Ok(views)
    }

    fn get_file_hash(&self, path: &Path) -> Result<Option<String>, StoreError> {
        let path_str = path.to_string_lossy();
        let hash = self
            .conn
            .query_row(
                "SELECT eval_hash FROM task_files WHERE relative_path = ?1",
                params![path_str.as_ref()],
                |row| row.get(0),
            )
            .optional()?;
        Ok(hash)
    }
}

// ---------------------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------------------

/// Compute SHA-256 hash of a file's contents, returned as hex string.
pub fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let bytes = std::fs::read(path)?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{:x}", hash))
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

    let result = eval::eval_file_full(world).map_err(|e| StoreError::Eval(e))?;

    let (task_file, tasks, props, bindings) = to_store_records(&result, relative, &hash);

    let file_id = store.upsert_task_file(&task_file)?;
    store.upsert_tasks(file_id, &tasks, &props)?;
    store.upsert_bindings(file_id, &bindings)?;

    Ok(true)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> SqliteStore {
        SqliteStore::open_memory().unwrap()
    }

    fn make_task_file(path: &str, hash: &str) -> TaskFile {
        TaskFile {
            id: None,
            relative_path: PathBuf::from(path),
            title: Some("Test".to_string()),
            eval_hash: hash.to_string(),
            updated_at: String::new(),
        }
    }

    fn make_record(title: &str, done: bool, pos: i32) -> TaskRecord {
        TaskRecord {
            id: None,
            task_file_id: 0,
            title: title.to_string(),
            is_done: done,
            position: pos,
        }
    }

    fn make_due_prop(value: &str) -> TaskProperty {
        TaskProperty {
            id: None,
            task_id: 0,
            kind: PropertyKind::Due,
            key: "due".to_string(),
            value: value.to_string(),
        }
    }

    fn make_tag_prop(value: &str) -> TaskProperty {
        TaskProperty {
            id: None,
            task_id: 0,
            kind: PropertyKind::Tag,
            key: "tag".to_string(),
            value: value.to_string(),
        }
    }

    fn make_binding(name: &str, vtype: &str, vjson: &str) -> FileBinding {
        FileBinding {
            id: None,
            task_file_id: 0,
            name: name.to_string(),
            value_type: vtype.to_string(),
            value_json: vjson.to_string(),
        }
    }

    // --- Schema / migration ---

    #[test]
    fn open_memory_creates_tables() {
        let store = test_store();
        let count: i32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('task_files','tasks','task_properties','file_bindings')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn open_twice_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let _s1 = SqliteStore::open(&db_path).unwrap();
        let _s2 = SqliteStore::open(&db_path).unwrap();
    }

    #[test]
    fn migration_sets_user_version() {
        let store = test_store();
        let version: i32 = store
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }

    // --- upsert_task_file ---

    #[test]
    fn upsert_task_file_insert() {
        let mut store = test_store();
        let file = make_task_file("notes/todo.typ", "abc123");
        let id = store.upsert_task_file(&file).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn upsert_task_file_update_on_same_path() {
        let mut store = test_store();
        let file1 = make_task_file("todo.typ", "hash1");
        let id1 = store.upsert_task_file(&file1).unwrap();

        let file2 = make_task_file("todo.typ", "hash2");
        let id2 = store.upsert_task_file(&file2).unwrap();

        assert_eq!(id1, id2);

        let hash: String = store
            .conn
            .query_row(
                "SELECT eval_hash FROM task_files WHERE id = ?1",
                params![id1],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(hash, "hash2");
    }

    // --- upsert_tasks ---

    #[test]
    fn upsert_tasks_basic() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("t.typ", "h")).unwrap();

        let tasks = vec![make_record("Task A", false, 0), make_record("Task B", true, 1)];
        let props = vec![vec![], vec![]];
        store.upsert_tasks(file_id, &tasks, &props).unwrap();

        let count: i32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM tasks WHERE task_file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn upsert_tasks_replaces_on_reindex() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("t.typ", "h")).unwrap();

        let tasks1 = vec![make_record("Old", false, 0)];
        store.upsert_tasks(file_id, &tasks1, &vec![vec![]]).unwrap();

        let tasks2 = vec![make_record("New A", false, 0), make_record("New B", false, 1)];
        store.upsert_tasks(file_id, &tasks2, &vec![vec![], vec![]]).unwrap();

        let count: i32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM tasks WHERE task_file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        let title: String = store
            .conn
            .query_row(
                "SELECT title FROM tasks WHERE task_file_id = ?1 ORDER BY position LIMIT 1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(title, "New A");
    }

    #[test]
    fn upsert_tasks_with_properties() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("t.typ", "h")).unwrap();

        let tasks = vec![make_record("Task", false, 0)];
        let props = vec![vec![make_due_prop("2026-03-01"), make_tag_prop("work")]];
        store.upsert_tasks(file_id, &tasks, &props).unwrap();

        let prop_count: i32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM task_properties tp
                 JOIN tasks t ON tp.task_id = t.id
                 WHERE t.task_file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(prop_count, 2);
    }

    // --- upsert_bindings ---

    #[test]
    fn upsert_bindings_basic() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("t.typ", "h")).unwrap();

        let bindings = vec![make_binding("note", "string", "\"hello\"")];
        store.upsert_bindings(file_id, &bindings).unwrap();

        let count: i32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM file_bindings WHERE task_file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn upsert_bindings_replaces_on_reindex() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("t.typ", "h")).unwrap();

        store
            .upsert_bindings(file_id, &vec![make_binding("old", "string", "\"x\"")])
            .unwrap();
        store
            .upsert_bindings(file_id, &vec![make_binding("new", "int", "42")])
            .unwrap();

        let name: String = store
            .conn
            .query_row(
                "SELECT name FROM file_bindings WHERE task_file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "new");
    }

    // --- remove_task_file ---

    #[test]
    fn remove_cascades_to_tasks_and_props() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("t.typ", "h")).unwrap();

        let tasks = vec![make_record("T", false, 0)];
        let props = vec![vec![make_tag_prop("x")]];
        store.upsert_tasks(file_id, &tasks, &props).unwrap();
        store
            .upsert_bindings(file_id, &vec![make_binding("n", "int", "1")])
            .unwrap();

        store.remove_task_file(Path::new("t.typ")).unwrap();

        let file_count: i32 = store
            .conn
            .query_row("SELECT count(*) FROM task_files", [], |row| row.get(0))
            .unwrap();
        let task_count: i32 = store
            .conn
            .query_row("SELECT count(*) FROM tasks", [], |row| row.get(0))
            .unwrap();
        let prop_count: i32 = store
            .conn
            .query_row("SELECT count(*) FROM task_properties", [], |row| row.get(0))
            .unwrap();
        let binding_count: i32 = store
            .conn
            .query_row("SELECT count(*) FROM file_bindings", [], |row| row.get(0))
            .unwrap();

        assert_eq!(file_count, 0);
        assert_eq!(task_count, 0);
        assert_eq!(prop_count, 0);
        assert_eq!(binding_count, 0);
    }

    #[test]
    fn remove_nonexistent_is_noop() {
        let mut store = test_store();
        store.remove_task_file(Path::new("nope.typ")).unwrap();
    }

    // --- query_tasks ---

    fn seed_store(store: &mut SqliteStore) -> i64 {
        let file_id = store
            .upsert_task_file(&make_task_file("todo.typ", "seed"))
            .unwrap();
        let tasks = vec![
            make_record("Buy milk", false, 0),
            make_record("Done thing", true, 1),
            make_record("Urgent", false, 2),
        ];
        let props = vec![
            vec![make_due_prop("2026-03-01")],
            vec![],
            vec![make_tag_prop("work"), make_due_prop("2026-01-15")],
        ];
        store.upsert_tasks(file_id, &tasks, &props).unwrap();
        file_id
    }

    #[test]
    fn query_all_tasks() {
        let mut store = test_store();
        seed_store(&mut store);
        let views = store.query_tasks(&TaskFilter::default()).unwrap();
        assert_eq!(views.len(), 3);
    }

    #[test]
    fn query_filter_by_done() {
        let mut store = test_store();
        seed_store(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                done: Some(false),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 2);
        assert!(views.iter().all(|v| !v.is_done));
    }

    #[test]
    fn query_filter_by_tag() {
        let mut store = test_store();
        seed_store(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                tag: Some("work".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Urgent");
    }

    #[test]
    fn query_filter_by_due_before() {
        let mut store = test_store();
        seed_store(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                due_before: Some("2026-02-01".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Urgent");
    }

    #[test]
    fn query_filter_by_file() {
        let mut store = test_store();
        seed_store(&mut store);
        // Add another file
        let file_id2 = store
            .upsert_task_file(&make_task_file("other.typ", "h2"))
            .unwrap();
        store
            .upsert_tasks(
                file_id2,
                &vec![make_record("Other", false, 0)],
                &vec![vec![]],
            )
            .unwrap();

        let views = store
            .query_tasks(&TaskFilter {
                file_path: Some(PathBuf::from("todo.typ")),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 3);
    }

    #[test]
    fn query_with_limit() {
        let mut store = test_store();
        seed_store(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                limit: Some(1),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
    }

    #[test]
    fn query_returns_due_and_tags() {
        let mut store = test_store();
        seed_store(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                tag: Some("work".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views[0].due, Some("2026-01-15".to_string()));
        assert_eq!(views[0].tags, vec!["work"]);
    }

    // --- get_file_hash ---

    #[test]
    fn get_hash_returns_none_for_unknown() {
        let store = test_store();
        let hash = store.get_file_hash(Path::new("nope.typ")).unwrap();
        assert_eq!(hash, None);
    }

    #[test]
    fn get_hash_returns_stored_hash() {
        let mut store = test_store();
        store
            .upsert_task_file(&make_task_file("t.typ", "abc123"))
            .unwrap();
        let hash = store.get_file_hash(Path::new("t.typ")).unwrap();
        assert_eq!(hash, Some("abc123".to_string()));
    }

    // --- hash_file ---

    #[test]
    fn hash_file_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("test.txt");
        std::fs::write(&f, "hello").unwrap();
        let h1 = hash_file(&f).unwrap();
        let h2 = hash_file(&f).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_file_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("test.txt");
        std::fs::write(&f, "hello").unwrap();
        let h1 = hash_file(&f).unwrap();
        std::fs::write(&f, "world").unwrap();
        let h2 = hash_file(&f).unwrap();
        assert_ne!(h1, h2);
    }

    // --- to_store_records ---

    #[test]
    fn to_store_records_converts_tags_and_due() {
        use typst::foundations::Datetime;

        let eval_result = EvalResult {
            tasks: vec![eval::Task {
                title: "Test".to_string(),
                done: false,
                due: Some(Datetime::from_ymd(2026, 3, 1).unwrap()),
                tags: vec!["work".to_string(), "urgent".to_string()],
                position: 0,
            }],
            title: Some("Heading".to_string()),
            bindings: vec![("note".to_string(), "string".to_string(), "\"hi\"".to_string())],
        };

        let (tf, tasks, props, bindings) =
            to_store_records(&eval_result, Path::new("test.typ"), "hash");

        assert_eq!(tf.relative_path, PathBuf::from("test.typ"));
        assert_eq!(tf.title, Some("Heading".to_string()));
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Test");
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].len(), 3); // 1 due + 2 tags
        assert_eq!(props[0][0].kind, PropertyKind::Due);
        assert_eq!(props[0][0].value, "2026-03-01");
        assert_eq!(props[0][1].kind, PropertyKind::Tag);
        assert_eq!(props[0][1].value, "work");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "note");
    }

    // --- PropertyKind ---

    #[test]
    fn property_kind_roundtrip() {
        assert_eq!(PropertyKind::from_str("due"), Some(PropertyKind::Due));
        assert_eq!(PropertyKind::from_str("tag"), Some(PropertyKind::Tag));
        assert_eq!(PropertyKind::from_str("id"), Some(PropertyKind::Id));
        assert_eq!(PropertyKind::from_str("unknown"), None);
        assert_eq!(PropertyKind::Due.as_str(), "due");
        assert_eq!(PropertyKind::Tag.as_str(), "tag");
        assert_eq!(PropertyKind::Id.as_str(), "id");
    }
}
