//! `[go]` config: the editor `mt go` opens a task file in.
//!
//! Shape only — the spawn machinery (placeholder substitution, resolution
//! ladder) lives in the CLI, keeping the library format- and process-blind.

use serde::Deserialize;

/// `[go]` — settings for `mt go`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct GoConfig {
    /// Editor command. A `{task}` token is replaced with the resolved file path;
    /// without it the path is appended. Falls back to `$VISUAL`, then `$EDITOR`;
    /// `go` errors when none is set (no `vi` floor — see the `go` command docs).
    /// May carry arguments (split on whitespace).
    pub editor: Option<String>,
}
