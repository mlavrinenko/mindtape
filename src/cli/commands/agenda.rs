use clap::Parser;

use crate::cli::format::{csv_escape, format_task_view};
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

#[must_use]
pub fn format_agenda_csv(agenda: &AgendaView) -> String {
    let mut csv = String::from("section,done,due,title,tags,file\n");

    for task in &agenda.overdue {
        csv.push_str(&format!(
            "overdue,{},{},{},{},{}\n",
            task.is_done,
            task.due.as_deref().unwrap_or(""),
            csv_escape(&task.title),
            task.tags.join(";"),
            task.file_path.display(),
        ));
    }

    for task in &agenda.today {
        csv.push_str(&format!(
            "today,{},{},{},{},{}\n",
            task.is_done,
            task.due.as_deref().unwrap_or(""),
            csv_escape(&task.title),
            task.tags.join(";"),
            task.file_path.display(),
        ));
    }

    for task in &agenda.this_week {
        csv.push_str(&format!(
            "this_week,{},{},{},{},{}\n",
            task.is_done,
            task.due.as_deref().unwrap_or(""),
            csv_escape(&task.title),
            task.tags.join(";"),
            task.file_path.display(),
        ));
    }

    csv
}
