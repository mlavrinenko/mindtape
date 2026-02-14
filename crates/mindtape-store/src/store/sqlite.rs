//! `SQLite` implementation of the `Store` trait.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use super::{
    BindingView, FileBinding, FileView, IndexStats, SearchResults, Store, StoreError, TaskFile,
    TaskFilter, TaskProperty, TaskRecord, TaskView,
};

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

const SCHEMA_V2: &str = "
CREATE INDEX IF NOT EXISTS idx_tasks_title ON tasks(title COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_bindings_name ON file_bindings(name COLLATE NOCASE);
";

// ---------------------------------------------------------------------------
// SqliteStore
// ---------------------------------------------------------------------------

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Open (or create) a `SQLite` database at the given path.
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the database cannot be opened or migrated.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// Open an in-memory database (for tests).
    ///
    /// # Errors
    ///
    /// Returns `StoreError` if the in-memory database cannot be initialized.
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

        if version < 2 {
            conn.execute_batch(SCHEMA_V2)
                .map_err(|e| StoreError::Migration(e.to_string()))?;
            conn.pragma_update(None, "user_version", 2)
                .map_err(|e| StoreError::Migration(e.to_string()))?;
        }

        Ok(())
    }

    fn build_task_query(
        filter: &TaskFilter,
    ) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
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

        if let Some(ref folder) = filter.folder {
            let prefix = folder.to_string_lossy().to_string();
            conditions.push(format!(
                "tf.relative_path LIKE ?{} || '%'",
                bind_values.len() + 1
            ));
            bind_values.push(Box::new(prefix));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY tf.relative_path, t.position");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        (sql, bind_values)
    }

    fn fetch_task_properties(
        &self,
        task_id: i64,
    ) -> Result<(Option<String>, Vec<String>), StoreError> {
        let mut due = None;
        let mut tags = Vec::new();
        let mut stmt = self
            .conn
            .prepare_cached("SELECT kind, value FROM task_properties WHERE task_id = ?1")?;
        let rows = stmt.query_map(params![task_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for prop in rows {
            let (kind, value) = prop?;
            match kind.as_str() {
                "due" => due = Some(value),
                "tag" => tags.push(value),
                _ => {}
            }
        }
        Ok((due, tags))
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
        let (sql, bind_values) = Self::build_task_query(filter);

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            bind_values.iter().map(Box::as_ref).collect();

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
            let (due, tags) = self.fetch_task_properties(task_id)?;

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

    fn list_files(&self) -> Result<Vec<FileView>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT tf.relative_path, tf.title, tf.updated_at, COUNT(t.id)
             FROM task_files tf
             LEFT JOIN tasks t ON t.task_file_id = tf.id
             GROUP BY tf.id
             ORDER BY tf.relative_path",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FileView {
                relative_path: PathBuf::from(row.get::<_, String>(0)?),
                title: row.get(1)?,
                updated_at: row.get(2)?,
                task_count: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(StoreError::from)
    }

    fn get_stats(&self) -> Result<IndexStats, StoreError> {
        let (file_count, task_count, done_count, last_updated): (
            i64,
            i64,
            i64,
            Option<String>,
        ) = self.conn.query_row(
            "SELECT
                 COUNT(DISTINCT tf.id),
                 COUNT(t.id),
                 SUM(CASE WHEN t.is_done THEN 1 ELSE 0 END),
                 MAX(tf.updated_at)
             FROM task_files tf
             LEFT JOIN tasks t ON t.task_file_id = tf.id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, Option<i64>>(2)?.unwrap_or(0), row.get(3)?)),
        )?;
        Ok(IndexStats {
            file_count,
            task_count,
            done_count,
            pending_count: task_count - done_count,
            last_updated,
        })
    }

    fn search(&self, query: &str, limit: Option<usize>) -> Result<SearchResults, StoreError> {
        let limit_val = limit.unwrap_or(100) as i64;
        let pattern = format!("%{query}%");

        // Search task titles
        let mut task_stmt = self.conn.prepare(
            "SELECT t.id, t.title, t.is_done, t.position, tf.relative_path, tf.title
             FROM tasks t
             JOIN task_files tf ON t.task_file_id = tf.id
             WHERE t.title LIKE ?1 COLLATE NOCASE
             ORDER BY tf.relative_path, t.position
             LIMIT ?2",
        )?;
        let task_rows: Vec<(i64, String, bool, i32, String, Option<String>)> = task_stmt
            .query_map(params![pattern, limit_val], |row| {
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

        let mut tasks = Vec::with_capacity(task_rows.len());
        for (task_id, title, is_done, position, file_path, file_title) in task_rows {
            let (due, tags) = self.fetch_task_properties(task_id)?;
            tasks.push(TaskView {
                title,
                is_done,
                position,
                file_path: PathBuf::from(file_path),
                file_title,
                due,
                tags,
            });
        }

        // Search bindings (name or value)
        let mut binding_stmt = self.conn.prepare(
            "SELECT fb.name, fb.value_json, tf.relative_path, tf.title
             FROM file_bindings fb
             JOIN task_files tf ON fb.task_file_id = tf.id
             WHERE fb.name LIKE ?1 COLLATE NOCASE
                OR fb.value_json LIKE ?1 COLLATE NOCASE
             ORDER BY tf.relative_path, fb.name
             LIMIT ?2",
        )?;
        let bindings: Vec<BindingView> = binding_stmt
            .query_map(params![pattern, limit_val], |row| {
                Ok(BindingView {
                    name: row.get(0)?,
                    value: row.get(1)?,
                    file_path: PathBuf::from(row.get::<_, String>(2)?),
                    file_title: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SearchResults { tasks, bindings })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use super::super::{FileBinding, PropertyKind, TaskFile, TaskFilter, TaskProperty, TaskRecord};

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
        assert_eq!(version, 2);
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
        store.upsert_tasks(file_id, &tasks1, &[vec![]]).unwrap();

        let tasks2 = vec![make_record("New A", false, 0), make_record("New B", false, 1)];
        store.upsert_tasks(file_id, &tasks2, &[vec![], vec![]]).unwrap();

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
            .upsert_bindings(file_id, &[make_binding("old", "string", "\"x\"")])
            .unwrap();
        store
            .upsert_bindings(file_id, &[make_binding("new", "int", "42")])
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
            .upsert_bindings(file_id, &[make_binding("n", "int", "1")])
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
                &[make_record("Other", false, 0)],
                &[vec![]],
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
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "hello").unwrap();
        let h1 = super::super::hash_file(&file).unwrap();
        let h2 = super::super::hash_file(&file).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_file_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "hello").unwrap();
        let h1 = super::super::hash_file(&file).unwrap();
        std::fs::write(&file, "world").unwrap();
        let h2 = super::super::hash_file(&file).unwrap();
        assert_ne!(h1, h2);
    }

    // --- to_store_records ---

    #[test]
    fn to_store_records_converts_tags_and_due() {
        use mindtape_eval::{EvalResult, Task};
        use typst::foundations::Datetime;

        let eval_result = EvalResult {
            tasks: vec![Task {
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
            super::super::to_store_records(&eval_result, Path::new("test.typ"), "hash");

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
        assert_eq!(PropertyKind::try_from_str("due"), Some(PropertyKind::Due));
        assert_eq!(PropertyKind::try_from_str("tag"), Some(PropertyKind::Tag));
        assert_eq!(PropertyKind::try_from_str("id"), Some(PropertyKind::Id));
        assert_eq!(PropertyKind::try_from_str("unknown"), None);
        assert_eq!(PropertyKind::Due.as_str(), "due");
        assert_eq!(PropertyKind::Tag.as_str(), "tag");
        assert_eq!(PropertyKind::Id.as_str(), "id");
    }

    // --- query_tasks: folder filter ---

    #[test]
    fn query_filter_by_folder() {
        let mut store = test_store();
        let f1 = store
            .upsert_task_file(&make_task_file("notes/todo.typ", "h1"))
            .unwrap();
        store
            .upsert_tasks(f1, &[make_record("A", false, 0)], &[vec![]])
            .unwrap();

        let f2 = store
            .upsert_task_file(&make_task_file("notes/work.typ", "h2"))
            .unwrap();
        store
            .upsert_tasks(f2, &[make_record("B", false, 0)], &[vec![]])
            .unwrap();

        let f3 = store
            .upsert_task_file(&make_task_file("other/misc.typ", "h3"))
            .unwrap();
        store
            .upsert_tasks(f3, &[make_record("C", false, 0)], &[vec![]])
            .unwrap();

        let views = store
            .query_tasks(&TaskFilter {
                folder: Some(PathBuf::from("notes/")),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 2);
        assert!(views.iter().all(|v| v.file_path.starts_with("notes/")));
    }

    #[test]
    fn query_folder_no_match() {
        let mut store = test_store();
        seed_store(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                folder: Some(PathBuf::from("nonexistent/")),
                ..Default::default()
            })
            .unwrap();
        assert!(views.is_empty());
    }

    // --- list_files ---

    #[test]
    fn list_files_empty() {
        let store = test_store();
        let files = store.list_files().unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn list_files_returns_all_with_counts() {
        let mut store = test_store();
        let f1 = store
            .upsert_task_file(&make_task_file("a.typ", "h1"))
            .unwrap();
        store
            .upsert_tasks(
                f1,
                &[make_record("T1", false, 0), make_record("T2", true, 1)],
                &[vec![], vec![]],
            )
            .unwrap();

        let f2 = store
            .upsert_task_file(&make_task_file("b.typ", "h2"))
            .unwrap();
        store
            .upsert_tasks(f2, &[make_record("T3", false, 0)], &[vec![]])
            .unwrap();

        let files = store.list_files().unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].relative_path, PathBuf::from("a.typ"));
        assert_eq!(files[0].task_count, 2);
        assert_eq!(files[0].title, Some("Test".to_string()));
        assert_eq!(files[1].relative_path, PathBuf::from("b.typ"));
        assert_eq!(files[1].task_count, 1);
    }

    #[test]
    fn list_files_file_with_no_tasks() {
        let mut store = test_store();
        store
            .upsert_task_file(&make_task_file("empty.typ", "h"))
            .unwrap();
        let files = store.list_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].task_count, 0);
    }

    // --- get_stats ---

    #[test]
    fn get_stats_empty() {
        let store = test_store();
        let stats = store.get_stats().unwrap();
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.task_count, 0);
        assert_eq!(stats.done_count, 0);
        assert_eq!(stats.pending_count, 0);
        assert!(stats.last_updated.is_none());
    }

    #[test]
    fn get_stats_with_data() {
        let mut store = test_store();
        seed_store(&mut store); // 3 tasks: 2 pending, 1 done
        let stats = store.get_stats().unwrap();
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.task_count, 3);
        assert_eq!(stats.done_count, 1);
        assert_eq!(stats.pending_count, 2);
        assert!(stats.last_updated.is_some());
    }

    #[test]
    fn get_stats_multiple_files() {
        let mut store = test_store();
        seed_store(&mut store);
        let f2 = store
            .upsert_task_file(&make_task_file("other.typ", "h2"))
            .unwrap();
        store
            .upsert_tasks(f2, &[make_record("Extra", true, 0)], &[vec![]])
            .unwrap();

        let stats = store.get_stats().unwrap();
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.task_count, 4);
        assert_eq!(stats.done_count, 2);
        assert_eq!(stats.pending_count, 2);
    }

    // --- search ---

    #[test]
    fn search_by_task_title() {
        let mut store = test_store();
        seed_store(&mut store); // "Buy milk", "Done thing", "Urgent"
        let results = store.search("milk", None).unwrap();
        assert_eq!(results.tasks.len(), 1);
        assert_eq!(results.tasks[0].title, "Buy milk");
        assert!(results.bindings.is_empty());
    }

    #[test]
    fn search_case_insensitive() {
        let mut store = test_store();
        seed_store(&mut store);
        let results = store.search("URGENT", None).unwrap();
        assert_eq!(results.tasks.len(), 1);
        assert_eq!(results.tasks[0].title, "Urgent");
    }

    #[test]
    fn search_by_binding_name() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("t.typ", "h")).unwrap();
        store.upsert_bindings(file_id, &[make_binding("author", "string", "\"Alice\"")]).unwrap();

        let results = store.search("author", None).unwrap();
        assert!(results.tasks.is_empty());
        assert_eq!(results.bindings.len(), 1);
        assert_eq!(results.bindings[0].name, "author");
    }

    #[test]
    fn search_by_binding_value() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("t.typ", "h")).unwrap();
        store.upsert_bindings(file_id, &[make_binding("author", "string", "\"Alice\"")]).unwrap();

        let results = store.search("Alice", None).unwrap();
        assert_eq!(results.bindings.len(), 1);
        assert_eq!(results.bindings[0].value, "\"Alice\"");
    }

    #[test]
    fn search_no_results() {
        let mut store = test_store();
        seed_store(&mut store);
        let results = store.search("nonexistent", None).unwrap();
        assert!(results.tasks.is_empty());
        assert!(results.bindings.is_empty());
    }

    #[test]
    fn search_with_limit() {
        let mut store = test_store();
        seed_store(&mut store); // 3 tasks
        let results = store.search("", Some(2)).unwrap();
        assert_eq!(results.tasks.len(), 2);
    }
}
