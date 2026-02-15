use clap::Parser;
use csv::Writer;

use crate::cli::format::{csv_to_string, format_task_view};
use crate::store::AgendaView;

use crate::cli::QueryOpts;

/// Show agenda (overdue, today, this week).
#[derive(Parser, Debug)]
pub struct AgendaArgs {
    /// Show overdue tasks
    #[arg(long)]
    pub overdue: bool,

    /// Show tasks due today
    #[arg(long)]
    pub today: bool,

    /// Show tasks due this week
    #[arg(long)]
    pub week: bool,

    /// Limit output to N items
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    #[command(flatten)]
    pub query: QueryOpts,
}

impl AgendaArgs {
    /// If no specific sections were requested, show all.
    #[must_use]
    pub fn show_overdue(&self) -> bool {
        self.overdue || self.show_all()
    }

    /// If no specific sections were requested, show all.
    #[must_use]
    pub fn show_today(&self) -> bool {
        self.today || self.show_all()
    }

    /// If no specific sections were requested, show all.
    #[must_use]
    pub fn show_week(&self) -> bool {
        self.week || self.show_all()
    }

    fn show_all(&self) -> bool {
        !self.overdue && !self.today && !self.week
    }
}

#[must_use]
pub fn format_agenda(
    agenda: &AgendaView,
    show_overdue: bool,
    show_today: bool,
    show_week: bool,
) -> String {
    let mut output = String::new();

    if show_overdue && !agenda.overdue.is_empty() {
        output.push_str("# Overdue\n\n");
        for task in &agenda.overdue {
            output.push_str(&format!("  {}\n", format_task_view(task)));
        }
        output.push('\n');
    }

    if show_today && !agenda.today.is_empty() {
        output.push_str("# Today\n\n");
        for task in &agenda.today {
            output.push_str(&format!("  {}\n", format_task_view(task)));
        }
        output.push('\n');
    }

    if show_week && !agenda.this_week.is_empty() {
        output.push_str("# This Week\n\n");
        for task in &agenda.this_week {
            output.push_str(&format!("  {}\n", format_task_view(task)));
        }
        output.push('\n');
    }

    if output.is_empty() {
        output.push_str("No tasks in agenda.\n");
    }

    output
}

/// Formats agenda as CSV
///
/// # Errors
/// Returns error if CSV writing fails (unlikely with in-memory writer)
pub fn format_agenda_csv(agenda: &AgendaView) -> Result<String, csv::Error> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(["section", "done", "due", "title", "tags", "file"])?;

    let sections = [
        ("overdue", agenda.overdue.as_slice()),
        ("today", agenda.today.as_slice()),
        ("this_week", agenda.this_week.as_slice()),
    ];
    for (label, tasks) in &sections {
        for task in *tasks {
            wtr.write_record([
                *label,
                &task.is_done.to_string(),
                task.due.as_deref().unwrap_or(""),
                &task.title,
                &task.tags.join(";"),
                &task.file_path.to_string_lossy(),
            ])?;
        }
    }

    csv_to_string(wtr)
}
