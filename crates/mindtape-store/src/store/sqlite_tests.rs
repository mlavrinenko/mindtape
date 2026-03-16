#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::uninlined_format_args
)]
use super::super::{FileBinding, PropertyKind, TaskFile, TaskFilter, TaskProperty, TaskRecord};
use super::*;

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
    assert_eq!(version, 10);
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
    let file_id = store
        .upsert_task_file(&make_task_file("t.typ", "h"))
        .unwrap();

    let tasks = vec![
        make_record("Task A", false, 0),
        make_record("Task B", true, 1),
    ];
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
    let file_id = store
        .upsert_task_file(&make_task_file("t.typ", "h"))
        .unwrap();

    let tasks1 = vec![make_record("Old", false, 0)];
    store.upsert_tasks(file_id, &tasks1, &[vec![]]).unwrap();

    let tasks2 = vec![
        make_record("New A", false, 0),
        make_record("New B", false, 1),
    ];
    store
        .upsert_tasks(file_id, &tasks2, &[vec![], vec![]])
        .unwrap();

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
    let file_id = store
        .upsert_task_file(&make_task_file("t.typ", "h"))
        .unwrap();

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
    let file_id = store
        .upsert_task_file(&make_task_file("t.typ", "h"))
        .unwrap();

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
    let file_id = store
        .upsert_task_file(&make_task_file("t.typ", "h"))
        .unwrap();

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
    let file_id = store
        .upsert_task_file(&make_task_file("t.typ", "h"))
        .unwrap();

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
            expr: Some("has_tag(\"work\")".into()),
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
            expr: Some("due <= \"2026-02-01\"".into()),
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
        .upsert_tasks(file_id2, &[make_record("Other", false, 0)], &[vec![]])
        .unwrap();

    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("file == \"todo.typ\"".into()),
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
            expr: Some("has_tag(\"work\")".into()),
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
        vec![
            make_tag_prop("ops"),
            make_tag_prop("urgent"),
            make_due_prop("2026-02-15"),
        ],
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
            expr: Some("due >= \"2026-02-01\"".into()),
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
            expr: Some("due >= \"2026-02-01\" && due <= \"2026-03-15\"".into()),
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
            expr: Some("contains(milestone, \"Ops\")".into()),
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
            expr: Some("contains(milestone, \"engineering\")".into()),
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
            expr: Some("contains(title, \"report\")".into()),
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
            expr: Some("has_tag(\"ops\") && has_tag(\"urgent\")".into()),
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
            expr: Some("has_tag(\"ops\")".into()),
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
            expr: Some("search(\"deploy\")".into()),
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
            expr: Some("search(\"auth\")".into()),
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
            done: Some(false),
            expr: Some("search(\"service\") && has_tag(\"ops\")".into()),
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
            expr: Some("search(\"nonexistent\")".into()),
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
            expr: Some("search(\"service OR report OR bug OR sprint\")".into()),
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
            expr: Some("search(\"Old\")".into()),
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
            expr: Some("search(\"Old\")".into()),
            ..Default::default()
        })
        .unwrap();
    assert!(views.is_empty());

    // New title should match
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("search(\"New\")".into()),
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
    // Only "Deploy service" matches: pending + tag:ops + due range + milestone "Deployment" + search
    let views = store
        .query_tasks(&TaskFilter {
            done: Some(false),
            expr: Some(
                "search(\"deploy\") && has_tag(\"ops\") && due >= \"2026-02-01\" \
                     && due <= \"2026-03-01\" && contains(milestone, \"Deployment\")"
                    .into(),
            ),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Deploy service");
}

// --- query_tasks: has() presence/absence filter via expr ---

#[test]
fn query_filter_with_due() {
    let mut store = test_store();
    seed_store(&mut store);
    // "Buy milk" has due=2026-03-01, "Urgent" has due=2026-01-15, "Done thing" has no due
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("has(due)".into()),
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
    store
        .upsert_tasks(file_id, &[t1, t2], &[vec![], vec![]])
        .unwrap();
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("has(rank)".into()),
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
            expr: Some("has(tag)".into()),
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
            expr: Some("has(due) && has(tag)".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Urgent");
}

#[test]
fn query_filter_without_due() {
    let mut store = test_store();
    seed_store(&mut store);
    // "Done thing" has no due
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("!has(due)".into()),
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
            expr: Some("!has(tag)".into()),
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
    store
        .upsert_tasks(file_id, &[t1, t2], &[vec![], vec![]])
        .unwrap();
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("!has(rank)".into()),
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
            expr: Some("!has(due) && !has(tag)".into()),
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
            expr: Some("has(due) && !has(tag)".into()),
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
        bindings: vec![(
            "note".to_string(),
            "string".to_string(),
            "\"hi\"".to_string(),
        )],
        dependencies: vec![],
    };

    let (tf, tasks, props, bindings) =
        super::super::to_store_records(&eval_result, Path::new("test.typ"), "hash", None);

    assert_eq!(tf.file_path, PathBuf::from("test.typ"));
    assert_eq!(tf.title, Some("Heading".to_string()));
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Test");
    assert_eq!(tasks[0].due, Some("2026-03-01".to_string()));
    assert_eq!(
        tasks[0].task_id,
        Some("019c5b9b-7317-77b1-bf52-ce7a298cfcad".to_string())
    );
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
    assert_eq!(
        PropertyKind::try_from_str("mindtape.due"),
        Some(PropertyKind::Due)
    );
    assert_eq!(
        PropertyKind::try_from_str("mindtape.start"),
        Some(PropertyKind::Start)
    );
    assert_eq!(
        PropertyKind::try_from_str("mindtape.rank"),
        Some(PropertyKind::Rank)
    );
    assert_eq!(
        PropertyKind::try_from_str("mindtape.tag"),
        Some(PropertyKind::Tag)
    );
    assert_eq!(
        PropertyKind::try_from_str("mindtape.id"),
        Some(PropertyKind::Id)
    );
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
    let file_id = store
        .upsert_task_file(&make_task_file("main.typ", "h1"))
        .unwrap();

    let refs = vec![
        PathBuf::from("lib/utils.typ"),
        PathBuf::from("lib/helpers.typ"),
    ];
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
    let file_id = store
        .upsert_task_file(&make_task_file("main.typ", "h1"))
        .unwrap();

    store
        .upsert_file_references(file_id, &[PathBuf::from("old.typ")])
        .unwrap();
    store
        .upsert_file_references(
            file_id,
            &[PathBuf::from("new1.typ"), PathBuf::from("new2.typ")],
        )
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
    let deps = store
        .get_file_dependencies(Path::new("unknown.typ"))
        .unwrap();
    assert!(deps.is_none());
}

#[test]
fn get_file_dependencies_returns_imports_and_imported_by() {
    let mut store = test_store();

    // Create files: main.typ imports lib/utils.typ
    let main_id = store
        .upsert_task_file(&make_task_file("main.typ", "h1"))
        .unwrap();
    let _utils_id = store
        .upsert_task_file(&make_task_file("lib/utils.typ", "h2"))
        .unwrap();

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

    let main_id = store
        .upsert_task_file(&make_task_file("main.typ", "h1"))
        .unwrap();
    let utils_id = store
        .upsert_task_file(&make_task_file("lib/utils.typ", "h2"))
        .unwrap();
    let _helper_id = store
        .upsert_task_file(&make_task_file("lib/helper.typ", "h3"))
        .unwrap();

    store
        .upsert_file_references(
            main_id,
            &[
                PathBuf::from("lib/utils.typ"),
                PathBuf::from("lib/helper.typ"),
            ],
        )
        .unwrap();
    store
        .upsert_file_references(utils_id, &[PathBuf::from("lib/helper.typ")])
        .unwrap();

    let all_deps = store.list_file_dependencies().unwrap();
    assert_eq!(all_deps.len(), 3);

    // main.typ: imports 2, imported by 0
    let main = all_deps
        .iter()
        .find(|d| d.file_path == Path::new("main.typ"))
        .unwrap();
    assert_eq!(main.imports.len(), 2);
    assert_eq!(main.imported_by.len(), 0);

    // lib/utils.typ: imports 1, imported by 1
    let utils = all_deps
        .iter()
        .find(|d| d.file_path == Path::new("lib/utils.typ"))
        .unwrap();
    assert_eq!(utils.imports.len(), 1);
    assert_eq!(utils.imported_by.len(), 1);

    // lib/helper.typ: imports 0, imported by 2
    let helper = all_deps
        .iter()
        .find(|d| d.file_path == Path::new("lib/helper.typ"))
        .unwrap();
    assert_eq!(helper.imports.len(), 0);
    assert_eq!(helper.imported_by.len(), 2);
}

#[test]
fn file_references_cascade_on_file_delete() {
    let mut store = test_store();
    let file_id = store
        .upsert_task_file(&make_task_file("main.typ", "h1"))
        .unwrap();

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

// --- query_tasks: --filter expression ---

#[test]
fn query_expr_due_lt() {
    let mut store = test_store();
    seed_store_rich(&mut store);
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("due < \"2026-02-01\"".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Fix login bug");
}

#[test]
fn query_expr_or_logic() {
    let mut store = test_store();
    seed_store_rich(&mut store);
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("has_tag(\"docs\") || has_tag(\"planning\")".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 2);
}

#[test]
fn query_expr_combined_with_flags() {
    let mut store = test_store();
    seed_store_rich(&mut store);
    let views = store
        .query_tasks(&TaskFilter {
            done: Some(false),
            expr: Some("has_tag(\"ops\")".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Deploy service");
}

#[test]
fn query_expr_search_fts() {
    let mut store = test_store();
    seed_store_rich(&mut store);
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some("search(\"deploy\")".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].title, "Deploy service");
}

#[test]
fn query_expr_has_or_tag() {
    let mut store = test_store();
    seed_store(&mut store);
    // "Buy milk" has due, "Urgent" has tag — both should match
    let views = store
        .query_tasks(&TaskFilter {
            done: Some(false),
            expr: Some("has(due) || has(tag)".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(views.len(), 2);
}

#[test]
fn query_expr_complex() {
    let mut store = test_store();
    seed_store_rich(&mut store);
    let views = store
        .query_tasks(&TaskFilter {
            expr: Some(
                "!done && (has_tag(\"ops\") || has_tag(\"planning\")) && due > \"2026-02-01\""
                    .into(),
            ),
            ..Default::default()
        })
        .unwrap();
    // "Deploy service" (ops, due 2026-02-15) and "Plan sprint" (planning, due 2026-04-01)
    assert_eq!(views.len(), 2);
}

#[test]
fn query_expr_invalid_returns_error() {
    let mut store = test_store();
    seed_store(&mut store);
    let result = store.query_tasks(&TaskFilter {
        expr: Some("((( bad".into()),
        ..Default::default()
    });
    assert!(result.is_err());
}

// --- file errors ---

#[test]
fn upsert_file_error_insert_and_list() {
    let mut store = test_store();
    store
        .upsert_file_error(
            Path::new("/a.typ"),
            Some(Path::new("/root")),
            "eval error: x",
        )
        .unwrap();
    let errors = store.list_file_errors().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].file_path, PathBuf::from("/a.typ"));
    assert_eq!(errors[0].watch_root, Some(PathBuf::from("/root")));
    assert_eq!(errors[0].error, "eval error: x");
}

#[test]
fn upsert_file_error_updates_on_same_path() {
    let mut store = test_store();
    store
        .upsert_file_error(Path::new("/a.typ"), None, "first error")
        .unwrap();
    store
        .upsert_file_error(Path::new("/a.typ"), None, "second error")
        .unwrap();
    let errors = store.list_file_errors().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error, "second error");
}

#[test]
fn remove_file_error_deletes() {
    let mut store = test_store();
    store
        .upsert_file_error(Path::new("/a.typ"), None, "err")
        .unwrap();
    store.remove_file_error(Path::new("/a.typ")).unwrap();
    let errors = store.list_file_errors().unwrap();
    assert!(errors.is_empty());
}

#[test]
fn remove_file_error_nonexistent_is_noop() {
    let mut store = test_store();
    store.remove_file_error(Path::new("/nope.typ")).unwrap();
}

#[test]
fn list_file_errors_empty() {
    let store = test_store();
    let errors = store.list_file_errors().unwrap();
    assert!(errors.is_empty());
}
