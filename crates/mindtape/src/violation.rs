//! What a check reports: one violation, and where in the source it points.
//!
//! Separate from [`crate::validate`], which is the checks themselves. Every
//! pass that can fail a task builds one of these — the core field checks, the
//! body-shape gates ([`crate::gate`], [`crate::content_gate`]), the comment
//! scan ([`crate::config_comments`]) and the config-declared rules
//! ([`crate::config_rules`]) — and every consumer reads exactly these fields,
//! so the report type outliving any one pass is the point.
//!
//! Both types are re-exported from [`crate::validate`], where callers have
//! always found them.

use std::path::PathBuf;

use crate::severity::Severity;

/// Source location for a violation that points to a specific line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// File path relative to the project root.
    pub file: PathBuf,
    /// One-based line number.
    pub line: usize,
}

/// A single validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Offending task id.
    pub task_id: String,
    /// Field or aspect at fault (`status`, `priority`, `tags`, `depends-on`,
    /// `rules.<name>`, …).
    pub field: String,
    /// Human-readable message.
    pub message: String,
    /// Where the violation points: the line of the offending field, heading
    /// or comment, else the `task.with(…)` call itself.
    pub location: Location,
    /// How loudly this violation reports, and whether it fails the command.
    ///
    /// Every check `MindTape` makes on its own is [`Severity::Error`]: a
    /// malformed artifact is not a matter of degree. A breach a project chose
    /// the level of — a `[[tasks.rules]]` entry, a `[tasks.comments]` policy of
    /// `warn`, a built-in template under `fallback-severity` — carries that
    /// level. Every report prints it ([`Severity::label`]) and
    /// [`Violation::is_fatal`] answers from it.
    pub severity: Severity,
}

impl Violation {
    /// Whether this violation fails the command it surfaced in — and, for a
    /// write command, refuses the write.
    ///
    /// Fatal exactly when the severity is [`Severity::Error`].
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        self.severity.is_error()
    }
}
