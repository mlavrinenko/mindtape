//! `[go]` config: the editor `mt go` opens a task file in.
//!
//! Shape only — the spawn machinery (placeholder substitution, resolution
//! ladder) lives in the CLI, keeping the library format- and process-blind.

use serde::Deserialize;

/// Settings for `mt go`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct GoConfig {
    /// Editor command. `mt go` replaces a `{task}` token with the task's file
    /// path, or appends the path when there is no token. Falls back to
    /// `$VISUAL`, then `$EDITOR`; `mt go` fails when none is set. May carry
    /// arguments, split on whitespace.
    pub editor: Option<String>,
}
