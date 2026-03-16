//! `SQLite` implementation of the `Store` trait.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use super::{
    FileBinding, FileView, IndexStats, SortDir, SortField, Store, StoreError, TaskFile, TaskFilter,
    TaskProperty, TaskRecord, TaskView,
};

// ---------------------------------------------------------------------------
// SqliteStore
// ---------------------------------------------------------------------------

/// Row tuple from a task query JOIN.
type TaskRow = (
    i64,
    String,
    bool,
    i32,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
);

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
    /// Uses unnamed `?` placeholders for cleaner code — rusqlite binds them positionally.
    ///
    /// # Errors
    ///
    /// Returns `StoreError::FilterExpr` if the `expr` filter is invalid.
    fn build_task_query(filter: &TaskFilter) -> Result<(String, Vec<String>), super::StoreError> {
        // Pre-translate the expression filter (if any) so we know whether a
        // FTS JOIN is needed before we start building the SQL string.
        let expr_fragment = filter
            .expr
            .as_ref()
            .map(|e| super::filter_expr::translate_expr(e))
            .transpose()?;

        let mut sql = String::from(
            "SELECT t.id, t.title, t.is_done, t.position, t.milestone, \
             t.due, t.start, t.rank, t.task_id, tf.file_path, tf.title, tf.watch_root
             FROM tasks t
             JOIN task_files tf ON t.task_file_id = tf.id",
        );

        let needs_fts = expr_fragment.as_ref().is_some_and(|f| f.needs_fts_join);
        if needs_fts {
            sql.push_str(" JOIN tasks_fts fts ON fts.rowid = t.id");
        }

        let (mut conditions, mut params) = Self::build_filter_conditions(filter);

        // Exclude imported duplicates: when file A imports file B, evaluating A
        // produces tasks from both A and B.  Keep only the source-file row by
        // filtering out rows whose file imports another file that also owns a
        // task with the same task_id.
        conditions.push(
            "NOT EXISTS (\
             SELECT 1 FROM tasks t2 \
             JOIN task_files tf2 ON t2.task_file_id = tf2.id \
             JOIN file_references fr ON fr.source_file_id = t.task_file_id \
                 AND fr.target_path = tf2.file_path \
             WHERE t2.task_id IS NOT NULL \
                 AND t2.task_id = t.task_id \
                 AND t2.task_file_id != t.task_file_id\
            )"
            .to_string(),
        );

        if let Some(fragment) = expr_fragment {
            conditions.push(fragment.condition);
            params.extend(fragment.params);
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY ");
        if filter.sort.is_empty() {
            sql.push_str("tf.file_path, t.position");
        } else {
            let clauses: Vec<String> = filter
                .sort
                .iter()
                .copied()
                .map(Self::sort_spec_to_sql)
                .collect();
            sql.push_str(&clauses.join(", "));
        }

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        Ok((sql, params))
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

        if let Some(ref folder) = filter.folder {
            conditions.push("tf.file_path LIKE ? || '%'".to_string());
            params.push(folder.to_string_lossy().to_string());
        }

        if let Some(ref root) = filter.watch_root {
            conditions.push("tf.watch_root = ?".to_string());
            params.push(root.to_string_lossy().to_string());
        }

        (conditions, params)
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
        let id_refs: Vec<&dyn rusqlite::types::ToSql> = row_ids
            .iter()
            .map(|id| id as &dyn rusqlite::types::ToSql)
            .collect();

        use std::collections::HashMap;
        let mut tags_map: HashMap<i64, Vec<String>> = HashMap::new();
        let tag_rows = tags_stmt.query_map(id_refs.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in tag_rows {
            let (task_id, value) = row?;
            tags_map.entry(task_id).or_default().push(value);
        }

        let tasks = task_rows
            .into_iter()
            .map(|row| Self::task_row_to_view(row, &mut tags_map))
            .collect();

        Ok(tasks)
    }

    /// Convert a single task row + its tags into a [`TaskView`].
    fn task_row_to_view(
        row: TaskRow,
        tags_map: &mut std::collections::HashMap<i64, Vec<String>>,
    ) -> TaskView {
        let (
            row_id,
            title,
            is_done,
            position,
            milestone,
            due,
            start,
            rank,
            task_id,
            file_path,
            file_title,
            watch_root,
        ) = row;
        let tags = tags_map.remove(&row_id).unwrap_or_default();
        TaskView {
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
        }
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
        let watch_root_str = file
            .watch_root
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        self.conn.execute(
            "INSERT INTO task_files (file_path, watch_root, title, eval_hash, updated_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(file_path) DO UPDATE SET
               watch_root = excluded.watch_root,
               title = excluded.title,
               eval_hash = excluded.eval_hash,
               updated_at = datetime('now')",
            params![
                path_str.as_ref(),
                watch_root_str,
                file.title,
                file.eval_hash
            ],
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
        tx.execute(
            "DELETE FROM tasks WHERE task_file_id = ?1",
            params![file_id],
        )?;

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
        let (sql, params) = Self::build_task_query(filter)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();

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
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    fn get_stats(&self) -> Result<IndexStats, StoreError> {
        let (file_count, task_count, done_count, last_updated): (i64, i64, i64, Option<String>) =
            self.conn.query_row(
                "SELECT
                 COUNT(DISTINCT tf.id),
                 COUNT(t.id),
                 SUM(CASE WHEN t.is_done THEN 1 ELSE 0 END),
                 MAX(tf.updated_at)
             FROM task_files tf
             LEFT JOIN tasks t ON t.task_file_id = tf.id",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                        row.get(3)?,
                    ))
                },
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
        let mut stmt = self
            .conn
            .prepare("INSERT INTO file_references (source_file_id, target_path) VALUES (?1, ?2)")?;
        for target in target_paths {
            stmt.execute(params![
                source_file_id,
                target.to_string_lossy().to_string()
            ])?;
        }

        Ok(())
    }

    fn get_file_dependencies(
        &self,
        path: &Path,
    ) -> Result<Option<super::FileDependencies>, StoreError> {
        // Get file info.
        let file_info: Option<(PathBuf, Option<String>, i64)> = self
            .conn
            .query_row(
                "SELECT file_path, title, id FROM task_files WHERE file_path = ?1",
                params![path.to_string_lossy().to_string()],
                |row| {
                    Ok((
                        PathBuf::from(row.get::<_, String>(0)?),
                        row.get(1)?,
                        row.get(2)?,
                    ))
                },
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
        let mut files_stmt = self
            .conn
            .prepare("SELECT id, file_path, title FROM task_files ORDER BY file_path")?;
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
            let canonical =
                crate::id::parse_task_id(id_or_mask).unwrap_or_else(|_| id_or_mask.to_string());
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

    fn upsert_file_error(
        &mut self,
        path: &Path,
        watch_root: Option<&Path>,
        error: &str,
    ) -> Result<(), StoreError> {
        let path_str = path.to_string_lossy();
        let watch_root_str = watch_root.map(|p| p.to_string_lossy().to_string());
        self.conn.execute(
            "INSERT INTO file_errors (file_path, watch_root, error, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))
             ON CONFLICT(file_path) DO UPDATE SET
               watch_root = excluded.watch_root,
               error = excluded.error,
               updated_at = datetime('now')",
            params![path_str.as_ref(), watch_root_str, error],
        )?;
        Ok(())
    }

    fn remove_file_error(&mut self, path: &Path) -> Result<(), StoreError> {
        let path_str = path.to_string_lossy();
        self.conn.execute(
            "DELETE FROM file_errors WHERE file_path = ?1",
            params![path_str.as_ref()],
        )?;
        Ok(())
    }

    fn list_file_errors(&self) -> Result<Vec<super::FileError>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT file_path, watch_root, error, updated_at
             FROM file_errors ORDER BY file_path",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(super::FileError {
                file_path: PathBuf::from(row.get::<_, String>(0)?),
                watch_root: row.get::<_, Option<String>>(1)?.map(PathBuf::from),
                error: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }
}

#[cfg(test)]
#[path = "sqlite_tests.rs"]
mod tests;
