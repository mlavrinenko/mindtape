//! `[[on.rm]]` binds actions to a task's removal.
//!
//! `mt rm` deletes a task file, so a cleanup, a message or a check before the
//! file goes has nowhere else to go. `[[on.rm]]` takes `[[on.flip]]`'s action
//! shape ([`FlipAction`]) and its `when` filter, minus `into(...)`: a removal
//! stamps no status.
//!
//! ```toml
//! [[on.rm]]
//! when = 'has_tag("chore")'
//! actions = [{ run = "just check", gate = true }]
//! ```
//!
//! `[[on.rm]]` entries append across packs and layers, a later entry named
//! like an earlier one replaces it in place, and `disable` drops one by name
//! ([`crate::config::packs`]), as for `[[on.flip]]`.
//!
//! Only `mt rm` runs these actions. `mt check`, `mt ls` and every other read
//! command run none. [`crate::shell`] runs the commands and states the trust
//! boundary.
//!
//! This module holds the shape and the selection (which hooks fire for a given
//! removal). Running the actions is the CLI's job.

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::filter::Filter;
use crate::graph::ready::Readiness;
use crate::model::Task;

use super::{FlipAction, HookError, HookFields, matching};

/// One `[[on.rm]]` entry: which removals it fires on, and what it does.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct RmHook {
    /// Optional name. A later entry with the same name, from a nearer layer
    /// or a pack, replaces an earlier one in place, and `disable` drops it by
    /// name. An unnamed entry always appends and is never replaced. Also the
    /// label `mt config show --origin` prints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Filter over the task the removal acts on. Empty fires on every removal.
    #[schemars(extend("x-expression" = "filter"))]
    pub when: String,
    /// Actions to run, in declaration order.
    pub actions: Vec<FlipAction>,
}

super::impl_hook_fields!(RmHook);

/// The `[[on.rm]]` hooks that fire when `mt rm` removes `task`, in declaration
/// order.
///
/// `task` is the artifact as it stands before the removal. The caller computes
/// `readiness` over the whole loaded set (see [`Readiness::compute`]) for the
/// `ready` / `blocked` predicates.
///
/// # Errors
/// Returns [`HookError`] when a `when` expression fails to parse or names an
/// unknown field or view.
pub fn matching_rm_hooks<'a>(
    config: &'a Config,
    task: &Task,
    readiness: &Readiness,
) -> Result<Vec<&'a RmHook>, HookError> {
    matching(&config.on.rm, "on.rm", |hook| {
        Filter::parse(&hook.when)?.matches_with(task, config, readiness)
    })
}
