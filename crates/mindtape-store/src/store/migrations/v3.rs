pub const SQL: &str = "
CREATE TABLE IF NOT EXISTS file_references (
    id            INTEGER PRIMARY KEY,
    source_file_id INTEGER NOT NULL REFERENCES task_files(id) ON DELETE CASCADE,
    target_path    TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_refs_source ON file_references(source_file_id);
CREATE INDEX IF NOT EXISTS idx_refs_target ON file_references(target_path);
";
