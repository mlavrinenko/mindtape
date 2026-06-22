//! `[add]` config: the interactive editor fill for `mindtape add`.
//!
//! Shapes when the editor opens and which one to spawn. The fill machinery
//! itself (scaffold, parse-back) lives in the CLI; this module is config shape
//! only, so the library stays format-blind.

use serde::Deserialize;

/// `[add]` — interactive editor fill settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct AddConfig {
    /// Editor command override. Falls back to `$VISUAL`, then `$EDITOR`, then a
    /// built-in default. May carry arguments (split on whitespace).
    pub editor: Option<String>,
    /// When the interactive editor opens.
    pub interactive: InteractiveMode,
}

/// When `mindtape add` opens the interactive editor.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
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
