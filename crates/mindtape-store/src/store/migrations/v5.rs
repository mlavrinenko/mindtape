pub const SQL: &str = "
ALTER TABLE task_files RENAME COLUMN relative_path TO file_path;
ALTER TABLE task_files ADD COLUMN watch_root TEXT;
";
