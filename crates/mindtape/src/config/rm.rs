//! `[rm]` / `[retire]` config: task removal and retirement policy.
//!
//! `rm` deletes a task file outright; `retire` (its gitless fallback strategy)
//! keeps the file and flips its status terminal instead. [`RmStrategy`] picks
//! how inbound references to a removed task are handled — git-pinned SWHID
//! tombstone, in-place retire, edge unlinking, or an outright refusal.

use serde::Deserialize;

/// How `mt rm` handles links from other tasks to a removed task.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RmStrategy {
    /// Mint a git-pinned SWHID tombstone and rewrite inbound edges onto it (the default in a git repo).
    #[default]
    GitTombstone,
    /// Keep the task file, flip it to a terminal status — a tombstone without git.
    Retire,
    /// Strip each inbound edge from the surviving tasks, then delete.
    Unlink,
    /// Refuse, listing every surviving task that references a removed task.
    Reject,
}

/// `[rm]` — defaults for the `rm` task-removal command.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RmConfig {
    /// Filter selecting which tasks a bare `mt rm` (no target given) retires.
    /// Defaults to `"terminal"`, sweeping every task in a terminal status.
    /// An explicit ref or filter argument on the command line overrides it.
    pub default_filter: Option<String>,
    /// How inbound references to a removed task are handled; defaults to `"git-tombstone"` in a git repository, `"reject"` without one.
    pub strategy: Option<RmStrategy>,
}

/// `[retire]` — the gitless `rm` tombstone: keep the file, flip the status terminal.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RetireConfig {
    /// The terminal status the retired task flips to; defaults to the legend's first terminal status (an error if the legend declares none).
    pub status: Option<String>,
    /// Move the retired task file into a sibling folder of this name, such as
    /// `.archive`. Omitted keeps the file where it is.
    pub rename: Option<String>,
}
