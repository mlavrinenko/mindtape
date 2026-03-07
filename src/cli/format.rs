use crate::store::{FileView, IndexStats, TaskView};
use csv::Writer;

/// Finish a CSV writer and return its contents as a `String`.
///
/// # Errors
/// Returns error if flushing or UTF-8 conversion fails.
pub fn csv_to_string(wtr: Writer<Vec<u8>>) -> Result<String, csv::Error> {
    let bytes = wtr
        .into_inner()
        .map_err(|e| csv::Error::from(std::io::Error::other(e)))?;
    String::from_utf8(bytes)
        .map_err(|e| csv::Error::from(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
}

/// Format a task as a valid Typst list item with inline tags.
///
/// Output: `- [ ] Title #due(2026, 5, 15) #start(2026, 3, 1) #high #tag("t") #id("uuid")`
#[must_use]
pub fn format_task_typst(task: &TaskView) -> String {
    let check = if task.is_done { "[x]" } else { "[ ]" };
    let mut parts = vec![format!("- {check} {}", task.title)];
    if let Some(ref date) = task.due
        && let Some(call) = date_to_typst_call("due", date)
    {
        parts.push(call);
    }
    if let Some(ref date) = task.start
        && let Some(call) = date_to_typst_call("start", date)
    {
        parts.push(call);
    }
    if let Some(r) = task.rank {
        parts.push(rank_to_typst(r));
    }
    for tag in &task.tags {
        parts.push(format!("#tag(\"{tag}\")"));
    }
    if let Some(ref id) = task.task_id {
        parts.push(format!("#id(\"{id}\")"));
    }
    parts.join(" ")
}

/// Convert a `YYYY-MM-DD` date string to `#func(Y, M, D)`.
fn date_to_typst_call(func: &str, date: &str) -> Option<String> {
    let parts: Vec<&str> = date.split('-').collect();
    let year = parts.first()?.trim_start_matches('0');
    let month = parts.get(1)?.trim_start_matches('0');
    let day = parts.get(2)?.trim_start_matches('0');
    Some(format!("#{func}({year}, {month}, {day})"))
}

/// Convert a numeric rank to its Typst representation.
fn rank_to_typst(rank: i64) -> String {
    match rank {
        100 => "#high".to_string(),
        50 => "#medium".to_string(),
        10 => "#low".to_string(),
        other => format!("#rank({other})"),
    }
}

#[must_use]
pub fn format_file_view(file: &FileView) -> String {
    let title_part = file
        .title
        .as_ref()
        .map(|t| format!("  {t}"))
        .unwrap_or_default();
    let count = file.task_count;
    let noun = if count == 1 { "task" } else { "tasks" };
    format!("{} ({count} {noun}){title_part}", file.file_path.display())
}

#[must_use]
pub fn format_stats(stats: &IndexStats) -> String {
    let mut lines = vec![
        format!("files:   {}", stats.file_count),
        format!(
            "tasks:   {} ({} pending, {} done)",
            stats.task_count, stats.pending_count, stats.done_count
        ),
    ];
    if let Some(ref ts) = stats.last_updated {
        lines.push(format!("updated: {ts}"));
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// CSV output
// ---------------------------------------------------------------------------

/// Formats tasks as CSV
///
/// # Errors
/// Returns error if CSV writing fails (unlikely with in-memory writer)
pub fn format_tasks_csv(tasks: &[TaskView]) -> Result<String, csv::Error> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(["status", "due", "start", "rank", "title", "file", "tags"])?;
    for t in tasks {
        let status = if t.is_done { "done" } else { "pending" };
        let due = t.due.as_deref().unwrap_or("");
        let start = t.start.as_deref().unwrap_or("");
        let rank_str = t.rank.map(|r| r.to_string()).unwrap_or_default();
        let tags = t.tags.join(";");
        wtr.write_record([
            status,
            due,
            start,
            &rank_str,
            &t.title,
            &t.file_path.to_string_lossy(),
            &tags,
        ])?;
    }
    csv_to_string(wtr)
}

/// Formats files as CSV
///
/// # Errors
/// Returns error if CSV writing fails (unlikely with in-memory writer)
pub fn format_files_csv(files: &[FileView]) -> Result<String, csv::Error> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(["path", "title", "tasks", "updated_at"])?;
    for f in files {
        let title = f.title.as_deref().unwrap_or("");
        wtr.write_record([
            f.file_path.to_string_lossy().as_ref(),
            title,
            &f.task_count.to_string(),
            &f.updated_at,
        ])?;
    }
    csv_to_string(wtr)
}

/// Formats stats as CSV
///
/// # Errors
/// Returns error if CSV writing fails (unlikely with in-memory writer)
pub fn format_stats_csv(stats: &IndexStats) -> Result<String, csv::Error> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(["metric", "value"])?;
    wtr.write_record(["files", &stats.file_count.to_string()])?;
    wtr.write_record(["tasks", &stats.task_count.to_string()])?;
    wtr.write_record(["done", &stats.done_count.to_string()])?;
    wtr.write_record(["pending", &stats.pending_count.to_string()])?;
    if let Some(ref ts) = stats.last_updated {
        wtr.write_record(["last_updated", ts])?;
    }
    csv_to_string(wtr)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // --- format_file_view ---

    #[test]
    fn format_file_view_with_title() {
        let file = FileView {
            file_path: PathBuf::from("notes/todo.typ"),
            watch_root: None,
            title: Some("My Tasks".to_string()),
            task_count: 5,
            updated_at: "2026-01-01".to_string(),
        };
        assert_eq!(
            format_file_view(&file),
            "notes/todo.typ (5 tasks)  My Tasks"
        );
    }

    #[test]
    fn format_file_view_no_title_singular() {
        let file = FileView {
            file_path: PathBuf::from("t.typ"),
            watch_root: None,
            title: None,
            task_count: 1,
            updated_at: "2026-01-01".to_string(),
        };
        assert_eq!(format_file_view(&file), "t.typ (1 task)");
    }

    // --- format_stats ---

    #[test]
    fn format_stats_with_data() {
        let stats = IndexStats {
            file_count: 3,
            task_count: 10,
            done_count: 4,
            pending_count: 6,
            last_updated: Some("2026-01-15 12:00:00".to_string()),
        };
        let output = format_stats(&stats);
        assert!(output.contains("files:   3"));
        assert!(output.contains("tasks:   10 (6 pending, 4 done)"));
        assert!(output.contains("updated: 2026-01-15 12:00:00"));
    }

    #[test]
    fn format_stats_empty() {
        let stats = IndexStats {
            file_count: 0,
            task_count: 0,
            done_count: 0,
            pending_count: 0,
            last_updated: None,
        };
        let output = format_stats(&stats);
        assert!(output.contains("files:   0"));
        assert!(!output.contains("updated:"));
    }

    // --- CSV formatters ---

    #[test]
    fn format_tasks_csv_basic() {
        let tasks = vec![TaskView {
            title: "Buy milk".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: Some("2026-03-01".to_string()),
            start: None,
            rank: None,
            task_id: None,
            tags: vec!["shop".to_string()],
            milestone: None,
            watch_root: None,
        }];
        let csv = format_tasks_csv(&tasks).unwrap();
        assert!(csv.starts_with("status,due,start,rank,title,file,tags\n"));
        assert!(csv.contains("pending,2026-03-01,,,Buy milk,todo.typ,shop\n"));
    }

    #[test]
    fn format_tasks_csv_escapes_commas() {
        let tasks = vec![TaskView {
            title: "Buy eggs, milk".to_string(),
            is_done: true,
            position: 0,
            file_path: PathBuf::from("t.typ"),
            file_title: None,
            due: None,
            start: None,
            rank: None,
            task_id: None,
            tags: vec![],
            milestone: None,
            watch_root: None,
        }];
        let csv = format_tasks_csv(&tasks).unwrap();
        assert!(csv.contains("done,,,,\"Buy eggs, milk\",t.typ,\n"));
    }

    #[test]
    fn format_files_csv_basic() {
        let files = vec![FileView {
            file_path: PathBuf::from("notes/todo.typ"),
            watch_root: None,
            title: Some("My Tasks".to_string()),
            task_count: 5,
            updated_at: "2026-01-01".to_string(),
        }];
        let csv = format_files_csv(&files).unwrap();
        assert!(csv.starts_with("path,title,tasks,updated_at\n"));
        assert!(csv.contains("notes/todo.typ,My Tasks,5,2026-01-01\n"));
    }

    #[test]
    fn format_stats_csv_basic() {
        let stats = IndexStats {
            file_count: 3,
            task_count: 10,
            done_count: 4,
            pending_count: 6,
            last_updated: Some("2026-01-15".to_string()),
        };
        let csv = format_stats_csv(&stats).unwrap();
        assert!(csv.starts_with("metric,value\n"));
        assert!(csv.contains("files,3\n"));
        assert!(csv.contains("tasks,10\n"));
        assert!(csv.contains("done,4\n"));
        assert!(csv.contains("pending,6\n"));
        assert!(csv.contains("last_updated,2026-01-15\n"));
    }

    // --- format_task_typst ---

    #[test]
    fn typst_pending_with_due() {
        let task = TaskView {
            title: "Buy milk".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: Some("2026-03-01".to_string()),
            start: None,
            rank: None,
            task_id: None,
            tags: vec![],
            milestone: None,
            watch_root: None,
        };
        assert_eq!(format_task_typst(&task), "- [ ] Buy milk #due(2026, 3, 1)");
    }

    #[test]
    fn typst_done_no_metadata() {
        let task = TaskView {
            title: "Done thing".to_string(),
            is_done: true,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: None,
            start: None,
            rank: None,
            task_id: None,
            tags: vec![],
            milestone: None,
            watch_root: None,
        };
        assert_eq!(format_task_typst(&task), "- [x] Done thing");
    }

    #[test]
    fn typst_rank_aliases() {
        let make = |rank| TaskView {
            title: "T".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("t.typ"),
            file_title: None,
            due: None,
            start: None,
            rank: Some(rank),
            task_id: None,
            tags: vec![],
            milestone: None,
            watch_root: None,
        };
        assert_eq!(format_task_typst(&make(100)), "- [ ] T #high");
        assert_eq!(format_task_typst(&make(50)), "- [ ] T #medium");
        assert_eq!(format_task_typst(&make(10)), "- [ ] T #low");
        assert_eq!(format_task_typst(&make(75)), "- [ ] T #rank(75)");
    }

    #[test]
    fn typst_full_metadata() {
        let task = TaskView {
            title: "Full task".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("t.typ"),
            file_title: None,
            due: Some("2026-04-01".to_string()),
            start: Some("2026-03-01".to_string()),
            rank: Some(100),
            task_id: Some("abc-123".to_string()),
            tags: vec!["work".to_string()],
            milestone: None,
            watch_root: None,
        };
        assert_eq!(
            format_task_typst(&task),
            "- [ ] Full task #due(2026, 4, 1) #start(2026, 3, 1) #high #tag(\"work\") #id(\"abc-123\")"
        );
    }

    // --- date_to_typst_call ---

    #[test]
    fn date_to_typst_strips_leading_zeros() {
        assert_eq!(
            date_to_typst_call("due", "2026-05-09").unwrap(),
            "#due(2026, 5, 9)"
        );
    }

    #[test]
    fn date_to_typst_no_leading_zeros() {
        assert_eq!(
            date_to_typst_call("start", "2026-12-25").unwrap(),
            "#start(2026, 12, 25)"
        );
    }

    #[test]
    fn date_to_typst_invalid_returns_none() {
        assert!(date_to_typst_call("due", "invalid").is_none());
        assert!(date_to_typst_call("due", "2026-03").is_none());
    }
}
