//! The import report: what [`super::map_canonical`] mapped, defaulted,
//! dropped, or flagged while reconciling a canonical record against project
//! config — plus the knobs (`MapOptions`) the CLI exposes over that policy.

use std::collections::BTreeMap;

/// Knobs the CLI (`mt import`) exposes over [`super::map_canonical`]'s policy
/// checks. `Default` matches the strictest behaviour: no status renaming, no
/// silent tag drops.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MapOptions {
    /// `--map old=new` status-name overrides, applied before legend
    /// validation — so a foreign tracker's vocabulary can be renamed onto
    /// this project's legend without failing the import.
    pub status_map: BTreeMap<String, String>,
    /// Drop a tag outside `[tasks.tags] allowed` instead of keeping it and
    /// reporting a violation.
    pub drop_unmapped_tags: bool,
}

/// What kind of event a [`ReportEntry`] records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// An authored value was substituted for another via an explicit option
    /// (currently only `--map` on `status`).
    Mapped,
    /// A missing/empty value was filled in with a project default.
    Defaulted,
    /// A value was dropped from its structured field rather than kept.
    Dropped,
    /// A hard policy problem (out-of-legend status, bad framework input, an
    /// out-of-allow-list tag kept rather than dropped, …). The mapper never
    /// panics on one of these; the CLI decides fail vs skip vs continue.
    Violated,
}

/// One noteworthy event from mapping a canonical record onto project config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportEntry {
    /// Field the entry concerns (`status`, `priority`, `tags`, …).
    pub field: String,
    /// What happened.
    pub kind: EntryKind,
    /// Human-readable detail.
    pub message: String,
}

impl ReportEntry {
    fn new(field: &str, kind: EntryKind, message: impl Into<String>) -> Self {
        Self {
            field: field.to_owned(),
            kind,
            message: message.into(),
        }
    }

    pub(crate) fn mapped(field: &str, message: impl Into<String>) -> Self {
        Self::new(field, EntryKind::Mapped, message)
    }

    pub(crate) fn defaulted(field: &str, message: impl Into<String>) -> Self {
        Self::new(field, EntryKind::Defaulted, message)
    }

    pub(crate) fn dropped(field: &str, message: impl Into<String>) -> Self {
        Self::new(field, EntryKind::Dropped, message)
    }

    pub(crate) fn violated(field: &str, message: impl Into<String>) -> Self {
        Self::new(field, EntryKind::Violated, message)
    }
}

/// The per-record result of [`super::map_canonical`]'s policy pass: every
/// noteworthy event, plus the `x-<ns>` extras keys carried through unchanged.
/// The CLI aggregates these across a batch to report "N filed, M skipped" and
/// to decide fail vs skip vs continue for one record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    /// Mapped/defaulted/dropped/violated events, in the order produced.
    pub entries: Vec<ReportEntry>,
    /// `x-<ns>` extras keys carried through to the task unchanged.
    pub carried_extras: Vec<String>,
}

impl ImportReport {
    /// Whether the mapping produced no noteworthy event at all.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether any entry is a hard policy [`EntryKind::Violated`] problem.
    #[must_use]
    pub fn has_violations(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.kind == EntryKind::Violated)
    }
}
