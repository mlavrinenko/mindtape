//! `[tasks.assignee]` — the assignee field's optional constraints.
//!
//! An assignee is the actor currently working a task. It is orthogonal to
//! status: a task keeps the same assignee through proposed/accepted/wip/done,
//! and a blocked task is still assigned to whoever unblocks and finishes it. It
//! is not ownership — the worker is mutable and can ping-pong between actors,
//! and saying who works it now carries no statement about who else may help.
//!
//! The field is plain free text by default; `allowed` constrains it to a
//! project's declared actors, and `required` (a boolean or a filter expression,
//! like every other core field's `required`) forces a task to name one. There
//! is no `default`: an assignee is a runtime value, so a template that wants
//! one fills it through a slot (`env: "MINDTAPE_PRINCIPAL"`).

use serde::Deserialize;

use crate::config::Requirement;

/// `[tasks.assignee]` — who may be named, and whether one must be.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AssigneeConfig {
    /// Allowed assignees (the project's actors). Empty = accept any value.
    pub allowed: Vec<String>,
    /// Whether an assignee must be set.
    pub required: Requirement,
    /// A filter naming the tasks held to `allowed`; every other task keeps
    /// whatever it carries. Absent holds every task.
    pub values_when: Option<String>,
    /// The assignee `mt check` files a spliced slot with. A required assignee
    /// with no `example` leaves that slot unfillable.
    pub example: Option<String>,
}

#[cfg(test)]
#[path = "assignee_tests.rs"]
mod tests;
