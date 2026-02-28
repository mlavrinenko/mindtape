pub const SQL: &str = "
-- v7: clean schema with denormalized due/task_id on tasks table.
-- Since the DB is a derived cache, we drop everything and recreate.

DROP TABLE IF EXISTS tasks_fts;
DROP TABLE IF EXISTS file_references;
DROP TABLE IF EXISTS file_bindings;
DROP TABLE IF EXISTS task_properties;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS task_files;

CREATE TABLE task_files (
    id            INTEGER PRIMARY KEY,
    file_path     TEXT    NOT NULL UNIQUE,
    watch_root    TEXT,
    title         TEXT,
    eval_hash     TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE tasks (
    id            INTEGER PRIMARY KEY,
    task_file_id  INTEGER NOT NULL REFERENCES task_files(id) ON DELETE CASCADE,
    title         TEXT    NOT NULL,
    is_done       INTEGER NOT NULL DEFAULT 0,
    position      INTEGER NOT NULL DEFAULT 0,
    milestone     TEXT,
    due           TEXT,
    task_id       TEXT
);

CREATE TABLE task_properties (
    id            INTEGER PRIMARY KEY,
    task_id       INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    kind          TEXT    NOT NULL,
    key           TEXT    NOT NULL,
    value         TEXT    NOT NULL
);

CREATE TABLE file_bindings (
    id            INTEGER PRIMARY KEY,
    task_file_id  INTEGER NOT NULL REFERENCES task_files(id) ON DELETE CASCADE,
    name          TEXT    NOT NULL,
    value_type    TEXT    NOT NULL,
    value_json    TEXT    NOT NULL
);

CREATE TABLE file_references (
    id             INTEGER PRIMARY KEY,
    source_file_id INTEGER NOT NULL REFERENCES task_files(id) ON DELETE CASCADE,
    target_path    TEXT    NOT NULL
);

-- Indexes
CREATE INDEX idx_tasks_file ON tasks(task_file_id);
CREATE INDEX idx_tasks_title ON tasks(title COLLATE NOCASE);
CREATE INDEX idx_tasks_due ON tasks(due);
CREATE INDEX idx_tasks_task_id ON tasks(task_id);
CREATE INDEX idx_props_task ON task_properties(task_id);
CREATE INDEX idx_props_kind ON task_properties(kind);
CREATE INDEX idx_bindings_file ON file_bindings(task_file_id);
CREATE INDEX idx_bindings_name ON file_bindings(name COLLATE NOCASE);
CREATE INDEX idx_refs_source ON file_references(source_file_id);
CREATE INDEX idx_refs_target ON file_references(target_path);

-- FTS5 for full-text search
CREATE VIRTUAL TABLE tasks_fts USING fts5(
    title,
    milestone,
    content='tasks',
    content_rowid='id'
);

CREATE TRIGGER tasks_ai AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts(rowid, title, milestone)
    VALUES (new.id, new.title, new.milestone);
END;

CREATE TRIGGER tasks_ad AFTER DELETE ON tasks BEGIN
    INSERT INTO tasks_fts(tasks_fts, rowid, title, milestone)
    VALUES('delete', old.id, old.title, old.milestone);
END;

CREATE TRIGGER tasks_au AFTER UPDATE ON tasks BEGIN
    INSERT INTO tasks_fts(tasks_fts, rowid, title, milestone)
    VALUES('delete', old.id, old.title, old.milestone);
    INSERT INTO tasks_fts(rowid, title, milestone)
    VALUES (new.id, new.title, new.milestone);
END;
";
