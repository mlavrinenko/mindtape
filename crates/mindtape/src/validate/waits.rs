//! The `waits-on` entry check for `mt check`.
//!
//! A thin [`crate::validate::Violation`] adapter over
//! [`crate::waits::command_line`], the same split as
//! [`crate::validate::recur`] over the recurrence policy check. An entry that
//! fails here never runs: [`crate::waits::annotate`] answers it unknown.

use crate::config::Config;
use crate::model::Task;
use crate::validate::{Violation, violation};

/// Push one [`Violation`] per `waits-on` entry of `task` that cannot run: no
/// `checker:id` shape, an undeclared checker, or an id that could reach the
/// shell.
pub(crate) fn check_all(task: &Task, config: &Config, out: &mut Vec<Violation>) {
    for (index, entry) in task.waits_on.iter().enumerate() {
        if let Err(problem) = crate::waits::command_line(entry, config) {
            let field = format!("{}[{index}]", crate::waits::FIELD);
            let mut breach = violation(task, &field, problem.message);
            breach.unknown = problem.unknown;
            out.push(breach);
        }
    }
}

#[cfg(test)]
#[path = "waits_tests.rs"]
mod tests;
