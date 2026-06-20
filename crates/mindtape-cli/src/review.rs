//! `mindtape review <id>` — stamp `reviewed:` with today's date.
//!
//! Resolves the task by slug, relative path, or absolute path; refuses terminal
//! statuses; writes the `reviewed:` slot via [`typst_edit`], idempotent on the
//! same day.

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{parse_ymd, today_iso};
use typst_edit::{Edit, apply};

use crate::task_edit::{task_with, upsert_named_arg};
use crate::util;

/// Arguments for `mindtape review`.
#[derive(Args)]
pub struct ReviewArgs {
    /// Task to stamp: slug, relative path, or absolute path.
    pub id: String,
}

/// Run the `review` command.
///
/// # Errors
/// Returns an error if the task cannot be resolved, is terminal, or the file
/// is wrong-shaped (no `task.with(…)` call found).
pub fn run(args: &ReviewArgs) -> Result<()> {
    let project = util::open()?;
    util::warn_errors(&project.outcome);

    let (task, abs) = util::resolve_task(&project, &args.id)?;

    if task.is_terminal(&project.config.tasks.status.terminal()) {
        bail!(
            "task {:?} ({}) has a terminal status; cannot review",
            args.id,
            task.title
        );
    }

    let today = today_iso();
    if task.reviewed.as_deref() == Some(today.as_str()) {
        println!("already reviewed today ({} — {})", args.id, task.title);
        return Ok(());
    }

    let source =
        std::fs::read_to_string(&abs).with_context(|| format!("read {}", abs.display()))?;

    let edit = build_edit(&source, &today)
        .with_context(|| format!("locate task.with(…) in {}", abs.display()))?;

    let updated =
        apply(&source, vec![edit]).with_context(|| format!("apply edit to {}", abs.display()))?;

    std::fs::write(&abs, updated).with_context(|| format!("write {}", abs.display()))?;
    println!(
        "reviewed: {} ({}) stamped on {}",
        args.id, task.title, today
    );
    Ok(())
}

/// Build the edit that upserts `reviewed: on(y, m, d)` in the `task.with(…)` call.
///
/// `today` must be a `YYYY-MM-DD` string; it is parsed to produce the `on(…)` call.
fn build_edit(source: &str, today: &str) -> Result<Edit> {
    let call = task_with(source)?;
    let (year, month, day) = parse_ymd(today).with_context(|| format!("invalid date {today:?}"))?;
    let value_text = format!("on({year}, {month}, {day})");
    upsert_named_arg(source, &call, "reviewed", &value_text)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::build_edit;
    use typst_edit::apply;

    const TASK_SOURCE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Do thing",
  status: proposed(2026, 6, 1),
  priority: "low",
)

== Body
"#;

    const TASK_WITH_REVIEWED: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Do thing",
  status: proposed(2026, 6, 1),
  priority: "low",
  reviewed: on(2026, 1, 1),
)

== Body
"#;

    #[test]
    fn inserts_reviewed_when_absent() {
        let today = "2026-06-18";
        let edit = build_edit(TASK_SOURCE, today).unwrap();
        let result = apply(TASK_SOURCE, vec![edit]).unwrap();
        assert!(result.contains("reviewed: on(2026, 6, 18),"));
        assert!(result.contains("priority: \"low\",\n  reviewed:"));
    }

    #[test]
    fn updates_reviewed_when_present() {
        let today = "2026-06-18";
        let edit = build_edit(TASK_WITH_REVIEWED, today).unwrap();
        let result = apply(TASK_WITH_REVIEWED, vec![edit]).unwrap();
        assert!(result.contains("reviewed: on(2026, 6, 18)"));
        assert!(!result.contains("on(2026, 1, 1)"));
    }

    #[test]
    fn errors_on_missing_task_call() {
        assert!(build_edit("not a task file", "2026-06-18").is_err());
    }
}
