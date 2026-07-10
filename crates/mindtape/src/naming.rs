//! Task stem naming: the `[tasks.naming]` template and the shared minter.
//!
//! Scaffold for the configurable-task-stem-naming task. The pure macro expander
//! (`{slug}`, `{date}`/`{date:FMT}`, `{yy}`/`{mm}`/`{dd}`, `{n}`/`{n:0N}`) and
//! the collision disambiguation land under that task. One shared minter here is
//! called by both the `mt add` write path and (later) the `mt recur` spawn path,
//! so the two never diverge. No clock reads inside the expander — the caller
//! passes the reference date.

use serde::Deserialize;

/// `[tasks.naming]` — how `mt add`/`mt recur` mint a new task's filename stem.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NamingConfig {
    /// Stem template with macros. Unset ⇒ bare `{slug}` (today's behaviour).
    pub stem: Option<String>,
}
