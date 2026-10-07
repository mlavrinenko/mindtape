//! The `<task>` argument every command that names one task shares, so the
//! concept has one name and one help text.

/// The help every `<task>` argument carries.
pub const HELP: &str = "The task: its stem, its hash, or the path to its file";

/// A task named on the command line; [`crate::util::resolve_task`] reads it.
#[derive(Debug, Clone, clap::Args)]
pub struct TaskRef {
    #[arg(help = HELP)]
    pub task: String,
}

impl From<&str> for TaskRef {
    fn from(task: &str) -> Self {
        Self {
            task: task.to_owned(),
        }
    }
}

impl From<String> for TaskRef {
    fn from(task: String) -> Self {
        Self { task }
    }
}
