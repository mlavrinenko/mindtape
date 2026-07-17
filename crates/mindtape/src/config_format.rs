//! `[format]` config: whether `mt` reformats the Typst source it writes.
//!
//! Shape only — the formatter itself is [`crate::format_typst`], and the CLI
//! consults [`FormatConfig::enabled`] at each write boundary.

use serde::Deserialize;

/// `[format]` — how `mt` canonicalizes task source before writing it.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FormatConfig {
    /// Run typstyle over the Typst source `mt` writes (`add`, `flip`, `review`,
    /// `recur`, `import`), so surgical edits and slot fills land in one canonical
    /// shape. `true` by default; set `false` to keep writes as raw byte splices
    /// (minimal diffs, at the cost of drift like stray body indentation).
    pub enabled: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
