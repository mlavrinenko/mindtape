//! `mt review <id>` — stamp `reviewed:` with today's date.
//!
//! Resolves the task by slug, relative path, or absolute path; refuses terminal
//! statuses; writes the `reviewed:` slot via [`typst_edit`], idempotent on the
//! same day.
//!
//! The rewrite goes through [`crate::pre_write`] first, so a stamp that would
//! leave the task invalid is refused rather than landed and reported.

use crate::diag::Result;
use clap::Args;
use mindtape::{parse_ymd, today_iso};

use crate::diag::{Diag, Hint};
use crate::pre_write;
use crate::task_edit::rewrite_field;
use crate::util;

/// Arguments for `mt review`.
#[derive(Args)]
pub struct ReviewArgs {
    #[command(flatten)]
    pub task: crate::cli::task_ref::TaskRef,
}

/// Run the `review` command.
///
/// # Errors
/// Returns an error if the task cannot be resolved, is terminal, or the file
/// is wrong-shaped (no `task.with(…)` call found).
pub fn run(args: &ReviewArgs) -> Result<()> {
    let project = util::open_warned()?;

    let (task, abs) = util::resolve_task(&project, &args.task.task)?;

    if task.is_terminal(&project.config.tasks.status.terminal()) {
        return Err(Diag::refused("can't review a task with a terminal status")
            .value(&task.id)
            .hint(Hint::new("a finished task needs no review")));
    }

    let today = today_iso();
    if task.reviewed.as_deref() == Some(today.as_str()) {
        println!("unchanged: {} was already reviewed today", task.id);
        return Ok(());
    }

    let updated = rewrite_field(&abs, "reviewed", &on_date(&today)?)?;
    pre_write::write_checked(&project, &abs, updated)?;
    util::report_change("reviewed", &task, &today);
    Ok(())
}

/// `today`, a `YYYY-MM-DD` string, as the `on(y, m, d)` call `reviewed:` holds.
fn on_date(today: &str) -> Result<String> {
    let (year, month, day) = parse_ymd(today)
        .ok_or_else(|| Diag::failed("not a date in YYYY-MM-DD form").value(today))?;
    Ok(format!("on({year}, {month}, {day})"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{on_date, rewrite_field};

    /// `source` with today's `reviewed:` stamp written in, as `mt review` does.
    fn stamp(source: &str, today: &str) -> crate::diag::Result<String> {
        let dir = mindtape::test_support::tempdir();
        let file = dir.path().join("task.typ");
        std::fs::write(&file, source).unwrap();
        rewrite_field(&file, "reviewed", &on_date(today)?)
    }

    const TASK_SOURCE: &str = mindtape::typst_task!(
        r#"(
  title: "Do thing",
  status: proposed(2026, 6, 1),
  priority: "low",
)

== Body
"#
    );

    const TASK_WITH_REVIEWED: &str = mindtape::typst_task!(
        r#"(
  title: "Do thing",
  status: proposed(2026, 6, 1),
  priority: "low",
  reviewed: on(2026, 1, 1),
)

== Body
"#
    );

    #[test]
    fn stamps_reviewed_whether_absent_or_present() {
        for source in [TASK_SOURCE, TASK_WITH_REVIEWED] {
            let stamped = stamp(source, "2026-06-18").unwrap();
            assert!(stamped.contains("reviewed: on(2026, 6, 18)"), "{stamped}");
            assert!(!stamped.contains("on(2026, 1, 1)"), "{stamped}");
        }
        // An absent stamp lands after the last field.
        let inserted = stamp(TASK_SOURCE, "2026-06-18").unwrap();
        assert!(inserted.contains("priority: \"low\",\n  reviewed: on(2026, 6, 18),"));
    }

    #[test]
    fn errors_on_missing_task_call() {
        assert!(stamp("not a task file", "2026-06-18").is_err());
    }
}
