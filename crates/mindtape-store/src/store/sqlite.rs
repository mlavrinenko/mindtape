//! `SQLite` implementation of the `Store` trait.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use super::{
    FileBinding, FileView, IndexStats, SortDir, SortField, Store, StoreError, TaskFile,
    TaskFilter, TaskProperty, TaskRecord, TaskView,
};

// ---------------------------------------------------------------------------
// SqliteStore
// ---------------------------------------------------------------------------

/// Row tuple from a task query JOIN.
type TaskRow = (i64, String, bool, i32, Option<String>, Option<String>, Option<String>, Option<i64>, Option<String>, String, Option<String>, Option<String>);

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
        super::migrations::apply_migrations(&conn)?;
        Ok(Self { conn })
    }

    /// Build a dynamic task query with filters. Returns SQL string and parameter values.
    ///
    /// Uses unnamed `?` placeholders for cleaner code - rusqlite binds them positionally.
    fn build_task_query(filter: &TaskFilter) -> (String, Vec<String>) {
        let mut sql = String::from(
            "SELECT t.id, t.title, t.is_done, t.position, t.milestone, \
             t.due, t.start, t.rank, t.task_id, tf.file_path, tf.title, tf.watch_root
             FROM tasks t
             JOIN task_files tf ON t.task_file_id = tf.id",
        );

        if filter.search.is_some() {
            sql.push_str(" JOIN tasks_fts fts ON fts.rowid = t.id");
        }

        let (conditions, params) = Self::build_filter_conditions(filter);

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY ");
        if filter.sort.is_empty() {
            sql.push_str("tf.file_path, t.position");
        } else {
            let clauses: Vec<String> = filter.sort.iter().copied().map(Self::sort_spec_to_sql).collect();
            sql.push_str(&clauses.join(", "));
        }

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        (sql, params)
    }

    /// Convert a `SortSpec` to a SQL ORDER BY clause fragment.
    fn sort_spec_to_sql(spec: super::SortSpec) -> String {
        let col = match spec.field {
            SortField::Due => "t.due",
            SortField::Start => "t.start",
            SortField::Rank => "t.rank",
            SortField::Id => "t.task_id",
            SortField::File => "tf.file_path",
            SortField::Position => "t.position",
            SortField::Title => "t.title",
            SortField::Status => "t.is_done",
        };
        let dir = match spec.dir {
            SortDir::Asc => "ASC",
            SortDir::Desc => "DESC",
        };
        // NULLS LAST so tasks without due/start/rank/id sort to the end regardless of direction.
        let nulls = match spec.field {
            SortField::Due | SortField::Start | SortField::Rank | SortField::Id => " NULLS LAST",
            _ => "",
        };
        format!("{col} {dir}{nulls}")
    }

    /// Build WHERE conditions and parameters from a `TaskFilter`.
    fn build_filter_conditions(filter: &TaskFilter) -> (Vec<String>, Vec<String>) {
        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(done) = filter.done {
            conditions.push("t.is_done = ?".to_string());
            params.push((done as i32).to_string());
        }

        for tag in &filter.tags {
            conditions.push(
                "EXISTS (SELECT 1 FROM task_properties tp \
                 WHERE tp.task_id = t.id AND tp.kind = 'mindtape.tag' AND tp.value = ?)"
                    .to_string(),
            );
            params.push(tag.clone());
        }

        if let Some(ref due_before) = filter.due_before {
            conditions.push("t.due IS NOT NULL AND t.due <= ?".to_string());
            params.push(due_before.clone());
        }

        if let Some(ref due_after) = filter.due_after {
            conditions.push("t.due IS NOT NULL AND t.due >= ?".to_string());
            params.push(due_after.clone());
        }

        if let Some(ref start_before) = filter.start_before {
            conditions.push("t.start IS NOT NULL AND t.start <= ?".to_string());
            params.push(start_before.clone());
        }

        if let Some(ref start_after) = filter.start_after {
            conditions.push("t.start IS NOT NULL AND t.start >= ?".to_string());
            params.push(start_after.clone());
        }

        if let Some(rank_min) = filter.rank_min {
            conditions.push("t.rank IS NOT NULL AND t.rank >= ?".to_string());
            params.push(rank_min.to_string());
        }

        if let Some(rank_max) = filter.rank_max {
            conditions.push("t.rank IS NOT NULL AND t.rank <= ?".to_string());
            params.push(rank_max.to_string());
        }

        if let Some(ref milestone) = filter.milestone {
            conditions.push("t.milestone LIKE '%' || ? || '%' COLLATE NOCASE".to_string());
            params.push(milestone.clone());
        }

        if let Some(ref title) = filter.title_contains {
            conditions.push("t.title LIKE '%' || ? || '%' COLLATE NOCASE".to_string());
            params.push(title.clone());
        }

        if let Some(ref query) = filter.search {
            conditions.push("tasks_fts MATCH ?".to_string());
            params.push(query.clone());
        }

        if let Some(ref file_path) = filter.file_path {
            conditions.push("tf.file_path = ?".to_string());
            params.push(file_path.to_string_lossy().to_string());
        }

        if let Some(ref folder) = filter.folder {
            conditions.push("tf.file_path LIKE ? || '%'".to_string());
            params.push(folder.to_string_lossy().to_string());
        }

        if let Some(ref root) = filter.watch_root {
            conditions.push("tf.watch_root = ?".to_string());
            params.push(root.to_string_lossy().to_string());
        }

        Self::push_with_conditions(&filter.with, &mut conditions);
        Self::push_without_conditions(&filter.without, &mut conditions);

        (conditions, params)
    }

    /// Append presence-filter (`--with`) conditions.
    fn push_with_conditions(with: &[String], conditions: &mut Vec<String>) {
        for prop in with {
            match prop.as_str() {
                "due" => conditions.push("t.due IS NOT NULL".to_string()),
                "start" => conditions.push("t.start IS NOT NULL".to_string()),
                "rank" => conditions.push("t.rank IS NOT NULL".to_string()),
                "id" => conditions.push("t.task_id IS NOT NULL".to_string()),
                "tag" => conditions.push(
                    "EXISTS (SELECT 1 FROM task_properties tp \
                     WHERE tp.task_id = t.id AND tp.kind = 'mindtape.tag')"
                        .to_string(),
                ),
                _ => {} // Validation happens in the CLI layer
            }
        }
    }

    /// Append absence-filter (`--without`) conditions.
    fn push_without_conditions(without: &[String], conditions: &mut Vec<String>) {
        for prop in without {
            match prop.as_str() {
                "due" => conditions.push("t.due IS NULL".to_string()),
                "start" => conditions.push("t.start IS NULL".to_string()),
                "rank" => conditions.push("t.rank IS NULL".to_string()),
                "id" => conditions.push("t.task_id IS NULL".to_string()),
                "tag" => conditions.push(
                    "NOT EXISTS (SELECT 1 FROM task_properties tp \
                     WHERE tp.task_id = t.id AND tp.kind = 'mindtape.tag')"
                        .to_string(),
                ),
                _ => {} // Validation happens in the CLI layer
            }
        }
    }

    /// Fetch task views from a prepared statement.
    ///
    /// `due` and `task_id` come from denormalized columns on `tasks`.
    /// Tags are batch-fetched from `task_properties` in a single query.
    fn fetch_task_views(
        &self,
        stmt: &mut rusqlite::Statement,
        params: &[&dyn rusqlite::ToSql],
    ) -> Result<Vec<TaskView>, StoreError> {
        let task_rows: Vec<TaskRow> = stmt
            .query_map(params, |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get::<_, i32>(2)? != 0,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if task_rows.is_empty() {
            return Ok(Vec::new());
        }

        // Batch-fetch tags from task_properties.
        let row_ids: Vec<i64> = task_rows.iter().map(|(id, ..)| *id).collect();
        let placeholders = row_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let tags_sql = format!(
            "SELECT task_id, value FROM task_properties \
             WHERE task_id IN ({placeholders}) AND kind = 'mindtape.tag' ORDER BY task_id"
        );
        let mut tags_stmt = self.conn.prepare(&tags_sql)?;
        let id_refs: Vec<&dyn rusqlite::types::ToSql> =
            row_ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();

        use std::collections::HashMap;
        let mut tags_map: HashMap<i64, Vec<String>> = HashMap::new();
        let tag_rows = tags_stmt.query_map(id_refs.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in tag_rows {
            let (task_id, value) = row?;
            tags_map.entry(task_id).or_default().push(value);
        }

        let mut tasks = Vec::with_capacity(task_rows.len());
        for (row_id, title, is_done, position, milestone, due, start, rank, task_id, file_path, file_title, watch_root) in task_rows {
            let tags = tags_map.remove(&row_id).unwrap_or_default();
            tasks.push(TaskView {
                title,
                is_done,
                position,
                file_path: PathBuf::from(file_path),
                file_title,
                due,
                start,
                rank,
                task_id,
                tags,
                milestone,
                watch_root: watch_root.map(PathBuf::from),
            });
        }

        Ok(tasks)
    }

    fn fetch_file_deps(
        &self,
        file_id: i64,
        file_path: &Path,
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>), StoreError> {
        // Get outgoing references (files this file imports).
        let mut imports_stmt = self.conn.prepare(
            "SELECT target_path FROM file_references WHERE source_file_id = ?1 ORDER BY target_path",
        )?;
        let imports: Vec<PathBuf> = imports_stmt
            .query_map(params![file_id], |row| {
                Ok(PathBuf::from(row.get::<_, String>(0)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Get incoming references (files that import this file).
        let mut imported_by_stmt = self.conn.prepare(
            "SELECT tf.file_path
             FROM file_references fr
             JOIN task_files tf ON fr.source_file_id = tf.id
             WHERE fr.target_path = ?1
             ORDER BY tf.file_path",
        )?;
        let imported_by: Vec<PathBuf> = imported_by_stmt
            .query_map(params![file_path.to_string_lossy().to_string()], |row| {
                Ok(PathBuf::from(row.get::<_, String>(0)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((imports, imported_by))
    }
}

impl Store for SqliteStore {
    fn upsert_task_file(&mut self, file: &TaskFile) -> Result<i64, StoreError> {
        let path_str = file.file_path.to_string_lossy();
        let watch_root_str = file.watch_root.as_ref().map(|p| p.to_string_lossy().to_string());
        self.conn.execute(
            "INSERT INTO task_files (file_path, watch_root, title, eval_hash, updated_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(file_path) DO UPDATE SET
               watch_root = excluded.watch_root,
               title = excluded.title,
               eval_hash = excluded.eval_hash,
               updated_at = datetime('now')",
            params![path_str.as_ref(), watch_root_str, file.title, file.eval_hash],
        )?;
        // Always query by path: last_insert_rowid() is unreliable after
        // ON CONFLICT DO UPDATE — it can return a stale rowid from a
        // previous INSERT into a different table.
        let id: i64 = self.conn.query_row(
            "SELECT id FROM task_files WHERE file_path = ?1",
            params![path_str.as_ref()],
            |row| row.get(0),
        )?;
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
                "INSERT INTO tasks (task_file_id, title, is_done, position, milestone, due, start, rank, task_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![file_id, task.title, task.is_done, task.position, task.milestone, task.due, task.start, task.rank, task.task_id],
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
            "DELETE FROM task_files WHERE file_path = ?1",
            params![path_str.as_ref()],
        )?;
        Ok(())
    }

    fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskView>, StoreError> {
        let (sql, params) = Self::build_task_query(filter);
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        self.fetch_task_views(&mut stmt, params_refs.as_slice())
    }

    fn get_file_hash(&self, path: &Path) -> Result<Option<String>, StoreError> {
        let path_str = path.to_string_lossy();
        let hash = self
            .conn
            .query_row(
                "SELECT eval_hash FROM task_files WHERE file_path = ?1",
                params![path_str.as_ref()],
                |row| row.get(0),
            )
            .optional()?;
        Ok(hash)
    }

    fn list_files(&self) -> Result<Vec<FileView>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT tf.file_path, tf.watch_root, tf.title, tf.updated_at, COUNT(t.id)
             FROM task_files tf
             LEFT JOIN tasks t ON t.task_file_id = tf.id
             GROUP BY tf.id
             ORDER BY tf.file_path",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FileView {
                file_path: PathBuf::from(row.get::<_, String>(0)?),
                watch_root: row.get::<_, Option<String>>(1)?.map(PathBuf::from),
                title: row.get(2)?,
                updated_at: row.get(3)?,
                task_count: row.get(4)?,
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

    fn upsert_file_references(
        &mut self,
        source_file_id: i64,
        target_paths: &[PathBuf],
    ) -> Result<(), StoreError> {
        // Delete existing references for this source file.
        self.conn.execute(
            "DELETE FROM file_references WHERE source_file_id = ?1",
            params![source_file_id],
        )?;

        // Insert new references.
        let mut stmt = self.conn.prepare(
            "INSERT INTO file_references (source_file_id, target_path) VALUES (?1, ?2)",
        )?;
        for target in target_paths {
            stmt.execute(params![source_file_id, target.to_string_lossy().to_string()])?;
        }

        Ok(())
    }

    fn get_file_dependencies(&self, path: &Path) -> Result<Option<super::FileDependencies>, StoreError> {
        // Get file info.
        let file_info: Option<(PathBuf, Option<String>, i64)> = self.conn
            .query_row(
                "SELECT file_path, title, id FROM task_files WHERE file_path = ?1",
                params![path.to_string_lossy().to_string()],
                |row| Ok((
                    PathBuf::from(row.get::<_, String>(0)?),
                    row.get(1)?,
                    row.get(2)?,
                )),
            )
            .optional()?;

        let Some((file_path, file_title, file_id)) = file_info else {
            return Ok(None);
        };

        let (imports, imported_by) = self.fetch_file_deps(file_id, &file_path)?;

        Ok(Some(super::FileDependencies {
            file_path,
            file_title,
            imports,
            imported_by,
        }))
    }

    fn list_file_dependencies(&self) -> Result<Vec<super::FileDependencies>, StoreError> {
        // Get all files.
        let mut files_stmt = self.conn.prepare(
            "SELECT id, file_path, title FROM task_files ORDER BY file_path",
        )?;
        let file_rows: Vec<(i64, PathBuf, Option<String>)> = files_stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    PathBuf::from(row.get::<_, String>(1)?),
                    row.get(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut results = Vec::new();
        for (file_id, file_path, file_title) in file_rows {
            let (imports, imported_by) = self.fetch_file_deps(file_id, &file_path)?;

            results.push(super::FileDependencies {
                file_path,
                file_title,
                imports,
                imported_by,
            });
        }

        Ok(results)
    }

    fn find_task_by_id(&self, id_or_mask: &str) -> Result<super::TaskWithFile, super::StoreError> {
        let (query, param) = if let Some(suffix) = id_or_mask.strip_prefix('*') {
            (
                "SELECT t.task_id, t.title, t.is_done, tf.file_path, tf.eval_hash
                 FROM tasks t
                 JOIN task_files tf ON t.task_file_id = tf.id
                 WHERE t.task_id LIKE ?1",
                format!("%{suffix}"),
            )
        } else {
            let canonical = crate::id::parse_task_id(id_or_mask)
                .unwrap_or_else(|_| id_or_mask.to_string());
            (
                "SELECT t.task_id, t.title, t.is_done, tf.file_path, tf.eval_hash
                 FROM tasks t
                 JOIN task_files tf ON t.task_file_id = tf.id
                 WHERE t.task_id = ?1",
                canonical,
            )
        };

        let mut stmt = self.conn.prepare(query)?;
        let mut rows = stmt.query(params![param])?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(super::TaskWithFile {
                task_id: row.get(0)?,
                task_title: row.get(1)?,
                is_done: row.get(2)?,
                file_path: PathBuf::from(row.get::<_, String>(3)?),
                file_hash: row.get(4)?,
            });
        }

        match results.len() {
            0 => Err(super::StoreError::Path(format!(
                "task not found: {id_or_mask}"
            ))),
            1 => Ok(results.into_iter().next().expect("exactly one result")),
            num_matches => Err(super::StoreError::Path(format!(
                "ambiguous task ID pattern '{id_or_mask}': matched {num_matches} tasks"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::uninlined_format_args)]
mod tests {
    use super::*;
    use super::super::{FileBinding, PropertyKind, TaskFile, TaskFilter, TaskProperty, TaskRecord};

    fn test_store() -> SqliteStore {
        SqliteStore::open_memory().unwrap()
    }

    fn make_task_file(path: &str, hash: &str) -> TaskFile {
        TaskFile {
            id: None,
            file_path: PathBuf::from(path),
            watch_root: None,
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
            milestone: None,
            due: None,
            start: None,
            rank: None,
            task_id: None,
        }
    }

    fn make_due_prop(value: &str) -> TaskProperty {
        TaskProperty {
            id: None,
            task_id: 0,
            kind: PropertyKind::Due,
            key: "mindtape.due".to_string(),
            value: value.to_string(),
        }
    }

    fn make_tag_prop(value: &str) -> TaskProperty {
        TaskProperty {
            id: None,
            task_id: 0,
            kind: PropertyKind::Tag,
            key: "mindtape.tag".to_string(),
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
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('task_files','tasks','task_properties','file_bindings','file_references')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 5);
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
        assert_eq!(version, 9);
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
        let mut tasks = vec![
            make_record("Buy milk", false, 0),
            make_record("Done thing", true, 1),
            make_record("Urgent", false, 2),
        ];
        tasks[0].due = Some("2026-03-01".to_string());
        tasks[2].due = Some("2026-01-15".to_string());
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
                tags: vec!["work".to_string()],
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
                tags: vec!["work".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views[0].due, Some("2026-01-15".to_string()));
        assert_eq!(views[0].tags, vec!["work"]);
    }

    // --- new filter & FTS5 tests ---

    /// Richer seed data with milestones and multi-tag tasks for filter tests.
    fn seed_store_rich(store: &mut SqliteStore) -> i64 {
        let file_id = store
            .upsert_task_file(&make_task_file("project.typ", "rich"))
            .unwrap();
        let mut tasks = vec![
            make_record("Deploy service", false, 0),
            make_record("Write report", false, 1),
            make_record("Fix login bug", true, 2),
            make_record("Plan sprint", false, 3),
        ];
        tasks[0].milestone = Some("Ops > Deployment".to_string());
        tasks[0].due = Some("2026-02-15".to_string());
        tasks[1].milestone = Some("Docs".to_string());
        tasks[1].due = Some("2026-03-01".to_string());
        tasks[2].milestone = Some("Engineering > Auth".to_string());
        tasks[2].due = Some("2026-01-10".to_string());
        tasks[3].milestone = Some("Planning".to_string());
        tasks[3].due = Some("2026-04-01".to_string());

        let props = vec![
            vec![make_tag_prop("ops"), make_tag_prop("urgent"), make_due_prop("2026-02-15")],
            vec![make_tag_prop("docs"), make_due_prop("2026-03-01")],
            vec![make_tag_prop("ops"), make_due_prop("2026-01-10")],
            vec![make_tag_prop("planning"), make_due_prop("2026-04-01")],
        ];
        store.upsert_tasks(file_id, &tasks, &props).unwrap();
        file_id
    }

    #[test]
    fn query_filter_by_due_after() {
        let mut store = test_store();
        seed_store(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                due_after: Some("2026-02-01".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Buy milk");
    }

    #[test]
    fn query_filter_by_due_range() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                due_after: Some("2026-02-01".to_string()),
                due_before: Some("2026-03-15".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 2);
        assert!(views.iter().any(|v| v.title == "Deploy service"));
        assert!(views.iter().any(|v| v.title == "Write report"));
    }

    #[test]
    fn query_filter_by_milestone() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                milestone: Some("Ops".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Deploy service");
    }

    #[test]
    fn query_filter_by_milestone_case_insensitive() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                milestone: Some("engineering".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Fix login bug");
    }

    #[test]
    fn query_filter_by_title_contains() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                title_contains: Some("report".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Write report");
    }

    #[test]
    fn query_filter_multi_tag_and() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        // "Deploy service" has both "ops" and "urgent"
        let views = store
            .query_tasks(&TaskFilter {
                tags: vec!["ops".to_string(), "urgent".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Deploy service");
    }

    #[test]
    fn query_filter_multi_tag_single_match() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        // "ops" tag matches both "Deploy service" and "Fix login bug"
        let views = store
            .query_tasks(&TaskFilter {
                tags: vec!["ops".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 2);
    }

    #[test]
    fn search_by_title_fts() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                search: Some("deploy".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Deploy service");
    }

    #[test]
    fn search_by_milestone_fts() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                search: Some("auth".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Fix login bug");
    }

    #[test]
    fn search_combined_with_filters() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        // Search for "service" but only pending tasks with "ops" tag
        let views = store
            .query_tasks(&TaskFilter {
                search: Some("service".to_string()),
                done: Some(false),
                tags: vec!["ops".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Deploy service");
    }

    #[test]
    fn search_no_results() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        let views = store
            .query_tasks(&TaskFilter {
                search: Some("nonexistent".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert!(views.is_empty());
    }

    #[test]
    fn search_with_limit() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        // Search broadly, limit to 1
        let views = store
            .query_tasks(&TaskFilter {
                search: Some("service OR report OR bug OR sprint".to_string()),
                limit: Some(1),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
    }

    #[test]
    fn fts_sync_on_reindex() {
        let mut store = test_store();
        let file_id = store
            .upsert_task_file(&make_task_file("t.typ", "h1"))
            .unwrap();
        store
            .upsert_tasks(file_id, &[make_record("Old title", false, 0)], &[vec![]])
            .unwrap();

        // Search should find the old title
        let views = store
            .query_tasks(&TaskFilter {
                search: Some("Old".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);

        // Re-index with new title (upsert_tasks deletes then re-inserts)
        store
            .upsert_tasks(file_id, &[make_record("New title", false, 0)], &[vec![]])
            .unwrap();

        // Old title should not match
        let views = store
            .query_tasks(&TaskFilter {
                search: Some("Old".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert!(views.is_empty());

        // New title should match
        let views = store
            .query_tasks(&TaskFilter {
                search: Some("New".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "New title");
    }

    #[test]
    fn query_all_filters_combined() {
        let mut store = test_store();
        seed_store_rich(&mut store);
        // Only "Deploy service" matches: pending + tag:ops + due range + milestone "Ops" + search
        let views = store
            .query_tasks(&TaskFilter {
                done: Some(false),
                tags: vec!["ops".to_string()],
                due_after: Some("2026-02-01".to_string()),
                due_before: Some("2026-03-01".to_string()),
                milestone: Some("Deployment".to_string()),
                search: Some("deploy".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Deploy service");
    }

    // --- query_tasks: --with (presence) filter ---

    #[test]
    fn query_filter_with_due() {
        let mut store = test_store();
        seed_store(&mut store);
        // "Buy milk" has due=2026-03-01, "Urgent" has due=2026-01-15, "Done thing" has no due
        let views = store
            .query_tasks(&TaskFilter {
                with: vec!["due".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 2);
        assert!(views.iter().all(|v| v.due.is_some()));
    }

    #[test]
    fn query_filter_with_rank() {
        let mut store = test_store();
        let file_id = store
            .upsert_task_file(&make_task_file("ranked.typ", "r1"))
            .unwrap();
        let mut t1 = make_record("Ranked", false, 0);
        t1.rank = Some(75);
        let t2 = make_record("Unranked", false, 1);
        store.upsert_tasks(file_id, &[t1, t2], &[vec![], vec![]]).unwrap();
        let views = store
            .query_tasks(&TaskFilter {
                with: vec!["rank".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Ranked");
    }

    #[test]
    fn query_filter_with_tag() {
        let mut store = test_store();
        seed_store(&mut store);
        // Only "Urgent" has a tag
        let views = store
            .query_tasks(&TaskFilter {
                with: vec!["tag".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Urgent");
    }

    #[test]
    fn query_filter_with_multiple() {
        let mut store = test_store();
        seed_store(&mut store);
        // Only "Urgent" has both due AND tag
        let views = store
            .query_tasks(&TaskFilter {
                with: vec!["due".to_string(), "tag".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Urgent");
    }

    // --- query_tasks: --without (absence) filter ---

    #[test]
    fn query_filter_without_due() {
        let mut store = test_store();
        seed_store(&mut store);
        // "Done thing" has no due
        let views = store
            .query_tasks(&TaskFilter {
                without: vec!["due".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Done thing");
    }

    #[test]
    fn query_filter_without_tag() {
        let mut store = test_store();
        seed_store(&mut store);
        // "Buy milk" and "Done thing" have no tag
        let views = store
            .query_tasks(&TaskFilter {
                without: vec!["tag".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 2);
        assert!(views.iter().all(|v| v.tags.is_empty()));
    }

    #[test]
    fn query_filter_without_rank() {
        let mut store = test_store();
        let file_id = store
            .upsert_task_file(&make_task_file("ranked.typ", "r1"))
            .unwrap();
        let mut t1 = make_record("Ranked", false, 0);
        t1.rank = Some(75);
        let t2 = make_record("Unranked", false, 1);
        store.upsert_tasks(file_id, &[t1, t2], &[vec![], vec![]]).unwrap();
        let views = store
            .query_tasks(&TaskFilter {
                without: vec!["rank".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Unranked");
    }

    #[test]
    fn query_filter_without_multiple() {
        let mut store = test_store();
        seed_store(&mut store);
        // "Done thing" has neither due nor tag
        let views = store
            .query_tasks(&TaskFilter {
                without: vec!["due".to_string(), "tag".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Done thing");
    }

    #[test]
    fn query_filter_with_and_without_combined() {
        let mut store = test_store();
        seed_store(&mut store);
        // "Buy milk" has due but no tag
        let views = store
            .query_tasks(&TaskFilter {
                with: vec!["due".to_string()],
                without: vec!["tag".to_string()],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].title, "Buy milk");
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
                start: None,
                rank: None,
                tags: vec!["work".to_string(), "urgent".to_string()],
                id: Some("019c5b9b-7317-77b1-bf52-ce7a298cfcad".to_string()),
                position: 0,
                milestone: Some("Heading".to_string()),
            }],
            title: Some("Heading".to_string()),
            bindings: vec![("note".to_string(), "string".to_string(), "\"hi\"".to_string())],
            dependencies: vec![],
        };

        let (tf, tasks, props, bindings) =
            super::super::to_store_records(&eval_result, Path::new("test.typ"), "hash", None);

        assert_eq!(tf.file_path, PathBuf::from("test.typ"));
        assert_eq!(tf.title, Some("Heading".to_string()));
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Test");
        assert_eq!(tasks[0].due, Some("2026-03-01".to_string()));
        assert_eq!(tasks[0].task_id, Some("019c5b9b-7317-77b1-bf52-ce7a298cfcad".to_string()));
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].len(), 4); // 1 id + 1 due + 2 tags
        assert_eq!(props[0][0].kind, PropertyKind::Id);
        assert_eq!(props[0][0].value, "019c5b9b-7317-77b1-bf52-ce7a298cfcad");
        assert_eq!(props[0][1].kind, PropertyKind::Due);
        assert_eq!(props[0][1].value, "2026-03-01");
        assert_eq!(props[0][2].kind, PropertyKind::Tag);
        assert_eq!(props[0][2].value, "work");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "note");
    }

    // --- PropertyKind ---

    #[test]
    fn property_kind_roundtrip() {
        assert_eq!(PropertyKind::try_from_str("mindtape.due"), Some(PropertyKind::Due));
        assert_eq!(PropertyKind::try_from_str("mindtape.start"), Some(PropertyKind::Start));
        assert_eq!(PropertyKind::try_from_str("mindtape.rank"), Some(PropertyKind::Rank));
        assert_eq!(PropertyKind::try_from_str("mindtape.tag"), Some(PropertyKind::Tag));
        assert_eq!(PropertyKind::try_from_str("mindtape.id"), Some(PropertyKind::Id));
        assert_eq!(PropertyKind::try_from_str("unknown"), None);
        assert_eq!(PropertyKind::Due.as_str(), "mindtape.due");
        assert_eq!(PropertyKind::Start.as_str(), "mindtape.start");
        assert_eq!(PropertyKind::Rank.as_str(), "mindtape.rank");
        assert_eq!(PropertyKind::Tag.as_str(), "mindtape.tag");
        assert_eq!(PropertyKind::Id.as_str(), "mindtape.id");
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
        assert_eq!(files[0].file_path, PathBuf::from("a.typ"));
        assert_eq!(files[0].task_count, 2);
        assert_eq!(files[0].title, Some("Test".to_string()));
        assert_eq!(files[1].file_path, PathBuf::from("b.typ"));
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

    // --- file references ---

    #[test]
    fn upsert_file_references_basic() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("main.typ", "h1")).unwrap();

        let refs = vec![PathBuf::from("lib/utils.typ"), PathBuf::from("lib/helpers.typ")];
        store.upsert_file_references(file_id, &refs).unwrap();

        let count: i32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM file_references WHERE source_file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn upsert_file_references_replaces() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("main.typ", "h1")).unwrap();

        store.upsert_file_references(file_id, &[PathBuf::from("old.typ")]).unwrap();
        store
            .upsert_file_references(file_id, &[PathBuf::from("new1.typ"), PathBuf::from("new2.typ")])
            .unwrap();

        let count: i32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM file_references WHERE source_file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        let has_old: bool = store
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM file_references WHERE source_file_id = ?1 AND target_path = 'old.typ')",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!has_old);
    }

    #[test]
    fn get_file_dependencies_returns_none_for_unknown() {
        let store = test_store();
        let deps = store.get_file_dependencies(Path::new("unknown.typ")).unwrap();
        assert!(deps.is_none());
    }

    #[test]
    fn get_file_dependencies_returns_imports_and_imported_by() {
        let mut store = test_store();

        // Create files: main.typ imports lib/utils.typ
        let main_id = store.upsert_task_file(&make_task_file("main.typ", "h1")).unwrap();
        let _utils_id = store.upsert_task_file(&make_task_file("lib/utils.typ", "h2")).unwrap();

        store
            .upsert_file_references(main_id, &[PathBuf::from("lib/utils.typ")])
            .unwrap();

        // Query main.typ dependencies
        let main_deps = store
            .get_file_dependencies(Path::new("main.typ"))
            .unwrap()
            .unwrap();
        assert_eq!(main_deps.file_path, PathBuf::from("main.typ"));
        assert_eq!(main_deps.imports.len(), 1);
        assert_eq!(main_deps.imports[0], PathBuf::from("lib/utils.typ"));
        assert_eq!(main_deps.imported_by.len(), 0);

        // Query lib/utils.typ dependencies
        let utils_deps = store
            .get_file_dependencies(Path::new("lib/utils.typ"))
            .unwrap()
            .unwrap();
        assert_eq!(utils_deps.file_path, PathBuf::from("lib/utils.typ"));
        assert_eq!(utils_deps.imports.len(), 0);
        assert_eq!(utils_deps.imported_by.len(), 1);
        assert_eq!(utils_deps.imported_by[0], PathBuf::from("main.typ"));
    }

    #[test]
    fn list_file_dependencies_all() {
        let mut store = test_store();

        let main_id = store.upsert_task_file(&make_task_file("main.typ", "h1")).unwrap();
        let utils_id = store.upsert_task_file(&make_task_file("lib/utils.typ", "h2")).unwrap();
        let _helper_id = store.upsert_task_file(&make_task_file("lib/helper.typ", "h3")).unwrap();

        store
            .upsert_file_references(main_id, &[PathBuf::from("lib/utils.typ"), PathBuf::from("lib/helper.typ")])
            .unwrap();
        store
            .upsert_file_references(utils_id, &[PathBuf::from("lib/helper.typ")])
            .unwrap();

        let all_deps = store.list_file_dependencies().unwrap();
        assert_eq!(all_deps.len(), 3);

        // main.typ: imports 2, imported by 0
        let main = all_deps.iter().find(|d| d.file_path == Path::new("main.typ")).unwrap();
        assert_eq!(main.imports.len(), 2);
        assert_eq!(main.imported_by.len(), 0);

        // lib/utils.typ: imports 1, imported by 1
        let utils = all_deps.iter().find(|d| d.file_path == Path::new("lib/utils.typ")).unwrap();
        assert_eq!(utils.imports.len(), 1);
        assert_eq!(utils.imported_by.len(), 1);

        // lib/helper.typ: imports 0, imported by 2
        let helper = all_deps.iter().find(|d| d.file_path == Path::new("lib/helper.typ")).unwrap();
        assert_eq!(helper.imports.len(), 0);
        assert_eq!(helper.imported_by.len(), 2);
    }

    #[test]
    fn file_references_cascade_on_file_delete() {
        let mut store = test_store();
        let file_id = store.upsert_task_file(&make_task_file("main.typ", "h1")).unwrap();

        store
            .upsert_file_references(file_id, &[PathBuf::from("lib/utils.typ")])
            .unwrap();

        store.remove_task_file(Path::new("main.typ")).unwrap();

        let count: i32 = store
            .conn
            .query_row("SELECT count(*) FROM file_references", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
