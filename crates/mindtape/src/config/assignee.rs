//! `[tasks.assignee]`: the assignee field's optional constraints.
//!
//! An assignee is the actor currently working a task. It is orthogonal to
//! status: a task keeps the same assignee through proposed/accepted/wip/done,
//! and a blocked task is still assigned to whoever unblocks and finishes it. It
//! is not ownership. The worker is mutable and can ping-pong between actors, and
//! saying who works it carries no statement about who else may help.
//!
//! The field is plain free text by default. `allowed` constrains it to a
//! project's declared actors, and `required` (a boolean or a filter expression,
//! like every other core field's `required`) forces a task to name one. There
//! is no `default`: an assignee is a runtime value, and a template that needs
//! one fills it through a slot (`env: "MINDTAPE_PRINCIPAL"`).

use serde::{Deserialize, Serialize};

use crate::config::Requirement;

/// Who may hold the assignee role, and whether a task must carry one.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AssigneeConfig {
    /// Allowed assignees (the project's actors). Empty = accept any value.
    pub allowed: Vec<String>,
    /// Whether a task must carry an assignee.
    #[schemars(extend("x-expression" = "filter"))]
    pub required: Requirement,
    /// A filter that selects the tasks held to `allowed`. Every other task keeps
    /// whatever it carries. Absent holds every task.
    #[schemars(extend("x-expression" = "filter"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values_when: Option<String>,
    /// The assignee `mt check` files a spliced slot with. A required assignee
    /// with no `example` leaves that slot unfillable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

#[cfg(test)]
#[path = "assignee_tests.rs"]
mod tests;
