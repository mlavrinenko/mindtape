//! `[rm]` / `[retire]` config: task removal and retirement policy.
//!
//! `rm` deletes a task file outright. `retire` (its gitless fallback strategy)
//! keeps the file and flips its status terminal instead. [`RmStrategy`] picks
//! how the code handles inbound references to a removed task: git-pinned SWHID
//! tombstone, in-place retire, edge unlinking, or an outright refusal.

use serde::{Deserialize, Serialize};

/// How `mt rm` handles links from other tasks to a removed task.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum RmStrategy {
    /// Mint a git-pinned SWHID tombstone and rewrite inbound edges onto it (the
    /// default in a git repo).
    #[default]
    GitTombstone,
    /// Keep the task file and flip it to a terminal status. This is a tombstone
    /// without git.
    Retire,
    /// Strip each inbound edge from the surviving tasks, then delete.
    Unlink,
    /// Refuse, listing every surviving task that references a removed task.
    Reject,
}

/// Defaults for the `rm` task-removal command.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RmConfig {
    /// Filter selecting which tasks a bare `mt rm` (no target given) retires.
    /// Defaults to `"terminal"`, sweeping every task in a terminal status.
    /// An explicit ref or filter argument on the command line overrides it.
    #[schemars(extend("x-expression" = "filter"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_filter: Option<String>,
    /// How the code handles inbound references to a removed task. Defaults to
    /// `"git-tombstone"` in a git repository, `"reject"` without one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<RmStrategy>,
}

/// The gitless `rm` tombstone. It keeps the file and flips its status to a
/// terminal one.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RetireConfig {
    /// The terminal status the retired task flips to. Defaults to the legend's
    /// first terminal status (an error if the legend declares none).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Move the retired task file into a sibling folder of this name, such as
    /// `.archive`. Omitted keeps the file where it is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
}
