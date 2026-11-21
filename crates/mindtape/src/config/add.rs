//! `[add]` config: the interactive editor fill for `mt add`, and whether a
//! run verifies the task it files.
//!
//! Shapes when the editor opens and which one to spawn, and the condition that
//! decides verification. The fill machinery itself (scaffold, parse-back) lives
//! in the CLI. This module holds config shape only, which keeps the library
//! format-blind.

use serde::{Deserialize, Serialize};

use crate::filter::condition::Condition;
use crate::problem::Problem;

/// Settings for `mt add`: the interactive editor fill, and whether a run
/// verifies the task it files.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AddConfig {
    /// Editor command override. Falls back to `$VISUAL`, then `$EDITOR`, then a
    /// built-in default. May carry arguments (split on whitespace).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    /// When the interactive editor opens.
    pub interactive: InteractiveMode,
    /// Scaffold header override. Absent uses the built-in instructions. An empty
    /// (or all-blank) string disables the header. Any other text replaces it,
    /// comment-prefixed so it stays valid YAML. Ignored when `comments = false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// Whether the scaffold carries explanatory `#` comments. These are the header
    /// block and the per-slot annotations (kind, required/optional, choices,
    /// default). `true` (default) keeps them. `false` strips every comment and
    /// leaves a bare mapping of slot names to values.
    pub comments: bool,
    /// Whether `mt add` verifies the task it files, as a condition over the
    /// invocation's ambient facts: `tty()`, `env("NAME")`, `on_branch("glob")`,
    /// `now()` and `weekday()`. Absent verifies always, so a project that says
    /// nothing keeps the refusal. `!tty()` verifies only a run without a
    /// terminal, which files an incomplete task from a terminal and still
    /// refuses one from an agent. `--verify` and `--no-verify` override it for
    /// one call.
    #[schemars(extend("x-expression" = "condition"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_when: Option<String>,
}

impl Default for AddConfig {
    fn default() -> Self {
        Self {
            editor: None,
            interactive: InteractiveMode::default(),
            header: None,
            comments: true,
            verify_when: None,
        }
    }
}

/// When `mt add` opens the interactive editor.
#[derive(
    Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum InteractiveMode {
    /// Open in a terminal when a required slot is still unfilled (default).
    #[default]
    Auto,
    /// Always open in a terminal, and error when stdin/stdout is not a TTY.
    Always,
    /// Never open, and behave as the non-interactive pipeline.
    Never,
}

/// The `[add]` problems that go beyond TOML shape: a `verify-when` that does
/// not parse as a condition. The parse dry-runs the tree against
/// [`crate::ConditionEnv::probe`]. A typo then fails at config load rather
/// than at the first `mt add`.
#[must_use]
pub(crate) fn validate_add(add: &AddConfig) -> Vec<Problem> {
    let Some(expr) = &add.verify_when else {
        return Vec::new();
    };
    match Condition::parse(expr) {
        Ok(_) => Vec::new(),
        Err(err) => vec![Problem::new(format!("add.verify-when: {err}"))],
    }
}

#[cfg(test)]
#[path = "add_tests.rs"]
mod tests;
