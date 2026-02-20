pub const SQL: &str = "
CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
    title,
    milestone,
    content='tasks',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS tasks_ai AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts(rowid, title, milestone)
    VALUES (new.id, new.title, new.milestone);
END;

CREATE TRIGGER IF NOT EXISTS tasks_ad AFTER DELETE ON tasks BEGIN
    INSERT INTO tasks_fts(tasks_fts, rowid, title, milestone)
    VALUES('delete', old.id, old.title, old.milestone);
END;

CREATE TRIGGER IF NOT EXISTS tasks_au AFTER UPDATE ON tasks BEGIN
    INSERT INTO tasks_fts(tasks_fts, rowid, title, milestone)
    VALUES('delete', old.id, old.title, old.milestone);
    INSERT INTO tasks_fts(rowid, title, milestone)
    VALUES (new.id, new.title, new.milestone);
END;

INSERT INTO tasks_fts(rowid, title, milestone)
SELECT id, title, milestone FROM tasks;
";
