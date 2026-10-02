//! `[format]` config: whether `mt` reformats the Typst source it writes.
//!
//! Shape only — the formatter itself is [`crate::format_typst`], and the CLI
//! consults [`FormatConfig::enabled`] at each write boundary.

use serde::Deserialize;

/// `[format]` — how `mt` canonicalizes task source before writing it.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FormatConfig {
    /// Whether `mt` runs typstyle over the Typst source it writes (`add`,
    /// `flip`, `review`, `recur`, `import`), so every write leaves the file in
    /// one formatted shape. `true` by default. With `false`, `mt` changes only
    /// the bytes it edits: diffs stay smaller, and stray body indentation can
    /// build up.
    pub enabled: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
