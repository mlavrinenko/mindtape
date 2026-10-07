//! `[add]` config: the interactive editor fill for `mt add`.
//!
//! Shapes when the editor opens and which one to spawn. The fill machinery
//! itself (scaffold, parse-back) lives in the CLI; this module is config shape
//! only, so the library stays format-blind.

use serde::Deserialize;

/// `[add]` — interactive editor fill settings.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct AddConfig {
    /// Editor command override. Falls back to `$VISUAL`, then `$EDITOR`, then a
    /// built-in default. May carry arguments (split on whitespace).
    pub editor: Option<String>,
    /// When the interactive editor opens.
    pub interactive: InteractiveMode,
    /// Scaffold header override. Absent uses the built-in instructions; an empty
    /// (or all-blank) string disables the header; any other text replaces it,
    /// comment-prefixed so it stays valid YAML. Ignored when `comments = false`.
    pub header: Option<String>,
    /// Whether the scaffold carries explanatory `#` comments — the header block
    /// and the per-slot annotations (kind, required/optional, choices, default).
    /// `true` (default) keeps them; `false` strips every comment, leaving a bare
    /// mapping of slot names to values.
    pub comments: bool,
}

impl Default for AddConfig {
    fn default() -> Self {
        Self {
            editor: None,
            interactive: InteractiveMode::default(),
            header: None,
            comments: true,
        }
    }
}

/// When `mt add` opens the interactive editor.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum InteractiveMode {
    /// Open in a terminal when a required slot is still unfilled (default).
    #[default]
    Auto,
    /// Always open in a terminal; error when stdin/stdout is not a TTY.
    Always,
    /// Never open; behave as the non-interactive pipeline.
    Never,
}
