use std::path::Path;

use anyhow::{Context, Result, bail};
use typst::foundations::Datetime;

use crate::eval::{self, format_date, Task};
use crate::world;

/// Run the eval command (backwards compat: `mindtape file.typ`).
///
/// # Errors
/// Returns error if file doesn't exist, world creation fails, or evaluation fails.
pub fn run(file: &Path, due: bool, limit: Option<usize>) -> Result<()> {
    if !file.exists() {
        bail!("file not found: {}", file.display());
    }

    let world = world::MindTapeWorld::new(file)
        .with_context(|| format!("failed to create world for {}", file.display()))?;

    let tasks = eval::eval_file(&world)
        .with_context(|| format!("failed to evaluate {}", file.display()))?;

    let tasks = filter_and_sort(tasks, due, limit);

    for task in &tasks {
        println!("{}", format_task(task));
    }
    Ok(())
}

/// # Panics
/// Panics if `due_only` is true and a retained task has a `None` due date.
#[must_use]
pub fn filter_and_sort(tasks: Vec<Task>, due_only: bool, limit: Option<usize>) -> Vec<Task> {
    let mut tasks: Vec<_> = tasks.into_iter().filter(|t| !t.done).collect();

    if due_only {
        tasks.retain(|t| t.due.is_some());
        #[allow(clippy::unwrap_used)]
        tasks.sort_by(|a, b| {
            let a_due = a.due.as_ref().unwrap();
            let b_due = b.due.as_ref().unwrap();
            due_sort_key(a_due).cmp(&due_sort_key(b_due))
        });
    }

    if let Some(n) = limit {
        tasks.truncate(n);
    }

    tasks
}

#[must_use]
pub fn format_task(task: &Task) -> String {
    match &task.due {
        Some(dt) => format!("- (due {}) {}", format_due(dt), task.title),
        None => format!("- {}", task.title),
    }
}

#[must_use]
pub fn format_due(dt: &Datetime) -> String {
    format_date(dt)
}

#[must_use]
pub fn due_sort_key(dt: &Datetime) -> (i32, u8, u8) {
    (
        dt.year().unwrap_or(0),
        dt.month().unwrap_or(0),
        dt.day().unwrap_or(0),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn make_task(title: &str, done: bool, due: Option<Datetime>) -> Task {
        Task {
            title: title.to_string(),
            done,
            due,
            tags: vec![],
            id: None,
            position: 0,
            milestone: None,
        }
    }

    fn ymd(year: i32, month: u8, day: u8) -> Datetime {
        Datetime::from_ymd(year, month, day).unwrap()
    }

    #[test]
    fn format_due_normal() {
        assert_eq!(format_due(&ymd(2026, 3, 15)), "2026-03-15");
    }

    #[test]
    fn format_due_zero_padded() {
        assert_eq!(format_due(&ymd(2026, 1, 5)), "2026-01-05");
    }

    #[test]
    fn due_sort_key_extracts_components() {
        assert_eq!(due_sort_key(&ymd(2026, 3, 1)), (2026, 3, 1));
    }

    #[test]
    fn format_task_with_due() {
        let task = make_task("Buy milk", false, Some(ymd(2026, 3, 1)));
        assert_eq!(format_task(&task), "- (due 2026-03-01) Buy milk");
    }

    #[test]
    fn format_task_without_due() {
        let task = make_task("Buy milk", false, None);
        assert_eq!(format_task(&task), "- Buy milk");
    }

    #[test]
    fn filter_and_sort_removes_done() {
        let tasks = vec![
            make_task("a", false, None),
            make_task("b", true, None),
            make_task("c", false, None),
        ];
        let result = filter_and_sort(tasks, false, None);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "a");
        assert_eq!(result[1].title, "c");
    }

    #[test]
    fn filter_and_sort_due_only() {
        let tasks = vec![
            make_task("a", false, Some(ymd(2026, 3, 1))),
            make_task("b", false, None),
            make_task("c", false, Some(ymd(2026, 1, 1))),
        ];
        let result = filter_and_sort(tasks, true, None);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "c");
        assert_eq!(result[1].title, "a");
    }

    #[test]
    fn filter_and_sort_sorts_by_due() {
        let tasks = vec![
            make_task("later", false, Some(ymd(2026, 6, 1))),
            make_task("sooner", false, Some(ymd(2026, 2, 1))),
            make_task("middle", false, Some(ymd(2026, 4, 1))),
        ];
        let result = filter_and_sort(tasks, true, None);
        assert_eq!(result[0].title, "sooner");
        assert_eq!(result[1].title, "middle");
        assert_eq!(result[2].title, "later");
    }

    #[test]
    fn filter_and_sort_with_limit() {
        let tasks = vec![
            make_task("a", false, None),
            make_task("b", false, None),
            make_task("c", false, None),
        ];
        let result = filter_and_sort(tasks, false, Some(1));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "a");
    }

    #[test]
    fn filter_and_sort_limit_none_returns_all() {
        let tasks = vec![make_task("a", false, None), make_task("b", false, None)];
        let result = filter_and_sort(tasks, false, None);
        assert_eq!(result.len(), 2);
    }
}
