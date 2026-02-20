pub const SQL: &str = "
CREATE INDEX IF NOT EXISTS idx_tasks_title ON tasks(title COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_bindings_name ON file_bindings(name COLLATE NOCASE);
";
