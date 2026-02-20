pub const SQL: &str = "
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
