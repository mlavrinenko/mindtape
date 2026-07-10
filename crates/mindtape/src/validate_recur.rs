//! Task-level `recur` policy check for `mt check`.
//!
//! A thin [`crate::validate::Violation`] adapter over
//! [`crate::recur::check_policy`]'s pure problem strings — the same split as
//! [`crate::validate_links`] over the link resolution/cardinality checks.

use crate::config::Config;
use crate::model::Task;
use crate::validate::{Violation, push};

/// Push one [`Violation`] per policy problem in `task`'s `recur` field (a
/// no-op when the task does not recur).
pub(crate) fn check_all(task: &Task, config: &Config, out: &mut Vec<Violation>) {
    let Some(recur) = &task.recur else { return };
    for problem in crate::recur::check_policy(recur, &config.tasks.recur) {
        push(out, task, "recur", problem);
    }
}

#[cfg(test)]
#[path = "validate_recur_tests.rs"]
mod tests;
