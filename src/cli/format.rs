use crate::store::{FileView, IndexStats, TaskView};

#[must_use]
pub fn format_task_view(task: &TaskView) -> String {
    let check = if task.is_done { "[x]" } else { "[ ]" };
    let due_part = task
        .due
        .as_ref()
        .map(|d| format!(" (due {d})"))
        .unwrap_or_default();
    let tag_part = if task.tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", task.tags.join(", "))
    };
    format!("- {check}{due_part} {}{tag_part}", task.title)
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
    format!(
        "{} ({count} {noun}){title_part}",
        file.relative_path.display()
    )
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

pub fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[must_use]
pub fn format_tasks_csv(tasks: &[TaskView]) -> String {
    let mut out = String::from("status,due,title,file,tags\n");
    for t in tasks {
        let status = if t.is_done { "done" } else { "pending" };
        let due = t.due.as_deref().unwrap_or("");
        let tags = t.tags.join(";");
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            status,
            due,
            csv_escape(&t.title),
            csv_escape(&t.file_path.to_string_lossy()),
            csv_escape(&tags),
        ));
    }
    out
}

#[must_use]
pub fn format_files_csv(files: &[FileView]) -> String {
    let mut out = String::from("path,title,tasks,updated_at\n");
    for f in files {
        let title = f.title.as_deref().unwrap_or("");
        out.push_str(&format!(
            "{},{},{},{}\n",
            csv_escape(&f.relative_path.to_string_lossy()),
            csv_escape(title),
            f.task_count,
            csv_escape(&f.updated_at),
        ));
    }
    out
}

#[must_use]
pub fn format_stats_csv(stats: &IndexStats) -> String {
    let mut out = String::from("metric,value\n");
    out.push_str(&format!("files,{}\n", stats.file_count));
    out.push_str(&format!("tasks,{}\n", stats.task_count));
    out.push_str(&format!("done,{}\n", stats.done_count));
    out.push_str(&format!("pending,{}\n", stats.pending_count));
    if let Some(ref ts) = stats.last_updated {
        out.push_str(&format!("last_updated,{}\n", csv_escape(ts)));
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // --- format_task_view ---

    #[test]
    fn format_task_view_pending_with_due() {
        let task = TaskView {
            title: "Buy milk".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: Some("2026-03-01".to_string()),
            tags: vec![],
        };
        assert_eq!(format_task_view(&task), "- [ ] (due 2026-03-01) Buy milk");
    }

    #[test]
    fn format_task_view_done_no_due() {
        let task = TaskView {
            title: "Done thing".to_string(),
            is_done: true,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: None,
            tags: vec![],
        };
        assert_eq!(format_task_view(&task), "- [x] Done thing");
    }

    #[test]
    fn format_task_view_with_tags() {
        let task = TaskView {
            title: "Task".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("t.typ"),
            file_title: None,
            due: None,
            tags: vec!["work".to_string(), "urgent".to_string()],
        };
        assert_eq!(format_task_view(&task), "- [ ] Task [work, urgent]");
    }

    // --- format_file_view ---

    #[test]
    fn format_file_view_with_title() {
        let file = FileView {
            relative_path: PathBuf::from("notes/todo.typ"),
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
            relative_path: PathBuf::from("t.typ"),
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
            tags: vec!["shop".to_string()],
        }];
        let csv = format_tasks_csv(&tasks);
        assert!(csv.starts_with("status,due,title,file,tags\n"));
        assert!(csv.contains("pending,2026-03-01,Buy milk,todo.typ,shop\n"));
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
            tags: vec![],
        }];
        let csv = format_tasks_csv(&tasks);
        assert!(csv.contains("done,,\"Buy eggs, milk\",t.typ,\n"));
    }

    #[test]
    fn format_files_csv_basic() {
        let files = vec![FileView {
            relative_path: PathBuf::from("notes/todo.typ"),
            title: Some("My Tasks".to_string()),
            task_count: 5,
            updated_at: "2026-01-01".to_string(),
        }];
        let csv = format_files_csv(&files);
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
        let csv = format_stats_csv(&stats);
        assert!(csv.starts_with("metric,value\n"));
        assert!(csv.contains("files,3\n"));
        assert!(csv.contains("tasks,10\n"));
        assert!(csv.contains("done,4\n"));
        assert!(csv.contains("pending,6\n"));
        assert!(csv.contains("last_updated,2026-01-15\n"));
    }

    #[test]
    fn csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn csv_escape_with_comma() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
    }

    #[test]
    fn csv_escape_with_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}
