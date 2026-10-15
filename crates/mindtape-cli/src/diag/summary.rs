//! The last line of a command that found several problems, and the verdict of
//! a command it ran: counts and exit statuses, worded once.
//!
//! ```text
//! error: 3 problems in 2 tasks
//! error: 1 commit message problem
//! error: 2 commits broke commit rules
//! error: gate failed: 'scripts/test': exit 1
//! ```

use mindtape::count;

use super::Diag;

impl Diag {
    /// What `mt check` ends with: `3 problems in 2 tasks`.
    pub(crate) fn problems(problems: usize, tasks: usize) -> Self {
        Self::refused("").reason(format!(
            "{} in {}",
            count(problems, "problem"),
            count(tasks, "task")
        ))
    }

    /// What `mt check --commit-msg` ends with: `2 commit message problems`.
    pub(crate) fn commit_message_problems(problems: usize) -> Self {
        Self::refused("").reason(count(problems, "commit message problem"))
    }

    /// What `mt check --commits` ends with: `2 commits broke commit rules`,
    /// or `1 commit and 1 task broke commit rules` when a `scope = "tip"` rule
    /// also fired.
    pub(crate) fn commits_broke_rules(commits: usize, tasks: usize) -> Self {
        let broke = [(commits, "commit"), (tasks, "task")]
            .into_iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, noun)| count(n, noun))
            .collect::<Vec<_>>()
            .join(" and ");
        Self::refused("").reason(format!("{broke} broke commit rules"))
    }

    /// How a command we ran ended: `exit 1`, or `terminated by signal` when
    /// it left no code.
    pub(crate) fn exit(self, code: Option<i32>) -> Self {
        self.reason(code.map_or_else(
            || "terminated by signal".to_owned(),
            |code| format!("exit {code}"),
        ))
    }
}

/// Keep each note once across the problems of one batch: a breach raised on
/// another task by several targets is context for the batch, not for each.
pub(crate) fn dedupe_notes(problems: &mut [Diag]) {
    let mut seen = std::collections::BTreeSet::new();
    for problem in problems {
        problem.0.notes.retain(|note| seen.insert(note.to_string()));
    }
}

#[cfg(test)]
#[path = "summary_tests.rs"]
mod tests;
